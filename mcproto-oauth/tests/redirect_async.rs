use std::{collections::HashMap, net::TcpListener, time::Duration};

use mcproto_oauth::{Error, RedirectFlow};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

const TEST_CLIENT_ID: &str = "18a1a4c2-ccae-4306-9e55-e9500a1793d7";

fn available_redirect() -> (u16, String) {
    let probe = TcpListener::bind("127.0.0.1:0").expect("test port should be available");
    let port = probe
        .local_addr()
        .expect("listener should have an address")
        .port();
    drop(probe);
    (port, format!("http://127.0.0.1:{port}/callback"))
}

#[test]
fn rejects_non_loopback_redirects() {
    let error = RedirectFlow::new(TEST_CLIENT_ID, "https://example.com/callback")
        .err()
        .expect("non-loopback redirect should fail");
    assert!(matches!(error, Error::InvalidRedirectUri(_)));
}

#[tokio::test]
async fn authorization_url_uses_client_id_and_pkce() {
    let (_, redirect_uri) = available_redirect();
    let flow = RedirectFlow::new(TEST_CLIENT_ID, &redirect_uri).expect("flow should be valid");
    let session = flow.start().await.expect("listener should bind");
    let parameters: HashMap<_, _> = session
        .authorization_url()
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();

    assert_eq!(
        parameters.get("client_id").map(String::as_str),
        Some(TEST_CLIENT_ID)
    );
    assert_eq!(
        parameters.get("redirect_uri").map(String::as_str),
        Some(redirect_uri.as_str())
    );
    assert_eq!(
        parameters.get("code_challenge_method").map(String::as_str),
        Some("S256")
    );
    assert_eq!(session.code_verifier().expose().len(), 43);
    assert_eq!(session.code_challenge().len(), 43);
    assert_eq!(session.state().expose().len(), 43);
}

#[tokio::test]
async fn receives_verified_callback_code() {
    let (port, redirect_uri) = available_redirect();
    let flow = RedirectFlow::new(TEST_CLIENT_ID, redirect_uri).expect("flow should be valid");
    let session = flow.start().await.expect("listener should bind");
    let state = session.state().expose().to_owned();
    let receiver = tokio::spawn(async move { session.receive_code().await });

    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("callback should connect");
    let request =
        format!("GET /callback?code=test-code&state={state} HTTP/1.1\r\nHost: localhost\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("request should write");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .expect("response should read");

    let authorization = tokio::time::timeout(Duration::from_secs(5), receiver)
        .await
        .expect("callback should not time out")
        .expect("callback task should not panic")
        .expect("callback should succeed");
    assert_eq!(authorization.code.expose(), "test-code");
    assert!(String::from_utf8_lossy(&response).starts_with("HTTP/1.1 200 OK"));
}
