use crate::HostError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

const MAX_FIELDS: usize = 32;
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_FIELD_NAME_BYTES: usize = 64;
const MAX_TEXT_BYTES: usize = 4096;
const MAX_SAFE_COUNT: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PayloadScalarKind {
    Text,
    Count,
    Boolean,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadField {
    pub name: String,
    pub kind: PayloadScalarKind,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadSchema {
    pub id: String,
    pub fields: Vec<PayloadField>,
}

impl PayloadSchema {
    pub fn validate(&self) -> Result<(), HostError> {
        if !valid_schema_id(&self.id) || self.fields.is_empty() || self.fields.len() > MAX_FIELDS {
            return Err(invalid("payload schema bounds are invalid"));
        }
        let names: HashSet<&str> = self
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        if names.len() != self.fields.len()
            || self
                .fields
                .iter()
                .any(|field| !valid_field_name(&field.name))
        {
            return Err(invalid("payload field names must be unique identifiers"));
        }
        Ok(())
    }

    pub fn validate_payload(&self, payload: &Value) -> Result<(), HostError> {
        self.validate()?;
        let object = payload
            .as_object()
            .ok_or_else(|| invalid("payload must be an object"))?;
        if object.len() != self.fields.len() {
            return Err(invalid("payload keys do not exactly match its schema"));
        }
        for field in &self.fields {
            let value = object
                .get(&field.name)
                .ok_or_else(|| invalid("payload is missing a declared field"))?;
            if !valid_scalar(value, field.kind) {
                return Err(invalid("payload field has the wrong scalar kind"));
            }
        }
        Ok(())
    }
}

fn valid_schema_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn valid_field_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FIELD_NAME_BYTES
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_scalar(value: &Value, kind: PayloadScalarKind) -> bool {
    match kind {
        PayloadScalarKind::Text => value
            .as_str()
            .is_some_and(|text| !text.is_empty() && text.len() <= MAX_TEXT_BYTES),
        PayloadScalarKind::Count => value.as_u64().is_some_and(|count| count <= MAX_SAFE_COUNT),
        PayloadScalarKind::Boolean => value.is_boolean(),
    }
}

fn invalid(message: &str) -> HostError {
    HostError::Configuration(message.to_string())
}
