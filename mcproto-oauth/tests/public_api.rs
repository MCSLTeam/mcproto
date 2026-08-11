use mcproto_oauth::{
    DeviceCodeFlow, DeviceCodePoll, Error, MicrosoftToken, Secret, XstsToken, sync,
};

const TEST_CLIENT_ID: &str = "18a1a4c2-ccae-4306-9e55-e9500a1793d7";

#[test]
fn device_flows_require_application_client_ids() {
    assert!(DeviceCodeFlow::new(TEST_CLIENT_ID).is_ok());
    assert!(sync::DeviceCodeFlow::new(TEST_CLIENT_ID).is_ok());
    assert!(matches!(
        DeviceCodeFlow::new(""),
        Err(Error::InvalidClientId)
    ));
    assert!(matches!(
        sync::DeviceCodeFlow::new(""),
        Err(Error::InvalidClientId)
    ));
}

#[test]
fn secrets_are_redacted_but_explicitly_accessible() {
    let secret = Secret::new("sensitive-token");
    assert_eq!(secret.expose(), "sensitive-token");
    assert_eq!(format!("{secret:?}"), "Secret([REDACTED])");
}

#[test]
fn shared_xsts_token_builds_minecraft_identity_token() {
    let xsts = XstsToken {
        issue_instant: "issued".into(),
        not_after: "expires".into(),
        token: Secret::new("xsts-token"),
        user_hash: "user-hash".into(),
    };
    assert_eq!(
        xsts.minecraft_identity_token().expose(),
        "XBL3.0 x=user-hash;xsts-token"
    );
}

#[test]
fn asynchronous_poll_result_exposes_microsoft_token() {
    let poll = DeviceCodePoll::Complete(MicrosoftToken {
        token_type: "Bearer".into(),
        expires_in: 3600,
        scope: "XboxLive.signin".into(),
        access_token: Secret::new("access"),
        refresh_token: Some(Secret::new("refresh")),
    });
    let DeviceCodePoll::Complete(token) = poll else {
        panic!("poll result should be complete");
    };
    assert_eq!(token.access_token.expose(), "access");
}
