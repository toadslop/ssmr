use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct OpenDataChannelInput {
    pub message_schema_version: String,
    pub request_id: Uuid,
    pub token_value: String,
    pub client_id: String,
    pub client_version: String,
}

impl OpenDataChannelInput {
    pub fn new(token_value: String, client_id: String) -> Self {
        Self {
            message_schema_version: config::MESSAGE_SCHEMA_VERSION.to_string(),
            request_id: Uuid::new_v4(),
            token_value,
            client_id,
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
