//! Synchronous Microsoft OAuth device-code flow.

use std::{
    thread,
    time::{Duration, Instant},
};

use reqwest::{StatusCode, blocking::Client};
use serde::Deserialize;
use url::Url;

use crate::{Error, LoginResult, MicrosoftToken, Secret, decode_json_bytes, service_error};

use super::{XboxLogin, decode_json, default_http_client, read_response};

const DEVICE_CODE_ENDPOINT: &str =
    "https://login.microsoftonline.com/common/oauth2/v2.0/devicecode";
const TOKEN_ENDPOINT: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
const OAUTH_SCOPE: &str = "XboxLive.signin offline_access";
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);
const SLOW_DOWN_INCREMENT: Duration = Duration::from_secs(5);

/// Configuration and blocking HTTP client for device-code login.
#[derive(Clone)]
pub struct DeviceCodeFlow {
    client_id: String,
    http: Client,
}

impl DeviceCodeFlow {
    /// Creates a synchronous device-code flow for an application-owned client ID.
    pub fn new(client_id: impl Into<String>) -> Result<Self, Error> {
        let client_id = client_id.into();
        if client_id.trim().is_empty() || client_id.chars().any(char::is_control) {
            return Err(Error::InvalidClientId);
        }
        Ok(Self {
            client_id,
            http: default_http_client()?,
        })
    }

    /// Replaces the blocking HTTP client used for all token exchanges.
    pub fn with_http_client(mut self, http: Client) -> Self {
        self.http = http;
        self
    }

    /// Requests a device code and creates a blocking polling session.
    pub fn start(&self) -> Result<DeviceCodeSession, Error> {
        let response = self
            .http
            .post(DEVICE_CODE_ENDPOINT)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("scope", OAUTH_SCOPE),
            ])
            .send()?;
        let authorization: RawDeviceAuthorization =
            decode_json(response, "Microsoft device authorization")?;
        DeviceCodeSession::from_response(self.clone(), authorization)
    }

    /// Exchanges a persisted refresh token for a new Microsoft token.
    pub fn exchange_refresh_token(&self, refresh_token: &Secret) -> Result<MicrosoftToken, Error> {
        let response = self
            .http
            .post(TOKEN_ENDPOINT)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("refresh_token", refresh_token.expose()),
                ("grant_type", "refresh_token"),
                ("scope", OAUTH_SCOPE),
            ])
            .send()?;
        decode_json(response, "Microsoft device token")
    }

    /// Refreshes Microsoft and completes Xbox, XSTS, and Minecraft login.
    pub fn login_with_refresh_token(&self, refresh_token: &Secret) -> Result<LoginResult, Error> {
        let microsoft = self.exchange_refresh_token(refresh_token)?;
        XboxLogin::with_http_client(self.http.clone()).login(microsoft)
    }

    fn poll_token(&self, device_code: &Secret) -> Result<RawDevicePoll, Error> {
        let response = self
            .http
            .post(TOKEN_ENDPOINT)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("grant_type", DEVICE_CODE_GRANT),
                ("device_code", device_code.expose()),
            ])
            .send()?;
        let (status, bytes) = read_response(response, "Microsoft device token")?;
        parse_poll_response(status, &bytes)
    }
}

/// A device authorization and its blocking polling state.
pub struct DeviceCodeSession {
    flow: DeviceCodeFlow,
    started_at: Instant,
    poll_interval: Duration,
    /// Short code that the user enters on Microsoft's verification page.
    pub user_code: String,
    /// Secret code sent only to Microsoft's token endpoint.
    pub device_code: Secret,
    /// Page the user should open to authorize the application.
    pub verification_uri: Url,
    /// Verification URL containing the user code, when supplied.
    pub verification_uri_complete: Option<Url>,
    /// Number of seconds for which the device code remains valid.
    pub expires_in: u64,
    /// Initial server-requested polling interval in seconds.
    pub interval: u64,
    /// Human-readable instructions returned by Microsoft.
    pub message: Option<String>,
}

impl DeviceCodeSession {
    fn from_response(
        flow: DeviceCodeFlow,
        response: RawDeviceAuthorization,
    ) -> Result<Self, Error> {
        let verification_uri = Url::parse(&response.verification_uri)
            .map_err(|error| Error::InvalidVerificationUri(error.to_string()))?;
        let verification_uri_complete = response
            .verification_uri_complete
            .map(|value| {
                Url::parse(&value).map_err(|error| Error::InvalidVerificationUri(error.to_string()))
            })
            .transpose()?;
        let interval = response.interval.unwrap_or(DEFAULT_POLL_INTERVAL.as_secs());
        let poll_interval = Duration::from_secs(interval.max(1));
        Ok(Self {
            flow,
            started_at: Instant::now(),
            poll_interval,
            user_code: response.user_code,
            device_code: Secret::new(response.device_code),
            verification_uri,
            verification_uri_complete,
            expires_in: response.expires_in,
            interval,
            message: response.message,
        })
    }

