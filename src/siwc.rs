//! Sign in with ChatGPT: lets an eligible ChatGPT Plus or Pro subscriber run
//! Polish on their plan allowance through OpenAI's official OAuth flow for
//! open-source clients. No client secret, no partner API key.
//!
//! Contract: https://developers.openai.com/siwc/token-sharing-open-source/sign-in

use crate::{
    config::{Config, Registration, load_config, save_config},
    credentials,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const AUTH_BASE: &str = "https://auth.openai.com";
const RESOURCE: &str = "https://api.openai.com/v1";
const PLAN_SCOPE: &str = "chatgpt.tokens.use.direct";
const REQUESTED_SCOPES: &str =
    "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
const CALLBACK_PATH: &str = "/auth/callback";
const CALLBACK_DEADLINE: Duration = Duration::from_secs(300);
/// Refresh the access token this long before it expires.
const EXPIRY_MARGIN: u64 = 120;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TokenRecord {
    pub(crate) email: String,
    pub(crate) subject: String,
    pub(crate) client_id: String,
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    #[serde(default)]
    pub(crate) id_token: String,
    #[serde(default)]
    pub(crate) scopes: Vec<String>,
    pub(crate) expires_at: u64,
}
impl TokenRecord {
    pub(crate) fn plan_usage(&self) -> bool {
        self.scopes.iter().any(|scope| scope == PLAN_SCOPE)
    }
}

pub(crate) struct SignInOutcome {
    pub(crate) email: String,
    pub(crate) plan_usage: bool,
    pub(crate) catalog: Vec<(String, String)>,
}

pub(crate) struct PlanCheck {
    pub(crate) email: String,
    pub(crate) plan_usage: bool,
    pub(crate) catalog: Vec<(String, String)>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn http_client(timeout: Duration) -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())
}

fn post_form(url: &str, form: &[(&str, &str)]) -> Result<(reqwest::StatusCode, String), String> {
    let client = http_client(Duration::from_secs(30))?;
    let res = client
        .post(url)
        .form(form)
        .send()
        .map_err(|e| format!("Could not reach OpenAI: {e}"))?;
    let status = res.status();
    let body = res
        .text()
        .map_err(|_| "OpenAI returned an invalid response".to_string())?;
    Ok((status, body))
}

fn read_record() -> Result<TokenRecord, String> {
    let raw = credentials::read_oauth()
        .map_err(|_| "Open Polish Settings to sign in with ChatGPT again.".to_string())?;
    serde_json::from_str(&raw)
        .map_err(|_| "Open Polish Settings to sign in with ChatGPT again.".to_string())
}

fn write_record(record: &TokenRecord) -> Result<(), String> {
    let raw = serde_json::to_string(record).map_err(|e| e.to_string())?;
    credentials::write_oauth(&raw)
}

fn random_url_safe(len: usize) -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn percent_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or_default();
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[derive(Debug, PartialEq, Eq)]
enum Callback {
    Success {
        code: String,
        client_id: Option<String>,
    },
    Declined,
}

fn parse_callback(query: &str, expected_state: &str) -> Result<Callback, String> {
    let mut params: Vec<(String, String)> = Vec::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut it = pair.splitn(2, '=');
        let key = it.next().unwrap_or_default().to_string();
        let value = percent_decode(it.next().unwrap_or_default());
        params.push((key, value));
    }
    let get = |name: &str| {
        params
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    };
    if get("error").is_some() {
        return Ok(Callback::Declined);
    }
    if get("state").as_deref() != Some(expected_state) {
        return Err("The sign-in response did not match this attempt. Try again.".into());
    }
    let Some(code) = get("code") else {
        return Err("The sign-in response was incomplete. Try again.".into());
    };
    Ok(Callback::Success {
        code,
        client_id: get("client_id"),
    })
}

fn read_request_head(stream: &mut TcpStream) -> String {
    let mut raw = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        if raw.len() > 8192 || raw.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
        }
    }
    String::from_utf8_lossy(&raw).into_owned()
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn done_page(message: &str) -> String {
    format!(
        r#"<html><head><meta charset="utf-8"><title>Polish</title></head>
<body style="font-family:-apple-system;background:#f9f3ef;color:#38282a;display:flex;align-items:center;justify-content:center;height:100vh;margin:0">
<div style="font-size:17px">{message}</div></body></html>"#
    )
}

