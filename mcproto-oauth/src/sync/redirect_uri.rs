//! Synchronous authorization-code login with PKCE and a local redirect listener.

use std::{
    io::{Read, Write},
    net::{IpAddr, TcpListener, TcpStream},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use url::Url;

use crate::{AuthorizationCode, Error, LoginResult, MicrosoftToken, Secret};

use super::{XboxLogin, decode_json, default_http_client};

const AUTHORIZE_ENDPOINT: &str = "https://login.live.com/oauth20_authorize.srf";
const TOKEN_ENDPOINT: &str = "https://login.live.com/oauth20_token.srf";
const OAUTH_SCOPE: &str = "XboxLive.signin offline_access";
const MAX_CALLBACK_BYTES: usize = 16 * 1024;

/// Configuration and blocking HTTP client for the redirect login flow.
#[derive(Clone)]
pub struct RedirectFlow {
    client_id: String,
    redirect_uri: Url,
    http: Client,
}

impl RedirectFlow {
    /// Creates a synchronous redirect flow for an application-owned client ID.
    pub fn new(client_id: impl Into<String>, redirect_uri: impl AsRef<str>) -> Result<Self, Error> {
        let client_id = client_id.into();
        if client_id.trim().is_empty() || client_id.chars().any(char::is_control) {
            return Err(Error::InvalidClientId);
        }
        let redirect_uri = Url::parse(redirect_uri.as_ref())
            .map_err(|error| Error::InvalidRedirectUri(error.to_string()))?;
        validate_redirect_uri(&redirect_uri)?;
        Ok(Self {
            client_id,
            redirect_uri,
            http: default_http_client()?,
        })
    }

    /// Replaces the blocking HTTP client used for all token exchanges.
    pub fn with_http_client(mut self, http: Client) -> Self {
        self.http = http;
        self
    }

    /// Binds the local callback listener and creates an authorization session.
    pub fn start(&self) -> Result<RedirectSession, Error> {
        let host = self
            .redirect_uri
            .host_str()
            .ok_or_else(|| Error::InvalidRedirectUri("missing host".into()))?;
        let port = self
            .redirect_uri
            .port_or_known_default()
            .ok_or_else(|| Error::InvalidRedirectUri("missing port".into()))?;
        let listener = TcpListener::bind((host, port))?;

        let code_verifier = random_base64url()?;
        let state = random_base64url()?;
        let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
        let mut authorization_url = Url::parse(AUTHORIZE_ENDPOINT)
            .map_err(|error| Error::InvalidRedirectUri(error.to_string()))?;
        authorization_url
            .query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", self.redirect_uri.as_str())
            .append_pair("scope", OAUTH_SCOPE)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &state);

        Ok(RedirectSession {
            flow: self.clone(),
            listener,
            authorization_url,
            code_verifier: Secret::new(code_verifier),
            code_challenge,
            state: Secret::new(state),
        })
    }

    /// Exchanges an authorization code and completes all shared login stages.
    pub fn login_with_code(&self, code: AuthorizationCode) -> Result<LoginResult, Error> {
        let microsoft = self.exchange_code(&code)?;
        XboxLogin::with_http_client(self.http.clone()).login(microsoft)
    }

    /// Refreshes a Microsoft token and completes all shared login stages.
    pub fn login_with_refresh_token(&self, refresh_token: &Secret) -> Result<LoginResult, Error> {
        let microsoft = self.exchange_refresh_token(refresh_token)?;
        XboxLogin::with_http_client(self.http.clone()).login(microsoft)
    }

    /// Exchanges an authorization code for a Microsoft token.
    pub fn exchange_code(&self, code: &AuthorizationCode) -> Result<MicrosoftToken, Error> {
        let response = self
            .http
            .post(TOKEN_ENDPOINT)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("code", code.code.expose()),
                ("redirect_uri", self.redirect_uri.as_str()),
                ("grant_type", "authorization_code"),
                ("code_verifier", code.code_verifier.expose()),
            ])
            .send()?;
        decode_json(response, "Microsoft OAuth")
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
        decode_json(response, "Microsoft OAuth")
    }
}

/// A bound blocking listener and its corresponding OAuth request.
pub struct RedirectSession {
    flow: RedirectFlow,
    listener: TcpListener,
    authorization_url: Url,
    code_verifier: Secret,
    code_challenge: String,
    state: Secret,
}

impl RedirectSession {
    /// Returns the Microsoft URL the user should open.
    pub fn authorization_url(&self) -> &Url {
        &self.authorization_url
    }

    /// Returns the generated PKCE verifier.
    pub fn code_verifier(&self) -> &Secret {
        &self.code_verifier
    }

