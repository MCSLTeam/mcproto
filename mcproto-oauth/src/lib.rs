//! Microsoft OAuth login flows for Minecraft.
//!
//! Authentication is available through the asynchronous top-level API and
//! the blocking [`sync`] API. Both are split into independent acquisition
//! flows and a shared Xbox-to-Minecraft exchange:
//!
//! - [`redirect_uri`] implements authorization-code login with PKCE and a
//!   local redirect listener.
//! - [`device_code`] implements device authorization and standards-compliant
//!   token polling.
//! - [`xbox_login`] exchanges a Microsoft access token through Xbox Live,
//!   XSTS, and Minecraft Services.

use std::fmt;

use reqwest::StatusCode;
use serde::{Deserialize, de::DeserializeOwned};

pub mod r#async;
pub mod sync;

pub use r#async as asynchronous;
pub use r#async::{
    DeviceCodeFlow, DeviceCodePoll, DeviceCodeSession, RedirectFlow, RedirectSession, XboxLogin,
    device_code, redirect_uri, xbox_login,
};

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

/// An OAuth or Minecraft authentication failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The Microsoft client ID was empty or contained control characters.
    #[error("the Microsoft client ID is invalid")]
    InvalidClientId,
    /// The redirect URI is not an HTTP loopback URL suitable for a local listener.
    #[error("invalid redirect URI: {0}")]
    InvalidRedirectUri(String),
    /// Secure random generation failed.
    #[error("failed to generate OAuth security values: {0}")]
    Random(getrandom::Error),
    /// A local listener or blocking response I/O operation failed.
    #[error("I/O operation failed: {0}")]
    Io(#[from] std::io::Error),
    /// An HTTP request could not be sent or received.
    #[error("request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// An authentication service returned a non-success status.
    #[error("{service} returned HTTP {status}: {body}")]
    Service {
        /// The service whose request failed.
        service: &'static str,
        /// The returned HTTP status.
        status: StatusCode,
        /// A bounded response-body excerpt.
        body: String,
    },
    /// A success response did not match the expected JSON schema.
    #[error("invalid response from {service}: {source}")]
    InvalidResponse {
        /// The service whose response was invalid.
        service: &'static str,
        /// The JSON decoding error.
        #[source]
        source: serde_json::Error,
    },
    /// A service response exceeded the safety limit.
    #[error("response from {service} exceeds {max_bytes} bytes")]
    ResponseTooLarge {
        /// The service whose response was too large.
        service: &'static str,
        /// The configured response limit.
        max_bytes: usize,
    },
    /// The callback request exceeded the local request limit.
    #[error("OAuth callback request exceeds the local request limit")]
    CallbackTooLarge,
    /// The callback did not contain an authorization code.
    #[error("OAuth callback did not contain an authorization code")]
    MissingAuthorizationCode,
    /// Microsoft redirected back with an OAuth error.
    #[error("Microsoft authorization failed: {error}: {description}")]
    Authorization {
        /// OAuth error code.
        error: String,
        /// OAuth error description, when supplied.
        description: String,
    },
    /// Microsoft reported that the user declined device-code authorization.
    #[error("Microsoft device authorization was declined: {description}")]
    DeviceAuthorizationDeclined {
        /// Description returned by Microsoft.
        description: String,
    },
    /// A device code expired before authorization completed.
    #[error("Microsoft device code expired: {description}")]
    DeviceCodeExpired {
        /// Description returned by Microsoft or generated locally.
        description: String,
    },
    /// Microsoft returned an unrecognized device-code OAuth error.
    #[error("Microsoft device authorization failed: {error}: {description}")]
    DeviceAuthorization {
        /// OAuth error code.
        error: String,
        /// OAuth error description.
        description: String,
    },
    /// The verification URI in Microsoft's device-code response was invalid.
    #[error("invalid device-code verification URI: {0}")]
    InvalidVerificationUri(String),
    /// Xbox did not return the user hash required by Minecraft Services.
    #[error("Xbox response did not contain a user hash")]
    MissingUserHash,
}

