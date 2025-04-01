//! TODO: add module documentation

/// TODO: Add a description of the data channel.
#[mockall::automock]
pub trait WebsocketChannel {
    /// TODO: document
    fn get_stream_url(&self) -> &str;

    /// TODO: document
    fn get_channel_token(&self) -> &str;
}

/// Default [`WebsocketChannel`] implementation.
#[derive(Debug, Default)]
pub struct DefaultWebsocketChannel {
    stream_url: String,
    channel_token: String,
}

impl WebsocketChannel for DefaultWebsocketChannel {
    fn get_stream_url(&self) -> &str {
        &self.stream_url
    }

    fn get_channel_token(&self) -> &str {
        &self.channel_token
    }
}

impl DefaultWebsocketChannel {
    /// Initialize with default settings.
    #[must_use]
    pub fn new(stream_url: String, channel_token: String) -> Self {
        Self {
            stream_url,
            channel_token,
        }
    }
}
