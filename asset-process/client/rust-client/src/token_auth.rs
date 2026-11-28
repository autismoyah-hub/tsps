//! Revision239 token acquisition before game-socket login (GameClient.ij/il/iu/iz).
//! Credentials and requests intentionally do not implement Debug.
use crate::{
    remote_json::{self, Value},
    world_http::{RestRequest, RestResponse},
    Error, Result,
};

pub struct Config {
    pub auth_base: String,
    pub session_base: String,
    pub basic_authorization: String,
    pub access_token: String,
    pub refresh_token: String,
    pub session_id: String,
    pub account_id: String,
}
impl Config {
    pub fn from_json(text: &str) -> Result<Self> {
        let object =
            remote_json::content(text).map_err(|_| Error::Invalid("token configuration JSON"))?;
        let get = |name: &str| -> Result<String> {
            match object.get(&name.encode_utf16().collect::<Vec<_>>()) {
                None => Ok(String::new()),
                Some(Value::String(v)) => String::from_utf16(v)
                    .map_err(|_| Error::Invalid("token configuration encoding")),
                _ => Err(Error::Invalid("token configuration field type")),
            }
        };
        Ok(Self {
            auth_base: get("auth_base")?,
            session_base: get("session_base")?,
            basic_authorization: get("basic_authorization")?,
            access_token: get("access_token")?,
            refresh_token: get("refresh_token")?,
            session_id: get("session_id")?,
            account_id: get("account_id")?,
        })
    }
}
fn present(value: &str) -> bool {
    !value.trim_matches(|c: char| c <= ' ').is_empty()
}
/// java.net.URLEncoder's UTF-8 form encoding, including '?' for unpaired UTF-16.
pub fn form(value: &[u16]) -> String {
    let mut out = String::new();
    for c in char::decode_utf16(value.iter().copied()) {
        let c = c.unwrap_or('?');
        let mut bytes = [0; 4];
        for b in c.encode_utf8(&mut bytes).bytes() {
            match b {
                b' ' => out.push('+'),
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'*' | b'_' => {
                    out.push(b as char)
                }
                _ => {
                    out.push('%');
                    out.push(b"0123456789ABCDEF"[(b >> 4) as usize] as char);
                    out.push(b"0123456789ABCDEF"[(b & 15) as usize] as char);
                }
            }
        }
    }
    out
}
fn authority_host(base: &str) -> Result<String> {
    let (_, tail) = base
        .split_once("://")
        .ok_or(Error::Invalid("token endpoint URL"))?;
    let authority = tail.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.contains('@') {
        return Err(Error::Invalid("token endpoint host"));
    }
    if authority.starts_with('[') {
        let end = authority
            .find(']')
            .ok_or(Error::Invalid("token endpoint IPv6"))?;
        Ok(authority[..=end].to_owned())
    } else {
        Ok(authority.split(':').next().unwrap().to_owned())
    }
}
pub fn refresh_request(base: &str, basic: &str, token: &[u16]) -> Result<(String, RestRequest)> {
    Ok((format!("{base}shield/oauth/token?refresh_token={}&grant_type=refresh_token&scope=gamesso.token.create",form(token)),
        RestRequest { method:"POST", headers:vec![("Authorization".into(),format!("Basic {basic}")),
            ("Host".into(),authority_host(base)?),("Accept".into(),"application/json".into())],body:vec![] }))
}
pub fn play_request(base: &str, access: &str) -> (String, RestRequest) {
    (
        format!("{base}public/v1/games/YCfdbvr2pM1zUYMxJRexZY/play"),
        RestRequest {
            method: "GET",
            headers: vec![("Authorization".into(), format!("Bearer {access}"))],
            body: vec![],
        },
    )
}
fn json_quote(text: &str) -> String {
    let mut out = String::from("\"");
    let mut previous = '\0';
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '/' if previous == '<' => out.push_str("\\/"),
            c if c < ' '
                || ('\u{80}'..'\u{a0}').contains(&c)
                || ('\u{2000}'..'\u{2100}').contains(&c) =>
            {
                out.push_str(&format!("\\u{:04x}", c as u32))
            }
            c => out.push(c),
        }
        previous = c;
    }
    out.push('"');
    out
}
pub fn session_request(base: &str, session: &str, account: &str) -> (String, RestRequest) {
    (
        format!("{base}game-session/v1/tokens"),
        RestRequest {
            method: "POST",
            headers: vec![
                ("Authorization".into(), format!("Bearer {session}")),
                ("Accept".into(), "application/json".into()),
                ("Content-Type".into(), "application/json".into()),
            ],
            body: format!("{{\"accountId\":{}}}", json_quote(account)).into_bytes(),
        },
    )
}
fn body(response: &RestResponse) -> Result<String> {
    if response.status != 200 {
        return Err(Error::Invalid("token HTTP response status"));
    }
    // qz joins BufferedReader.readLine results without separators.
    Ok(String::from_utf8_lossy(&response.body)
        .chars()
        .filter(|&c| c != '\r' && c != '\n')
        .collect())
}
fn field(object: &remote_json::Object, name: &str) -> Result<String> {
    match object.get(&name.encode_utf16().collect::<Vec<_>>()) {
        Some(Value::String(v)) => {
            String::from_utf16(v).map_err(|_| Error::Invalid("token response encoding"))
        }
        Some(Value::Integer(v)) => Ok(v.to_string()),
        Some(Value::Bool(v)) => Ok(v.to_string()),
        Some(Value::Null) => Ok("null".into()),
        _ => Err(Error::Invalid("token response field")),
    }
}
pub fn parse_play(response: &RestResponse) -> Result<String> {
    let text = body(response)?;
    if response
        .headers
        .iter()
        .any(|(name, value)| name == "Content-Type" && value == "application/json")
    {
        let object =
            remote_json::content(&text).map_err(|_| Error::Invalid("play response JSON"))?;
        field(&object, "token")
    } else {
        Ok(text)
    }
}
/// Transport injection keeps request construction, rotation and ordering testable.
pub fn acquire_with(
    config: &mut Config,
    mut send: impl FnMut(&str, &RestRequest) -> Result<RestResponse>,
) -> Result<String> {
    let response = if present(&config.access_token) && present(&config.refresh_token) {
        let (url, request) = refresh_request(
            &config.auth_base,
            &config.basic_authorization,
            &config.refresh_token.encode_utf16().collect::<Vec<_>>(),
        )?;
        let response = send(&url, &request)?;
        let object = remote_json::content(&body(&response)?)
            .map_err(|_| Error::Invalid("refresh response JSON"))?;
        // Original assignments are sequential; preserve a rotated access token
        // even if reading refresh_token or the subsequent play request fails.
        config.access_token = field(&object, "access_token")?;
        config.refresh_token = field(&object, "refresh_token")?;
        let (url, request) = play_request(&config.auth_base, &config.access_token);
        send(&url, &request)?
    } else if present(&config.session_id) && present(&config.account_id) {
        let (url, request) =
            session_request(&config.session_base, &config.session_id, &config.account_id);
        send(&url, &request)?
    } else {
        return Err(Error::Invalid("token credentials unavailable"));
    };
    parse_play(&response)
}
pub fn acquire(mut config: Config) -> (Config, Result<String>) {
    let result = acquire_with(&mut config, |url, request| {
        // Original qu requires HTTPS. Literal loopback HTTP is confined to the
        // native transport acceptance harness and custom local test services.
        let scheme = url.split_once("://").map_or("", |(scheme, _)| scheme);
        if !scheme.eq_ignore_ascii_case("https")
            && !(scheme.eq_ignore_ascii_case("http")
                && matches!(authority_host(url)?.as_str(), "127.0.0.1" | "[::1]"))
        {
            return Err(Error::Invalid("token endpoint requires HTTPS"));
        }
        crate::world_http::rest(url, request)
    });
    (config, result)
}
