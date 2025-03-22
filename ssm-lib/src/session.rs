//! Types and functions for managing ssm sessions. This is the high-level API for the session manager.
//! Most users will want to use the functions in this module.
//!
//! Roughly corresponds to the code in [this folder](https://github.com/aws/session-manager-plugin/tree/mainline/src/sessionmanagerplugin/session),
//! although input validation logic has been extracted to the main session-manager-plugin crate.

use std::collections::HashMap;
use uuid::Uuid;

/// A session represents a connection to a target.
#[allow(dead_code)] // TODO: remove
#[derive(Debug)]
pub struct Session {
    // data_channel: DataChannel, TODO: Implement this
    session_id: Uuid,
    stream_url: String,
    token_value: String,
    is_aws_cli_upgrade_needed: bool,
    endpoint: String,
    client_id: String,
    target_id: String,
    // sdk: SSM TODO: Implement this
    // retry_params: RepeatableExponentialRetryer TODO: Implement this
    session_type: String,
    session_properties: HashMap<String, String>,
    // display_mode: DisplayMode, TODO: Implement this
}

/// A builder for creating a [Session].
#[derive(Debug, Default)]
pub struct SessionBuilder {
    stream_url: String,
    token_value: String,
    is_aws_cli_upgrade_needed: bool,
    endpoint: String,
    client_id: String,
    target_id: String,
    session_type: String,
    session_properties: HashMap<String, String>,
}

impl SessionBuilder {
    /// Initialize the builder with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the websockets stream url. This value should be output from the call to the `StartSession` API.
    #[must_use]
    pub fn with_stream_url(mut self, stream_url: String) -> Self {
        self.stream_url = stream_url;
        self
    }

    /// Convert the builder into a [Session].
    #[must_use]
    pub fn build(self) -> Session {
        Session {
            session_id: Uuid::new_v4(),
            stream_url: self.stream_url,
            token_value: self.token_value,
            is_aws_cli_upgrade_needed: self.is_aws_cli_upgrade_needed,
            endpoint: self.endpoint,
            client_id: self.client_id,
            target_id: self.target_id,
            session_type: self.session_type,
            session_properties: self.session_properties,
        }
    }
}
