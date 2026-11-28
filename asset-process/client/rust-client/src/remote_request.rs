//! Remote-widget HTTP requests share the native HTTP transport and use one FIFO worker.
use crate::{Error, Result};
use std::sync::{mpsc, Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Request {
    pub url: String,
    result: Arc<Mutex<Option<Option<Arc<Vec<u8>>>>>>,
}
impl Request {
    pub fn same_request(&self,other:&Self)->bool {Arc::ptr_eq(&self.result,&other.result)}
    pub fn pending(url: String) -> Self {
        Self {
            url,
            result: Arc::new(Mutex::new(None)),
        }
    }
    pub fn complete(&self, bytes: Option<Vec<u8>>) {
        *self.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(bytes.map(Arc::new));
    }
    pub fn result(&self) -> Option<Option<Arc<Vec<u8>>>> {
        self.result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

#[derive(Default)]
pub struct Requester {
    worker: Option<mpsc::Sender<Request>>,
    manual: Option<Vec<Request>>,
}
impl Requester {
    /// Deterministic transport: callers complete the same shared request handles.
    pub fn manual() -> Self {
        Self {
            worker: None,
            manual: Some(Vec::new()),
        }
    }
    pub fn queued(&self) -> &[Request] {
        self.manual.as_deref().unwrap_or(&[])
    }
    pub fn request(&mut self, url: String) -> Result<Request> {
        let request = Request::pending(url);
        if let Some(queue) = &mut self.manual {
            queue.push(request.clone());
            return Ok(request);
        }
        if self.worker.is_none() {
            let (send, receive) = mpsc::channel::<Request>();
            std::thread::Builder::new()
                .name("remote-widget-http".into())
                .spawn(move || {
                    for request in receive {
                        let bytes = if request.url.starts_with("http:")
                            || request.url.starts_with("https:")
                        {
                            crate::world_http::fetch(&request.url).ok()
                        } else {
                            None
                        };
                        request.complete(bytes);
                    }
                })
                .map_err(|e| Error::Io(e.to_string()))?;
            self.worker = Some(send);
        }
        self.worker
            .as_ref()
            .unwrap()
            .send(request.clone())
            .map_err(|_| Error::Invalid("remote request worker unavailable"))?;
        Ok(request)
    }
}

/// The absolute URL forms accepted by the original requester. Dot segments in
/// an absolute path are retained; URL validity is distinct from HTTP support.
pub fn absolute_url(text: &str) -> Option<String> {
    let mut text = text.trim_matches(|c: char| c <= ' ');
    if text
        .get(..4)
        .is_some_and(|v| v.eq_ignore_ascii_case("url:"))
    {
        text = &text[4..];
    }
    let (scheme, rest) = text.split_once(':')?;
    let scheme = scheme.to_ascii_lowercase();
    if !matches!(
        scheme.as_str(),
        "http" | "https" | "ftp" | "file" | "jar" | "mailto" | "jrt"
    ) {
        return None;
    }
    if scheme == "jar" {
        let (inner, _) = rest.split_once("!/")?;
        if inner.starts_with("jar:") || absolute_url(inner).is_none() {
            return None;
        }
    }
    if scheme == "mailto" && rest.split('#').next().unwrap_or("").trim().is_empty() {
        return None;
    }
    let mut rest = rest;
    if matches!(scheme.as_str(), "http" | "https" | "ftp" | "file") {
        if let Some(authority_and_path) = rest
            .strip_prefix("//")
            .filter(|_| !rest.starts_with("////"))
        {
            let end = authority_and_path
                .find(['/', '?', '#'])
                .unwrap_or(authority_and_path.len());
            let authority = &authority_and_path[..end];
            // URLStreamHandler treats multiple @ signs as an opaque authority.
            if authority.matches('@').count() > 1 {
                return Some(format!("{scheme}:{rest}"));
            }
            let host = authority
                .rsplit_once('@')
                .map_or(authority, |(_, host)| host);
            let port = if host.starts_with('[') {
                let end = host.find(']')?;
                let address = &host[1..end];
                let literal = if let Some((literal, scope)) = address.split_once('%') {
                    if scope.is_empty()
                        || scope
                            .chars()
                            .any(|c| c < ' ' || c == '\u{7f}' || "[]/\\#?".contains(c))
                    {
                        return None;
                    }
                    literal
                } else {
                    address
                };
                literal.parse::<std::net::Ipv6Addr>().ok()?;
                let tail = &host[end + 1..];
                if tail.is_empty() {
                    None
                } else {
                    Some(tail.strip_prefix(':')?)
                }
            } else {
                let name = host.split(':').next().unwrap_or("");
                if name.chars().any(|c| {
                    c <= ' '
                        || c == '\u{7f}'
                        || ":/?#[]@\\<>\"{}|^`".contains(c)
                        || [
                            8263, 8264, 8265, 8448, 8449, 8453, 8454, 10868, 65109, 65110, 65119,
                            65131, 65283, 65295, 65306, 65311, 65312,
                        ]
                        .contains(&(c as u32))
                }) {
                    return None;
                }
                host.split_once(':').map(|(_, port)| port)
            };
            if let Some(port) = port.filter(|p| !p.is_empty()) {
                if crate::script_string::parse_java_int(port)? < -1 {
                    return None;
                }
            }
            if authority.is_empty() {
                rest = &rest[2..];
            }
        }
    }
    Some(format!("{scheme}:{rest}"))
}
