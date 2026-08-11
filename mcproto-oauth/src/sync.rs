//! Synchronous OAuth flows backed by blocking Reqwest and standard-library I/O.

use std::{io::Read, time::Duration};

use reqwest::{
    StatusCode,
    blocking::{Client, Response},
};
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

pub(crate) fn decode_json<T: DeserializeOwned>(
    response: Response,
    service: &'static str,
) -> Result<T, Error> {
    let (status, bytes) = read_response(response, service)?;
    if !status.is_success() {
        return Err(service_error(service, status, &bytes));
    }
    decode_json_bytes(&bytes, service)
}

pub(crate) fn read_response(
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
    let mut chunk = [0_u8; 8192];
    loop {
        let read = response.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        read_response_body(&mut bytes, &chunk[..read], service)?;
    }
    Ok((status, bytes))
}
