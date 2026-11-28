//! Original mb CRM session state. Identifier values are never logged.
use crate::{remote_request::absolute_url, world_http::Response, Error, Result};
use std::sync::{mpsc, Arc, Mutex};
#[derive(Clone, Debug)]
pub struct Request {
    pub url: String,
    result: Arc<Mutex<Option<Arc<Response>>>>,
}
impl Request {
    pub fn pending(url: String) -> Self {
        Self {
            url,
            result: Arc::new(Mutex::new(None)),
        }
    }
    pub fn complete(&self, response: Response) {
        *self.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(response));
    }
    pub fn result(&self) -> Option<Arc<Response>> {
        self.result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}
#[derive(Default)]
pub struct Transport {
    worker: Option<mpsc::SyncSender<Request>>,
    manual: Option<Vec<Request>>,
}
impl Transport {
    pub fn manual() -> Self {
        Self {
            worker: None,
            manual: Some(Vec::new()),
        }
    }
    pub fn queued(&self) -> &[Request] {
        self.manual.as_deref().unwrap_or(&[])
    }
    fn submit(&mut self, url: String) -> Result<Request> {
        let request = Request::pending(url);
        if let Some(queue) = &mut self.manual {
            queue.push(request.clone());
            return Ok(request);
        }
        if self.worker.is_none() {
            let (send, receive) = mpsc::sync_channel::<Request>(1);
            std::thread::Builder::new()
                .name("remote-crm-session".into())
                .spawn(move || {
                    for request in receive {
                        let response =
                            crate::world_http::post_json(&request.url).unwrap_or(Response {
                                status: 400,
                                body: Vec::new(),
                            });
                        request.complete(response);
                    }
                })
                .map_err(|e| Error::Io(e.to_string()))?;
            self.worker = Some(send);
        }
        match self.worker.as_ref().unwrap().try_send(request.clone()) {
            Ok(()) => (),
            Err(mpsc::TrySendError::Full(_)) => request.complete(Response {
                status: 400,
                body: Vec::new(),
            }),
            Err(mpsc::TrySendError::Disconnected(_)) => {
                return Err(Error::Invalid("CRM worker unavailable"))
            }
        }
        Ok(request)
    }
}
pub struct Session {
    pub base: Option<String>,
    pub device: Option<String>,
    pub user_hash: Option<String>,
    pub session_id: String,
    pub status: i32,
    pub pending: Option<Request>,
    pub event_kind: i32,
    pub event_id: Option<String>,
    pub started_millis: i64,
    pub transport: Transport,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            base: Some(String::new()),
            device: Some(String::new()),
            user_hash: Some(String::new()),
            session_id: String::new(),
            status: 1,
            pending: None,
            event_kind: 0,
            event_id: None,
            started_millis: -1,
            transport: Transport::default(),
        }
    }
}
impl Session {
    // None means synchronous URL/IO failure. Other URL handlers throw at the
    // original HttpsURLConnection cast and leave the prior pending request intact.
    fn request(&mut self, text: String) -> Result<Option<Request>> {
        let Some(url) = absolute_url(&text) else {
            return Ok(None);
        };
        if !url.starts_with("https:") {
            return Err(Error::Invalid("CRM requires HTTPS connection"));
        }
        self.transport.submit(url).map(Some)
    }
    pub fn start(
        &mut self,
        base: Option<String>,
        device: Option<String>,
        hash: Option<String>,
        now: i64,
    ) -> Result<()> {
        if self.status == 2 {
            return Ok(());
        }
        self.base = base;
        self.device = device;
        self.user_hash = hash;
        let base = self.base.as_mut().ok_or(Error::Invalid("CRM null base"))?;
        if !base.ends_with('/') {
            base.push('/');
        }
        let device = self
            .device
            .as_ref()
            .ok_or(Error::Invalid("CRM null device"))?;
        if device.is_empty() {
            return Ok(());
        }
        let hash = self
            .user_hash
            .as_ref()
            .ok_or(Error::Invalid("CRM null hash"))?;
        let url = format!("{base}session/open/{device}?userHash={hash}");
        let request = self.request(url)?;
        if let Some(request) = request {
            self.pending = Some(request);
            self.started_millis = now;
        } else {
            self.pending = None;
            self.device = Some(String::new());
            self.user_hash = Some(String::new());
        }
        Ok(())
    }
    pub fn event(&mut self, kind: i32, id: Option<String>) {
        self.event_kind = kind;
        self.event_id = id;
    }
    pub fn poll(&mut self) -> Result<()> {
        let Some(response) = self.pending.as_ref().and_then(Request::result) else {
            return Ok(());
        };
        if response.status == 200 && self.session_id.is_empty() {
            let body = String::from_utf8_lossy(&response.body).replace(['\r', '\n'], "");
            if body.is_empty() {
                return Ok(());
            }
            self.session_id = body;
        }
        if self.event_kind == 0 || self.session_id.is_empty() {
            return Ok(());
        }
        let id = self
            .event_id
            .as_ref()
            .ok_or(Error::Invalid("CRM null event identifier"))?;
        if id.is_empty() {
            return Ok(());
        }
        let event = match self.event_kind {
            1 => "events/click",
            2 => "events/dismissed",
            3 => "events/impression",
            _ => "",
        };
        // Java string concatenation renders null as the literal "null".
        let url = format!(
            "{}{event}/{}/{}/{id}?userHash={}",
            self.base.as_deref().unwrap_or("null"),
            self.device.as_deref().unwrap_or("null"),
            self.session_id,
            self.user_hash.as_deref().unwrap_or("null")
        );
        if let Some(request) = self.request(url)? {
            self.pending = Some(request);
        }
        self.status = 1;
        self.event_kind = 0;
        Ok(())
    }
    pub fn close(&mut self, now: i64) -> Result<()> {
        if self.pending.is_some() || self.session_id.is_empty() || self.status != 1 {
            return Ok(());
        }
        let hash = self
            .user_hash
            .as_ref()
            .ok_or(Error::Invalid("CRM null hash"))?;
        let prefix = if hash.is_empty() {
            String::new()
        } else {
            format!("userHash={hash}&")
        };
        let url = format!(
            "{}session/close/{}/{}?{prefix}sessionDuration={}",
            self.base.as_deref().unwrap_or("null"),
            self.device.as_deref().unwrap_or("null"),
            self.session_id,
            now.wrapping_sub(self.started_millis)
        );
        if let Some(request) = self.request(url)? {
            self.pending = Some(request);
        } else {
            self.status = 1;
        }
        Ok(())
    }
}