    /// Returns the generated S256 PKCE challenge.
    pub fn code_challenge(&self) -> &str {
        &self.code_challenge
    }

    /// Returns the generated OAuth state.
    pub fn state(&self) -> &Secret {
        &self.state
    }

    /// Blocks until a valid redirect returns an authorization code.
    pub fn receive_code(self) -> Result<AuthorizationCode, Error> {
        loop {
            let (mut stream, _) = self.listener.accept()?;
            let request = read_callback_request(&mut stream)?;
            let target = match callback_target(&request) {
                Some(target) => target,
                None => {
                    send_callback_response(&mut stream, "400 Bad Request", "Invalid request")?;
                    continue;
                }
            };
            let callback = self
                .flow
                .redirect_uri
                .join(target)
                .map_err(|error| Error::InvalidRedirectUri(error.to_string()))?;
            if callback.path() != self.flow.redirect_uri.path() {
                send_callback_response(&mut stream, "404 Not Found", "Not found")?;
                continue;
            }

            let mut code = None;
            let mut state = None;
            let mut oauth_error = None;
            let mut description = String::new();
            for (key, value) in callback.query_pairs() {
                match key.as_ref() {
                    "code" => code = Some(value.into_owned()),
                    "state" => state = Some(value.into_owned()),
                    "error" => oauth_error = Some(value.into_owned()),
                    "error_description" => description = value.into_owned(),
                    _ => {}
                }
            }
            if state.as_deref() != Some(self.state.expose()) {
                send_callback_response(&mut stream, "400 Bad Request", "Invalid OAuth state")?;
                continue;
            }
            if let Some(error) = oauth_error {
                send_callback_response(&mut stream, "400 Bad Request", "Authorization failed")?;
                return Err(Error::Authorization { error, description });
            }
            let code = match code {
                Some(code) => code,
                None => {
                    send_callback_response(
                        &mut stream,
                        "400 Bad Request",
                        "Missing authorization code",
                    )?;
                    return Err(Error::MissingAuthorizationCode);
                }
            };
            send_callback_response(
                &mut stream,
                "200 OK",
                "Login complete. You may close this window.",
            )?;
            return Ok(AuthorizationCode {
                code: Secret::new(code),
                code_verifier: self.code_verifier,
            });
        }
    }

    /// Blocks through Microsoft, Xbox, XSTS, and Minecraft login.
    pub fn complete(self) -> Result<LoginResult, Error> {
        let flow = self.flow.clone();
        let code = self.receive_code()?;
        flow.login_with_code(code)
    }
}

fn validate_redirect_uri(uri: &Url) -> Result<(), Error> {
    if uri.scheme() != "http" {
        return Err(Error::InvalidRedirectUri(
            "local redirects must use HTTP".into(),
        ));
    }
    if !uri.username().is_empty() || uri.password().is_some() || uri.fragment().is_some() {
        return Err(Error::InvalidRedirectUri(
            "credentials and fragments are not allowed".into(),
        ));
    }
    if uri.port() == Some(0) {
        return Err(Error::InvalidRedirectUri(
            "callback port must not be zero".into(),
        ));
    }
    let is_loopback = match uri.host_str() {
        Some(host) if host.eq_ignore_ascii_case("localhost") => true,
        Some(host) => host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback()),
        None => false,
    };
    if !is_loopback {
        return Err(Error::InvalidRedirectUri(
            "host must be localhost or a loopback IP".into(),
        ));
    }
    Ok(())
}

fn random_base64url() -> Result<String, Error> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(Error::Random)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn read_callback_request(stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
    let mut request = Vec::with_capacity(1024);
    loop {
        if request.len() == MAX_CALLBACK_BYTES {
            return Err(Error::CallbackTooLarge);
        }
        let mut buffer = [0_u8; 1024];
        let limit = buffer.len().min(MAX_CALLBACK_BYTES - request.len());
        let read = stream.read(&mut buffer[..limit])?;
        if read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    Ok(request)
}

fn callback_target(request: &[u8]) -> Option<&str> {
    let first_line_end = request.windows(2).position(|window| window == b"\r\n")?;
    let first_line = std::str::from_utf8(&request[..first_line_end]).ok()?;
    let mut parts = first_line.split_ascii_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let target = parts.next()?;
    if parts.next()? != "HTTP/1.1" || parts.next().is_some() {
        return None;
    }
    Some(target)
}

fn send_callback_response(
    stream: &mut TcpStream,
    status: &str,
    message: &str,
) -> Result<(), Error> {
    let body =
        format!("<!doctype html><meta charset=utf-8><title>mcproto OAuth</title><p>{message}</p>");
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
