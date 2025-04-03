//! Error type for the SSM library.

/// Error type for the SSM library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An attempt to reconnect to a channel failed.
    #[error("failed to reconnect data channel {stream_url} with error: {source}")]
    Reconnect {
        /// The underlying error
        #[source]
        source: Box<Self>,
        /// The url of the stream that the reconnect was attempted for
        stream_url: String,
    },

    /// A data channel could not be opened
    #[error("failed to open data channel with error: {0}")]
    DataChannelOpen(#[source] Box<Self>),

    /// TODO: document
    #[error("error sending token for handshake: {0}")]
    FinalizeHandshake(#[source] Box<Self>),
}
