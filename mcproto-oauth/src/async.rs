//! Asynchronous OAuth flows backed by Tokio and nonblocking Reqwest.

use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;

use crate::{Error, MAX_RESPONSE_BYTES, decode_json_bytes, read_response_body, service_error};

pub mod device_code;
pub mod redirect_uri;
pub mod xbox_login;

pub use device_code::{DeviceCodeFlow, DeviceCodePoll, DeviceCodeSession};
pub use redirect_uri::{RedirectFlow, RedirectSession};
pub use xbox_login::XboxLogin;

pub(crate) fn default_http_client() -> Result<Client, Error> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()?)
}

pub(crate) async fn decode_json<T: DeserializeOwned>(
    response: Response,
    service: &'static str,
) -> Result<T, Error> {
    let (status, bytes) = read_response(response, service).await?;
    if !status.is_success() {
        return Err(service_error(service, status, &bytes));
    }
    decode_json_bytes(&bytes, service)
}

pub(crate) async fn read_response(
    mut response: Response,
    service: &'static str,
) -> Result<(StatusCode, Vec<u8>), Error> {
    let status = response.status();
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(Error::ResponseTooLarge {
            service,
            max_bytes: MAX_RESPONSE_BYTES,
        });
    }

    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        read_response_body(&mut bytes, &chunk, service)?;
    }
    Ok((status, bytes))
}