fn wait_for_callback(listener: &TcpListener, expected_state: &str) -> Result<Callback, String> {
    let deadline = Instant::now() + CALLBACK_DEADLINE;
    loop {
        if Instant::now() > deadline {
            return Err("Sign-in timed out. Try again.".into());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
                let head = read_request_head(&mut stream);
                let path = head
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or_default()
                    .to_string();
                let Some(query) = path.strip_prefix(&format!("{CALLBACK_PATH}?")) else {
                    respond(
                        &mut stream,
                        "404 Not Found",
                        &done_page("Polish is waiting for sign-in."),
                    );
                    continue;
                };
                let result = parse_callback(query, expected_state);
                match &result {
                    Ok(Callback::Success { .. }) => respond(
                        &mut stream,
                        "200 OK",
                        &done_page("Signed in to Polish. You can close this tab."),
                    ),
                    _ => respond(
                        &mut stream,
                        "200 OK",
                        &done_page("Sign-in not completed. You can close this tab."),
                    ),
                }
                return result;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(150));
            }
            Err(_) => continue,
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    id_token: String,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    scope: Option<String>,
}

fn exchange_code(
    code: &str,
    client_id: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    let (status, body) = post_form(
        &format!("{AUTH_BASE}/api/accounts/oauth/token"),
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("code_verifier", verifier),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("resource", RESOURCE),
        ],
    )?;
    if !status.is_success() {
        // invalid_grant: discard the code; the saved registration lets the next
        // attempt reuse the issued client without re-registering.
        return Err("OpenAI could not finish the sign-in. Try Continue with ChatGPT again.".into());
    }
    serde_json::from_str(&body).map_err(|_| "OpenAI returned an unexpected sign-in response".into())
}

#[derive(Deserialize)]
struct OpenIdConfiguration {
    jwks_uri: String,
    #[serde(default)]
    issuer: String,
    #[serde(default)]
    revocation_endpoint: Option<String>,
}

fn discovery() -> Result<OpenIdConfiguration, String> {
    let client = http_client(Duration::from_secs(15))?;
    let value: serde_json::Value = client
        .get(format!("{AUTH_BASE}/.well-known/openid-configuration"))
        .send()
        .map_err(|e| format!("Could not reach OpenAI: {e}"))?
        .json()
        .map_err(|_| "OpenAI returned an invalid discovery response".to_string())?;
    serde_json::from_value(value)
        .map_err(|_| "OpenAI returned an invalid discovery response".to_string())
}

struct IdClaims {
    subject: String,
    email: String,
}

fn validate_id_token(id_token: &str, audience: &str, nonce: &str) -> Result<IdClaims, String> {
    use jsonwebtoken::{Algorithm, DecodingKey, Validation};
    let incomplete = "OpenAI's sign-in response was incomplete. Try again.";
    let mut parts = id_token.split('.');
    let header = parts.next().ok_or(incomplete)?.to_string();
    let _claims_b64 = parts.next().ok_or(incomplete)?.to_string();
    let header: serde_json::Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(header.as_bytes())
            .map_err(|_| incomplete)?,
    )
    .map_err(|_| incomplete)?;
    let kid = header["kid"].as_str().ok_or(incomplete)?;
    let config = discovery()?;
    let jwks: serde_json::Value = {
        let client = http_client(Duration::from_secs(15))?;
        client
            .get(&config.jwks_uri)
            .send()
            .map_err(|e| format!("Could not reach OpenAI: {e}"))?
            .json()
            .map_err(|_| "OpenAI returned an invalid key set".to_string())?
    };
    let key = jwks["keys"]
        .as_array()
        .and_then(|keys| keys.iter().find(|key| key["kid"].as_str() == Some(kid)))
        .ok_or("OpenAI sign-in keys changed. Try signing in again.")?;
    let modulus = key["n"].as_str().ok_or("OpenAI key set incomplete")?;
    let exponent = key["e"].as_str().ok_or("OpenAI key set incomplete")?;
    let decoding = DecodingKey::from_rsa_components(modulus, exponent)
        .map_err(|_| "Could not read OpenAI sign-in keys".to_string())?;
    let mut validation = Validation::new(Algorithm::RS256);
    if !config.issuer.is_empty() {
        validation.set_issuer(&[config.issuer]);
    }
    validation.set_audience(&[audience]);
    let data = jsonwebtoken::decode::<serde_json::Value>(id_token, &decoding, &validation)
        .map_err(|_| "OpenAI's sign-in response could not be validated. Try again.".to_string())?;
    let claims = &data.claims;
    if claims["nonce"].as_str() != Some(nonce) {
        return Err("The sign-in response did not match this attempt. Try again.".into());
    }
    let subject = claims["sub"].as_str().ok_or(incomplete)?.to_string();
    let email = claims["email"].as_str().unwrap_or_default().to_string();
    Ok(IdClaims { subject, email })
}

