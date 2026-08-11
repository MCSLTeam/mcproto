use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use mcproto_oauth::sync::RedirectFlow;

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
fn authorization_url_uses_client_id_and_pkce() {
    let (_, redirect_uri) = available_redirect();
    let flow = RedirectFlow::new(TEST_CLIENT_ID, redirect_uri).expect("flow should be valid");
    let session = flow.start().expect("listener should bind");
    assert!(
        session
            .authorization_url()
            .as_str()
            .contains(TEST_CLIENT_ID)
    );
    assert_eq!(session.code_verifier().expose().len(), 43);
    assert_eq!(session.code_challenge().len(), 43);
}

#[test]
fn receives_verified_callback_code() {
    let (port, redirect_uri) = available_redirect();
    let flow = RedirectFlow::new(TEST_CLIENT_ID, redirect_uri).expect("flow should be valid");
    let session = flow.start().expect("listener should bind");
    let state = session.state().expose().to_owned();
    let receiver = std::thread::spawn(move || session.receive_code());

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("callback should connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout should set");
    let request =
        format!("GET /callback?code=test-code&state={state} HTTP/1.1\r\nHost: localhost\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("request should write");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("response should read");

    let authorization = receiver
        .join()
        .expect("callback thread should not panic")
        .expect("callback should succeed");
    assert_eq!(authorization.code.expose(), "test-code");
    assert!(String::from_utf8_lossy(&response).starts_with("HTTP/1.1 200 OK"));
}
