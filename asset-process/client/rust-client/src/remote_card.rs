//! Parsed version2 remote cards (mc.fc), retaining shared image requests.
use crate::{
    remote_json::Value,
    remote_registry::{get, object, text, Object},
    remote_request::{absolute_url, Request, Requester},
    Result,
};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Clone, Debug)]
pub struct Image {
    pub request: Request,
    pub placement: [f32; 4],
}
#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    pub alignment: [u8; 2],
    pub font: i32,
    pub placement: [f32; 4],
}
#[derive(Clone, Debug, PartialEq)]
pub enum ParameterValue {
    Int(i32),
    Text(String),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: ParameterValue,
}
#[derive(Clone, Debug)]
pub struct Card {
    pub raw: Arc<Object>,
    pub images: Vec<Image>,
    pub labels: Vec<Label>,
    pub endpoint: Option<String>,
    pub click_bounds: [f32; 4],
    pub id: Option<String>,
    pub priority: i32,
    pub parameters: Vec<Parameter>,
    pub meta: BTreeMap<Vec<u16>, i32>,
    pub meta_strings: BTreeMap<Vec<u16>, String>,
}
pub fn placement(value: &Object, key: &str) -> [f32; 4] {
    let defaults = [0., 0., 1., 1.];
    let Some(Value::Array(values)) = get(value, key) else {
        return defaults;
    };
    std::array::from_fn(|i| {
        values
            .get(i)
            .and_then(Value::number)
            .map_or(defaults[i], |v| v as f32)
    })
}
fn alignment(text: &str) -> u8 {
    match text {
        "centre" => 1,
        "bottom" | "right" => 2,
        _ => 0,
    }
}
impl Card {
    pub fn parse(raw: &Object, requester: Option<&mut Requester>) -> Result<Self> {
        let mut card = Self {
            raw: Arc::new(raw.clone()),
            images: Vec::new(),
            labels: Vec::new(),
            endpoint: None,
            click_bounds: [0., 0., 1., 1.],
            id: None,
            priority: 0,
            parameters: Vec::new(),
            meta: BTreeMap::new(),
            meta_strings: BTreeMap::new(),
        };
        if let Some(image) = get(raw, "image").and_then(object) {
            if let Some(url) = get(image, "src")
                .and_then(text)
                .and_then(|v| absolute_url(&v))
            {
                if let Some(requester) = requester {
                    card.images.push(Image {
                        request: requester.request(url)?,
                        placement: placement(image, "placement"),
                    });
                }
            }
        }
        if let Some(label) = get(raw, "label").and_then(object) {
            card.labels.push(Label {
                text: get(label, "text").and_then(text).unwrap_or_default(),
                alignment: [
                    alignment(&get(label, "align_x").and_then(text).unwrap_or_default()),
                    alignment(&get(label, "align_y").and_then(text).unwrap_or_default()),
                ],
                font: get(label, "font").and_then(Value::int).unwrap_or(0),
                placement: placement(label, "placement"),
            });
        }
        if let Some(behaviour) = get(raw, "behaviour").and_then(object) {
            if let Some(endpoint) = get(behaviour, "endpoint").and_then(text) {
                card.endpoint = Some(endpoint);
                card.click_bounds = placement(behaviour, "clickbounds");
                for key in behaviour.java_keys() {
                    let Ok(name) = String::from_utf16(key) else {
                        continue;
                    };
                    if matches!(name.as_str(), "clickbounds" | "endpoint") {
                        continue;
                    }
                    let raw = &behaviour[key];
                    let value = if let Some(value) = raw.int() {
                        Some(ParameterValue::Int(value))
                    } else {
                        text(raw).map(|value| match value.as_str() {
                            "true" => ParameterValue::Int(1),
                            "false" => ParameterValue::Int(0),
                            _ => ParameterValue::Text(value),
                        })
                    };
                    if let Some(value) = value {
                        card.parameters.push(Parameter { name, value });
                    }
                }
            }
        }
        if let Some(meta) = get(raw, "meta").and_then(object) {
            for (key, value) in meta {
                card.meta.insert(key.clone(), value.int().unwrap_or(0));
            }
        }
        card.id = get(raw, "id").and_then(text);
        if card.id.is_some() {
            card.priority = get(raw, "priority")
                .and_then(text)
                .and_then(|v| crate::script_string::parse_java_int(&v))
                .unwrap_or(0);
        }
        Ok(card)
    }
}