/// `ext_agent_host_id` must be an opaque URI-formatted identifier; OpenAI
/// accepts `urn:ietf:params:oauth:jwk-thumbprint:…`, `urn:uuid:…`, or
/// `did:key:…`. See https://developers.openai.com/siwc/token-sharing-open-source
fn valid_host_id(id: &str) -> bool {
    id.starts_with("urn:uuid:")
        || id.starts_with("urn:ietf:params:oauth:jwk-thumbprint:")
        || id.starts_with("did:key:")
}

/// Hyphenated UUIDv4 per RFC 9562 §5.4.
fn uuid_v4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

fn ensure_host_id(cfg: &mut Config) -> Result<(), String> {
    if cfg.host_id.as_deref().is_some_and(valid_host_id) {
        return Ok(());
    }
    // Regenerate host IDs saved in a rejected format before the first
    // successful sign-in; no registration depends on them yet.
    cfg.host_id = Some(format!("urn:uuid:{}", uuid_v4()));
    save_config(cfg)
}

fn app_support_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("Home directory unavailable")?;
    Ok(std::path::PathBuf::from(home).join("Library/Application Support/Polish"))
}

/// The GUI and the menu-bar helper are two processes sharing one rotating
/// refresh token, so every refresh happens inside this cross-process lock.
fn with_refresh_lock<T>(work: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let dir = app_support_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("siwc.lock"))
        .map_err(|e| e.to_string())?;
    use fs2::FileExt;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => break,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            Err(_) => return Err("Timed out waiting for another Polish action. Try again.".into()),
        }
    }
    let result = work();
    let _ = FileExt::unlock(&file);
    result
}

fn refresh_error_is_terminal(error: &str) -> bool {
    matches!(
        error,
        "invalid_grant"
            | "invalid_refresh_token"
            | "token_expired"
            | "refresh_token_expired"
            | "refresh_token_invalidated"
            | "refresh_token_reused"
    )
}

fn refresh_record(record: &TokenRecord) -> Result<String, String> {
    let (status, body) = post_form(
        &format!("{AUTH_BASE}/api/accounts/oauth/token"),
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &record.refresh_token),
            ("client_id", &record.client_id),
            ("resource", RESOURCE),
        ],
    )?;
    let error_code = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|value| value["error"].as_str().map(str::to_string));
    if !status.is_success() {
        if error_code.as_deref().is_some_and(refresh_error_is_terminal) {
            let _ = credentials::remove_oauth();
            return Err(
                "Your ChatGPT session ended. Open Polish Settings to sign in again.".into(),
            );
        }
        return Err(
            "Could not renew your ChatGPT session. Check your connection and try again.".into(),
        );
    }
    let response: TokenResponse = serde_json::from_str(&body)
        .map_err(|_| "OpenAI returned an unexpected session response".to_string())?;
    let mut updated = record.clone();
    updated.access_token = response.access_token.clone();
    if !response.refresh_token.is_empty() {
        updated.refresh_token = response.refresh_token;
    }
    if !response.id_token.is_empty() {
        updated.id_token = response.id_token;
    }
    if let Some(scopes) = response.scope {
        updated.scopes = scopes.split_whitespace().map(str::to_string).collect();
    }
    updated.expires_at = now() + response.expires_in.unwrap_or(3600).max(60);
    let token = updated.access_token.clone();
    write_record(&updated)?;
    Ok(token)
}

