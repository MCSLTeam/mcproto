//! Synchronous Xbox Live, XSTS, and Minecraft Services token exchange.

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{Error, LoginResult, MicrosoftToken, MinecraftToken, Secret, XboxLiveToken, XstsToken};

use super::{decode_json, default_http_client};

const XBOX_ENDPOINT: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_ENDPOINT: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_ENDPOINT: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";

/// Synchronously exchanges Microsoft access tokens into Minecraft tokens.
#[derive(Clone)]
pub struct XboxLogin {
    http: Client,
}

impl XboxLogin {
    /// Creates a token exchanger with a bounded, non-redirecting HTTP client.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            http: default_http_client()?,
        })
    }

    /// Creates a token exchanger using an application-provided blocking client.
    pub fn with_http_client(http: Client) -> Self {
        Self { http }
    }

    /// Completes every shared login stage and retains all intermediate results.
    pub fn login(&self, microsoft: MicrosoftToken) -> Result<LoginResult, Error> {
        let xbox_live = self.authenticate_xbox_live(&microsoft.access_token)?;
        let xsts = self.authorize_xsts(&xbox_live)?;
        let minecraft = self.authenticate_minecraft(&xsts)?;
        Ok(LoginResult {
            microsoft,
            xbox_live,
            xsts,
            minecraft,
        })
    }

    /// Authenticates a Microsoft access token with Xbox Live.
    pub fn authenticate_xbox_live(
        &self,
        microsoft_access_token: &Secret,
    ) -> Result<XboxLiveToken, Error> {
        let response = self
            .http
            .post(XBOX_ENDPOINT)
            .json(&serde_json::json!({
                "Properties": {
                    "AuthMethod": "RPS",
                    "SiteName": "user.auth.xboxlive.com",
                    "RpsTicket": format!("d={}", microsoft_access_token.expose())
                },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT"
            }))
            .send()?;
        let token: RawXboxToken = decode_json(response, "Xbox Live")?;
        let user_hash = token.user_hash()?;
        Ok(XboxLiveToken {
            issue_instant: token.issue_instant,
            not_after: token.not_after,
            token: token.token,
            user_hash,
        })
    }

    /// Authorizes an Xbox Live user token with XSTS for Minecraft Services.
    pub fn authorize_xsts(&self, xbox_live: &XboxLiveToken) -> Result<XstsToken, Error> {
        let response = self
            .http
            .post(XSTS_ENDPOINT)
            .json(&serde_json::json!({
                "Properties": {
                    "SandboxId": "RETAIL",
                    "UserTokens": [xbox_live.token.expose()]
                },
                "RelyingParty": "rp://api.minecraftservices.com/",
                "TokenType": "JWT"
            }))
            .send()?;
        let token: RawXboxToken = decode_json(response, "XSTS")?;
        let user_hash = token.user_hash()?;
        Ok(XstsToken {
            issue_instant: token.issue_instant,
            not_after: token.not_after,
            token: token.token,
            user_hash,
        })
    }

    /// Exchanges an XSTS token for a Minecraft Services access token.
    pub fn authenticate_minecraft(&self, xsts: &XstsToken) -> Result<MinecraftToken, Error> {
        let identity_token = xsts.minecraft_identity_token();
        let response = self
            .http
            .post(MINECRAFT_ENDPOINT)
            .json(&serde_json::json!({
                "identityToken": identity_token.expose()
            }))
            .send()?;
        decode_json(response, "Minecraft Services")
    }

    /// Exchanges a Microsoft access token through all shared stages.
    pub fn minecraft_token(
        &self,
        microsoft_access_token: &Secret,
    ) -> Result<MinecraftToken, Error> {
        let xbox_live = self.authenticate_xbox_live(microsoft_access_token)?;
        let xsts = self.authorize_xsts(&xbox_live)?;
        self.authenticate_minecraft(&xsts)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawXboxToken {
    issue_instant: String,
    not_after: String,
    token: Secret,
    display_claims: XboxDisplayClaims,
}

impl RawXboxToken {
    fn user_hash(&self) -> Result<String, Error> {
        self.display_claims
            .xui
            .first()
            .map(|claim| claim.uhs.clone())
            .ok_or(Error::MissingUserHash)
    }
}

#[derive(Deserialize)]
struct XboxDisplayClaims {
    xui: Vec<XboxUserClaim>,
}

#[derive(Deserialize)]
struct XboxUserClaim {
    uhs: String,
}