    /// Returns the current recommended delay before the next poll.
    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    /// Returns the remaining local lifetime of the device code.
    pub fn remaining(&self) -> Duration {
        Duration::from_secs(self.expires_in).saturating_sub(self.started_at.elapsed())
    }

    /// Polls Microsoft once without sleeping first.
    pub fn poll_once(&mut self) -> Result<DeviceCodePoll, Error> {
        if self.remaining().is_zero() {
            return Err(expired_locally());
        }
        match self.flow.poll_token(&self.device_code)? {
            RawDevicePoll::Complete(token) => Ok(DeviceCodePoll::Complete(token)),
            RawDevicePoll::Pending { description } => Ok(DeviceCodePoll::AuthorizationPending {
                retry_after: self.poll_interval,
                description,
            }),
            RawDevicePoll::SlowDown { description } => {
                self.poll_interval = self
                    .poll_interval
                    .checked_add(SLOW_DOWN_INCREMENT)
                    .unwrap_or(Duration::MAX);
                Ok(DeviceCodePoll::SlowDown {
                    retry_after: self.poll_interval,
                    description,
                })
            }
        }
    }

    /// Blocks and polls at Microsoft's requested interval until authorized.
    pub fn wait_for_token(mut self) -> Result<MicrosoftToken, Error> {
        loop {
            let delay = self.poll_interval.min(self.remaining());
            if delay.is_zero() {
                return Err(expired_locally());
            }
            thread::sleep(delay);
            match self.poll_once()? {
                DeviceCodePoll::Complete(token) => return Ok(token),
                DeviceCodePoll::AuthorizationPending { .. } | DeviceCodePoll::SlowDown { .. } => {}
            }
        }
    }

    /// Blocks through Microsoft, Xbox, XSTS, and Minecraft login.
    pub fn complete(self) -> Result<LoginResult, Error> {
        let http = self.flow.http.clone();
        let microsoft = self.wait_for_token()?;
        XboxLogin::with_http_client(http).login(microsoft)
    }
}

/// Result of one synchronous device-token polling request.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum DeviceCodePoll {
    /// The user has not completed authorization yet.
    AuthorizationPending {
        /// Minimum recommended delay before polling again.
        retry_after: Duration,
        /// Description returned by Microsoft.
        description: String,
    },
    /// Microsoft requested a slower polling rate.
    SlowDown {
        /// Updated minimum delay before polling again.
        retry_after: Duration,
        /// Description returned by Microsoft.
        description: String,
    },
    /// Microsoft returned access and refresh tokens.
    Complete(MicrosoftToken),
}

#[derive(Deserialize)]
struct RawDeviceAuthorization {
    user_code: String,
    device_code: String,
    verification_uri: String,
    #[serde(default)]
    verification_uri_complete: Option<String>,
    expires_in: u64,
    #[serde(default)]
    interval: Option<u64>,
    #[serde(default)]
    message: Option<String>,
}

enum RawDevicePoll {
    Pending { description: String },
    SlowDown { description: String },
    Complete(MicrosoftToken),
}

#[derive(Deserialize)]
struct DeviceOAuthError {
    error: String,
    #[serde(default)]
    error_description: String,
}

fn parse_poll_response(status: StatusCode, bytes: &[u8]) -> Result<RawDevicePoll, Error> {
    if status.is_success() {
        return decode_json_bytes(bytes, "Microsoft device token").map(RawDevicePoll::Complete);
    }
    let error: DeviceOAuthError = match serde_json::from_slice(bytes) {
        Ok(error) => error,
        Err(_) => return Err(service_error("Microsoft device token", status, bytes)),
    };
    match error.error.as_str() {
        "authorization_pending" => Ok(RawDevicePoll::Pending {
            description: error.error_description,
        }),
        "slow_down" => Ok(RawDevicePoll::SlowDown {
            description: error.error_description,
        }),
        "authorization_declined" => Err(Error::DeviceAuthorizationDeclined {
            description: error.error_description,
        }),
        "expired_token" | "bad_verification_code" => Err(Error::DeviceCodeExpired {
            description: error.error_description,
        }),
        _ => Err(Error::DeviceAuthorization {
            error: error.error,
            description: error.error_description,
        }),
    }
}

fn expired_locally() -> Error {
    Error::DeviceCodeExpired {
        description: "the device code reached its expires_in limit".into(),
    }
}