/// A secret token whose debug representation is always redacted.
#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    /// Wraps a token obtained from secure application storage.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Exposes the secret value for authenticated API calls or persistence.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret([REDACTED])")
    }
}

/// An authorization code paired with the PKCE verifier that created it.
#[derive(Clone, Debug)]
pub struct AuthorizationCode {
    /// Code returned by the Microsoft authorization redirect.
    pub code: Secret,
    /// PKCE verifier required when exchanging the authorization code.
    pub code_verifier: Secret,
}

/// Tokens returned by a Microsoft OAuth token endpoint.
#[derive(Clone, Debug, Deserialize)]
pub struct MicrosoftToken {
    /// Token scheme, normally `bearer`.
    pub token_type: String,
    /// Lifetime of the access token in seconds.
    pub expires_in: u64,
    /// Granted OAuth scopes.
    #[serde(default)]
    pub scope: String,
    /// Access token used to authenticate with Xbox Live.
    pub access_token: Secret,
    /// Refresh token, present when Microsoft grants `offline_access`.
    #[serde(default)]
    pub refresh_token: Option<Secret>,
}

/// Token and display claim returned by Xbox Live user authentication.
#[derive(Clone, Debug)]
pub struct XboxLiveToken {
    /// Time at which Xbox Live issued the token.
    pub issue_instant: String,
    /// Time after which the token is no longer valid.
    pub not_after: String,
    /// Xbox Live user token.
    pub token: Secret,
    /// Xbox user hash from the display claims.
    pub user_hash: String,
}

/// Token and display claim returned by XSTS authorization.
#[derive(Clone, Debug)]
pub struct XstsToken {
    /// Time at which XSTS issued the token.
    pub issue_instant: String,
    /// Time after which the token is no longer valid.
    pub not_after: String,
    /// XSTS token authorized for Minecraft Services.
    pub token: Secret,
    /// Xbox user hash used in the Minecraft identity token.
    pub user_hash: String,
}

impl XstsToken {
    /// Builds the identity token sent to Minecraft Services.
    pub fn minecraft_identity_token(&self) -> Secret {
        Secret::new(format!(
            "XBL3.0 x={};{}",
            self.user_hash,
            self.token.expose()
        ))
    }
}

/// A Minecraft Services access token.
#[derive(Clone, Debug, Deserialize)]
pub struct MinecraftToken {
    /// Minecraft account identifier returned by the login endpoint.
    pub username: String,
    /// Bearer token for Minecraft Services.
    pub access_token: Secret,
    /// Token scheme, normally `Bearer`.
    pub token_type: String,
    /// Lifetime of the token in seconds.
    pub expires_in: u64,
}

/// The complete result of a Microsoft-to-Minecraft login.
#[derive(Clone, Debug)]
pub struct LoginResult {
    /// Microsoft tokens, including the refresh token when granted.
    pub microsoft: MicrosoftToken,
    /// Xbox Live user token and user hash.
    pub xbox_live: XboxLiveToken,
    /// XSTS token authorized for Minecraft Services.
    pub xsts: XstsToken,
    /// Minecraft Services token.
    pub minecraft: MinecraftToken,
}

pub(crate) fn decode_json_bytes<T: DeserializeOwned>(
    bytes: &[u8],
    service: &'static str,
) -> Result<T, Error> {
    serde_json::from_slice(bytes).map_err(|source| Error::InvalidResponse { service, source })
}

pub(crate) fn read_response_body(
    destination: &mut Vec<u8>,
    chunk: &[u8],
    service: &'static str,
) -> Result<(), Error> {
    if destination.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
        return Err(Error::ResponseTooLarge {
            service,
            max_bytes: MAX_RESPONSE_BYTES,
        });
    }
    destination.extend_from_slice(chunk);
    Ok(())
}

pub(crate) fn service_error(service: &'static str, status: StatusCode, bytes: &[u8]) -> Error {
    Error::Service {
        service,
        status,
        body: String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]).into_owned(),
    }
}