/// A fresh-enough access token for one request, refreshing under the
/// cross-process lock when needed.
pub(crate) fn access_token() -> Result<String, String> {
    let record = read_record()?;
    if now() + EXPIRY_MARGIN < record.expires_at {
        return Ok(record.access_token);
    }
    with_refresh_lock(|| {
        let record = read_record()?;
        if now() + EXPIRY_MARGIN < record.expires_at {
            return Ok(record.access_token);
        }
        refresh_record(&record)
    })
}

/// Parses the plan-route model catalog. The plan flow returns `{models: [...]}`;
/// key-route responses keep the `{data: [...]}` shape as a fallback.
pub(crate) fn parse_catalog(body: &serde_json::Value) -> Vec<(String, String)> {
    if let Some(models) = body["models"].as_array() {
        return models
            .iter()
            .filter(|model| model["visibility"].as_str() == Some("list"))
            .filter_map(|model| {
                let slug = model["slug"].as_str()?.to_string();
                let display = model["display_name"].as_str().unwrap_or(&slug).to_string();
                Some((slug, display))
            })
            .collect();
    }
    if let Some(models) = body["data"].as_array() {
        return models
            .iter()
            .filter_map(|model| {
                let slug = model["id"].as_str()?.to_string();
                Some((slug.clone(), slug))
            })
            .collect();
    }
    Vec::new()
}

pub(crate) fn fetch_catalog(token: &str) -> Result<Vec<(String, String)>, String> {
    let client = http_client(Duration::from_secs(15))?;
    let res = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(token)
        .send()
        .map_err(|e| format!("Could not reach OpenAI: {e}"))?;
    let status = res.status();
    let body: serde_json::Value = res
        .json()
        .map_err(|_| "OpenAI returned an invalid response".to_string())?;
    if !status.is_success() {
        return Err("Could not load the models on your ChatGPT plan. Try signing in again.".into());
    }
    Ok(parse_catalog(&body))
}

