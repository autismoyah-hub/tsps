//! State retained by the original inline and URL remote-widget loaders.
use crate::{
    remote_json::{self, Value},
    Error, Result,
};

#[derive(Clone, Debug)]
pub struct Loader {
    pub card: Option<std::sync::Arc<crate::remote_card::Card>>,
    pub raw_url: Option<String>,
    pub location: Option<String>,
    pub crm_base: Option<String>,
    pub crm_attached: bool,
    pub request: Option<crate::remote_request::Request>,
    pub status: i32,
    pub version: i32,
    pub content: Option<std::sync::Arc<remote_json::Object>>,
}
impl Default for Loader {
    fn default() -> Self {
        Self {
            card: None,
            raw_url: None,
            location: None,
            crm_base: None,
            crm_attached: false,
            request: None,
            status: -1,
            version: 0,
            content: None,
        }
    }
}
impl Loader {
    /// Poll URL responses and selected-card images. Initial1130 has no card
    /// location;1151 can select a version2 registry card.
    pub fn poll_initial(&mut self, context: &mut crate::remote_url::Context) -> Result<()> {
        if self.status == 1 {
            let card = self
                .card
                .as_ref()
                .ok_or(Error::Invalid("remote card absent"))?;
            if card
                .images
                .iter()
                .any(|image| image.request.result().is_none())
            {
                return Ok(());
            }
            self.status = if card.images.iter().any(|image| {
                image
                    .request
                    .result()
                    .flatten()
                    .is_some_and(|bytes| !bytes.is_empty())
            }) {
                2
            } else {
                5
            };
            self.request = None;
            return Ok(());
        }
        if self.status != 0 {
            return Ok(());
        }
        let Some(result) = self.request.as_ref().and_then(|request| request.result()) else {
            return Ok(());
        };
        let Some(bytes) = result else {
            self.request = None;
            self.status = 4;
            return Ok(());
        };
        let text = String::from_utf8_lossy(&bytes);
        let object = match remote_json::content(&text) {
            Ok(value) => value,
            Err(remote_json::Failure::Syntax) => {
                self.request = None;
                self.status = 6;
                return Ok(());
            }
            Err(remote_json::Failure::Runtime) => {
                return Err(Error::Invalid("remote response parse exception"))
            }
        };
        let Some(version) = crate::remote_registry::get(&object, "version").and_then(Value::int)
        else {
            self.request = None;
            self.status = 6;
            return Ok(());
        };
        self.version = version;
        if version < 2 {
            return Err(Error::Invalid("remote response content absent"));
        }
        if version == 2 {
            context.registry.load(&object, context.requester.as_mut())?;
            self.card = self
                .location
                .as_ref()
                .and_then(|key| context.registry.parsed.get(key))
                .cloned();
            self.content = self.card.as_ref().map(|card| card.raw.clone());
            self.status = self
                .card
                .as_ref()
                .map_or(7, |card| if card.images.is_empty() { 2 } else { 1 });
        }
        self.request = None;
        Ok(())
    }

    pub fn inline(&mut self, text: &str) -> Result<()> {
        let object = match remote_json::content(text) {
            Ok(v) => v,
            Err(remote_json::Failure::Syntax) => {
                self.status = 6;
                return Ok(());
            }
            Err(remote_json::Failure::Runtime) => {
                return Err(Error::Invalid("remote inline content parse exception"))
            }
        };
        let key: Vec<_> = "version".encode_utf16().collect();
        let Some(version) = object.get(&key).and_then(Value::int) else {
            self.status = 6;
            return Ok(());
        };
        self.version = version;
        // bo always installs a new mg with av=null. The pinned mc.fc dereferences
        // it before parsing content, leaving the version and initial status intact.
        Err(Error::Invalid("remote inline content absent"))
    }
}
