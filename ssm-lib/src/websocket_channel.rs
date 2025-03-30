//! TODO: add module documentation

/// TODO: Add a description of the data channel.
#[mockall::automock]
pub trait WebsocketChannel {}

/// Default [`WebsocketChannel`] implementation.
#[derive(Debug, Default)]
pub struct DefaultWebsocketChannel {}

impl WebsocketChannel for DefaultWebsocketChannel {}

impl DefaultWebsocketChannel {
    /// Initialize with default settings.
    #[must_use]
    pub fn new() -> Self {
        DefaultWebsocketChannel::default()
    }
}
