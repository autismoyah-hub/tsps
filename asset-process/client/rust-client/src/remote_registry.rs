//! The version2 registry reached by mg.ml -> my.ag. Initial1130 loaders retain
//! a null location, so no non-null JSON location can select a card for them.
use crate::{remote_json::Value, remote_request::Requester, Result};
use std::collections::BTreeMap;
pub type Object = crate::remote_json::Object;
pub fn get<'a>(object: &'a Object, key: &str) -> Option<&'a Value> {
    object.get(&key.encode_utf16().collect::<Vec<_>>())
}
pub fn object(value: &Value) -> Option<&Object> {
    if let Value::Object(value) = value {
        Some(value)
    } else {
        None
    }
}
pub fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(v) => String::from_utf16(v).ok(),
        Value::Null => Some("null".into()),
        Value::Bool(v) => Some(v.to_string()),
        Value::Integer(v) => Some(v.to_string()),
        _ => None,
    }
}
fn boolean(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(v) => Some(*v),
        Value::String(v) => {
            let s = String::from_utf16(v).ok()?;
            if s.eq_ignore_ascii_case("true") {
                Some(true)
            } else if s.eq_ignore_ascii_case("false") {
                Some(false)
            } else {
                None
            }
        }
        _ => None,
    }
}
#[derive(Default)]
pub struct Registry {
    pub cards: BTreeMap<String, Object>,
    pub parsed: BTreeMap<String, std::sync::Arc<crate::remote_card::Card>>,
}
impl Registry {
    pub fn load(&mut self, root: &Object, requester: Option<&mut Requester>) -> Result<()> {
        self.cards.clear();
        self.parsed.clear();
        // getLong accepts exactly these numeric categories; only success is used.
        if !["lastfullsyncat", "lastcardupdatedat"]
            .iter()
            .all(|key| get(root, key).and_then(Value::int).is_some())
        {
            return Ok(());
        }
        let Some(Value::Array(cards)) = get(root, "crmcomponents") else {
            return Ok(());
        };
        let mut requester = requester;
        for card in cards {
            // my.ag catches the entire loop: malformed entries preserve prior
            // registry entries and requests, and prevent processing later cards.
            let Some(card) = object(card) else { break };
            let Some(game) = get(card, "game").and_then(text) else {
                break;
            };
            if game != "osrs" {
                continue;
            }
            let Some(removed) = get(card, "removed").and_then(boolean) else {
                break;
            };
            if removed {
                continue;
            }
            let Some(platform) = get(card, "platform").and_then(text) else {
                break;
            };
            if matches!(platform.as_str(), "mobile" | "android" | "ios") {
                continue;
            }
            let parsed = crate::remote_card::Card::parse(card, requester.as_deref_mut())?;
            let Some(endpoint) = &parsed.endpoint else {
                break;
            };
            if endpoint.is_empty() {
                continue;
            }
            let Some(location) = get(card, "location").and_then(text) else {
                break;
            };
            self.cards.insert(location.clone(), card.clone());
            self.parsed.insert(location, std::sync::Arc::new(parsed));
        }
        Ok(())
    }
}