pub(crate) fn sign_in() -> Result<SignInOutcome, String> {
    let mut cfg = load_config();
    ensure_host_id(&mut cfg)?;
    let host_id = cfg.host_id.clone().unwrap_or_default();
    let registration = cfg.chatgpt_registration.clone();
    let retained_id_token = read_record()
        .ok()
        .map(|record| record.id_token)
        .unwrap_or_default();

    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|_| "Could not start the sign-in listener".to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");

    let state = random_url_safe(32);
    let nonce = random_url_safe(32);
    let verifier = random_url_safe(64);

    let mut params: Vec<(&str, String)> = vec![
        ("response_type", "code".into()),
        ("redirect_uri", redirect_uri.clone()),
        ("scope", REQUESTED_SCOPES.into()),
        ("resource", RESOURCE.into()),
        ("state", state.clone()),
        ("nonce", nonce.clone()),
        ("code_challenge_method", "S256".into()),
        ("code_challenge", pkce_challenge(&verifier)),
        ("ext_agent_host_id", host_id),
    ];
    match &registration {
        Some(reg) => {
            params.push(("client_id", reg.client_id.clone()));
            if !reg.email.is_empty() {
                params.push(("login_hint", reg.email.clone()));
            }
            if !retained_id_token.is_empty() {
                params.push(("id_token_hint", retained_id_token));
            }
        }
        None => {
            params.push(("client_id", "dynamic_agent_client".into()));
            params.push(("agent_name_hint", "Polish".into()));
        }
    }
    let query = params
        .iter()
        .map(|(key, value)| format!("{key}={}", percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    let url = format!("{AUTH_BASE}/api/accounts/authorize?{query}");

    std::process::Command::new("open")
        .arg(&url)
        .status()
        .map_err(|_| "Could not open your browser.".to_string())?;

    let callback = wait_for_callback(&listener, &state)?;
    let (code, callback_client_id) = match callback {
        Callback::Declined => {
            return Err("Sign-in or plan usage was declined in your browser. You can add an OpenAI API key instead.".into())
        }
        Callback::Success { code, client_id } => (code, client_id),
    };

    // A returning registration is authoritative: reject a callback that swaps it.
    let client_id = match (&registration, callback_client_id) {
        (Some(reg), Some(issued)) if issued != reg.client_id => {
            return Err(
                "The browser returned a different registration. Try signing in again.".into(),
            );
        }
        (Some(reg), _) => reg.client_id.clone(),
        (None, Some(issued)) => {
            // First registration: persist the issued client before exchanging,
            // so a failed exchange retries with the same client next time.
            cfg.chatgpt_registration = Some(Registration {
                client_id: issued.clone(),
                email: String::new(),
                subject: String::new(),
            });
            save_config(&cfg).map_err(|e| e.to_string())?;
            issued
        }
        (None, None) => {
            return Err("OpenAI did not finish registration. Try signing in again.".into());
        }
    };

    let response = exchange_code(&code, &client_id, &verifier, &redirect_uri)?;
    let claims = validate_id_token(&response.id_token, &client_id, &nonce)?;
    if let Some(reg) = &registration {
        if !reg.subject.is_empty() && reg.subject != claims.subject {
            return Err("That account does not match the one signed in to Polish. Sign out first to switch accounts.".into());
        }
    }
    let scopes: Vec<String> = response
        .scope
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let plan_usage = scopes.iter().any(|scope| scope == PLAN_SCOPE);
    let record = TokenRecord {
        email: claims.email.clone(),
        subject: claims.subject.clone(),
        client_id: client_id.clone(),
        access_token: response.access_token.clone(),
        refresh_token: response.refresh_token.clone(),
        id_token: response.id_token.clone(),
        scopes,
        expires_at: now() + response.expires_in.unwrap_or(3600).max(60),
    };
    write_record(&record)?;
    let mut cfg = load_config();
    cfg.auth = "chatgpt".into();
    cfg.chatgpt_registration = Some(Registration {
        client_id: client_id.clone(),
        email: record.email.clone(),
        subject: record.subject.clone(),
    });
    save_config(&cfg).map_err(|e| e.to_string())?;

    let catalog = if plan_usage {
        fetch_catalog(&record.access_token).unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(SignInOutcome {
        email: record.email,
        plan_usage,
        catalog,
    })
}

/// Display-only helpers for the settings view: the signed-in account and
/// whether plan usage was granted.
pub(crate) fn signed_in_email() -> Option<String> {
    read_record()
        .ok()
        .map(|record| record.email)
        .filter(|email| !email.is_empty())
}

pub(crate) fn signed_in_plan_usage() -> bool {
    read_record()
        .map(|record| record.plan_usage())
        .unwrap_or(false)
}

pub(crate) fn verify_plan() -> Result<PlanCheck, String> {
    let record = read_record()?;
    let plan_usage = record.plan_usage();
    let catalog = if plan_usage {
        let token = access_token()?;
        fetch_catalog(&token)?
    } else {
        Vec::new()
    };
    Ok(PlanCheck {
        email: record.email,
        plan_usage,
        catalog,
    })
}

fn revoke_session(endpoint: &str, record: &TokenRecord) -> Result<(), String> {
    let (status, _) = post_form(
        endpoint,
        &[
            ("token", &record.refresh_token),
            ("token_type_hint", "refresh_token"),
            ("client_id", &record.client_id),
        ],
    )?;
    if status.is_success() {
        Ok(())
    } else {
        Err("Revocation failed".into())
    }
}

/// Sign out revokes the renewable session at OpenAI, clears tokens locally, and
/// keeps the registration so a later sign-in reuses the same client.
pub(crate) fn sign_out() -> Result<String, String> {
    let mut message = "Signed out of ChatGPT.".to_string();
    if let Ok(record) = read_record() {
        let endpoint = discovery().ok().and_then(|c| c.revocation_endpoint);
        let mut confirmed = false;
        if let Some(endpoint) = endpoint.as_deref() {
            for _ in 0..2 {
                if revoke_session(endpoint, &record).is_ok() {
                    confirmed = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        }
        if !confirmed {
            message = "Signed out locally. Polish could not confirm the sign-out with OpenAI — you can disconnect Polish in ChatGPT Settings.".into();
        }
    }
    credentials::remove_oauth().map_err(|e| format!("Could not sign out: {e}"))?;
    let mut cfg = load_config();
    cfg.auth = "api-key".into();
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_the_rfc_7636_vector() {
        // RFC 7636 appendix B test vector.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            pkce_challenge(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn percent_round_trips_special_characters() {
        let email = "user@example.com ~-_";
        assert_eq!(percent_encode(email), "user%40example.com%20~-_");
        assert_eq!(percent_decode(&percent_encode(email)), email);
    }

    #[test]
    fn callback_parsing_validates_state_and_errors() {
        let state = "state-123";
        let query = "code=abc123&state=state-123&client_id=oaiapp_1";
        assert_eq!(
            parse_callback(query, state).unwrap(),
            Callback::Success {
                code: "abc123".into(),
                client_id: Some("oaiapp_1".into())
            }
        );
        assert_eq!(
            parse_callback("error=access_denied&state=state-123", state).unwrap(),
            Callback::Declined
        );
        assert!(parse_callback("code=abc123&state=other", state).is_err());
        assert!(parse_callback("state=state-123", state).is_err());
    }

    #[test]
    fn record_round_trips_and_reports_plan_usage() {
        let record = TokenRecord {
            email: "user@example.com".into(),
            subject: "sub-1".into(),
            client_id: "oaiapp_1".into(),
            access_token: "at".into(),
            refresh_token: "rt".into(),
            id_token: "id".into(),
            scopes: vec!["chatgpt.tokens.use.direct".into(), "openid".into()],
            expires_at: 1000,
        };
        let raw = serde_json::to_string(&record).unwrap();
        let loaded: TokenRecord = serde_json::from_str(&raw).unwrap();
        assert!(loaded.plan_usage());
        assert_eq!(loaded.email, record.email);

        let key_only = TokenRecord {
            scopes: vec!["openid".into()],
            ..record
        };
        assert!(!key_only.plan_usage());
    }

    #[test]
    fn catalog_keeps_listed_models_in_server_order() {
        let body = serde_json::json!({
            "models": [
                {"slug": "gpt-plan-a", "display_name": "Plan A", "visibility": "list"},
                {"slug": "gpt-hidden", "display_name": "Hidden", "visibility": "hidden"},
                {"slug": "gpt-plan-b", "display_name": "Plan B", "visibility": "list"}
            ]
        });
        assert_eq!(
            parse_catalog(&body),
            vec![
                ("gpt-plan-a".to_string(), "Plan A".to_string()),
                ("gpt-plan-b".to_string(), "Plan B".to_string()),
            ]
        );
        let fallback = serde_json::json!({"data": [{"id": "legacy"}]});
        assert_eq!(
            parse_catalog(&fallback),
            vec![("legacy".to_string(), "legacy".to_string())]
        );
    }

    #[test]
    fn terminal_refresh_errors_are_recognized() {
        for code in [
            "invalid_grant",
            "invalid_refresh_token",
            "token_expired",
            "refresh_token_expired",
            "refresh_token_invalidated",
            "refresh_token_reused",
        ] {
            assert!(refresh_error_is_terminal(code), "{code}");
        }
        assert!(!refresh_error_is_terminal("temporarily_unavailable"));
        assert!(!refresh_error_is_terminal("invalid_client"));
    }

    #[test]
    fn host_ids_use_an_accepted_uri_format_and_bad_ones_are_replaced() {
        let generated = format!("urn:uuid:{}", uuid_v4());
        assert!(valid_host_id(&generated), "{generated}");
        let hex = generated.trim_start_matches("urn:uuid:").to_string();
        let parts: Vec<&str> = hex.split('-').collect();
        assert_eq!(
            parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
            vec![8, 4, 4, 4, 12]
        );
        assert!(parts[2].starts_with('4'), "UUIDv4 version nibble");
        assert!(parts[3].starts_with(['8', '9', 'a', 'b']), "UUIDv4 variant");

        assert!(valid_host_id(
            "urn:uuid:00000000-000-4000-8000-000000000000"
        ));
        assert!(valid_host_id(
            "urn:ietf:params:oauth:jwk-thumbprint:NzbLsXh8uDCc-"
        ));
        assert!(valid_host_id("did:key:z6MkhaXg"));
        // The bare-random format OpenAI rejected must be treated as invalid.
        assert!(!valid_host_id("Pv_cyjGhv1vxUTwzjZ2oL"));
        assert!(!valid_host_id(""));
    }
}
