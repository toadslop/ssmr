//! TODO: document

use std::fmt::Debug;

use bitflags::bitflags;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod message_parser;

pub use message_parser::Error;

/// TODO: document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    /// TODO: document
    Output = 1,
    /// TODO: document
    Error = 2,
    /// TODO: document
    Size = 3,
    /// TODO: document
    Parameter = 4,
    /// TODO: document
    HandshakeRequestPayloadType = 5,
    /// TODO: document
    HandshakeResponsePayloadType = 6,
    /// TODO: document
    HandshakeCompletePayloadType = 7,
    /// TODO: document
    EncChallengeRequest = 8,
    /// TODO: document
    EncChallengeResponse = 9,
    /// TODO: document
    Flag = 10,
    /// TODO: document
    StdErr = 11,
    /// TODO: document
    ExitCode = 12,
}

#[derive(Debug, Serialize, Deserialize)]
/// TODO: document
pub enum MessageType {
    /// `InputStreamMessage` represents message type for input data
    #[serde(rename = "input_stream_data")]
    InputStreamMessage,

    /// `OutputStreamMessage` represents message type for output data
    #[serde(rename = "output_stream_data")]
    OutputStreamMessage,

    /// `AcknowledgeMessage` represents message type for acknowledge
    #[serde(rename = "acknowledge")]
    AcknowledgeMessage,

    /// `ChannelClosedMessage` represents message type for `ChannelClosed`
    #[serde(rename = "channel_closed")]
    ChannelClosedMessage,

    /// `StartPublicationMessage` represents the message type that notifies the CLI to start sending stream messages
    #[serde(rename = "start_publication")]
    StartPublicationMessage,

    /// `PausePublicationMessage` represents the message type that notifies the CLI to pause sending stream messages
    /// as the remote data channel is inactive
    #[serde(rename = "pause_publication")]
    PausePublicationMessage,
}

#[derive(Debug)]
/// `ClientMessage` represents a message for client to send/receive. `ClientMessage` Message in MGS is equivalent to MDS' `InstanceMessage`.
/// All client messages are sent in this form to the MGS service.
///
/// * Payload digest is a 32 byte containing the SHA-256 hash of the payload.
/// * Payload length is an 4 byte unsigned integer containing the byte length of data in the Payload field.
/// * Payload is a variable length byte data.
///
/// ## Notes
///
/// The original implementation had a `HeaderLength` field, but I couldn't find any use for it in the code so far so for now I removed it.
#[allow(dead_code)]
pub struct ClientMessage {
    /// `MessageType` is a 32 byte UTF-8 string containing the message type.
    message_type: MessageType,
    /// `SchemaVersion` is a 4 byte integer containing the message schema version number.
    schema_version: u32,
    /// `CreatedDate` is an 8 byte integer containing the message create epoch millis in UTC.
    create_date: DateTime<Utc>,
    /// `SequenceNumber` is an 8 byte integer containing the message sequence number. The sequence number is incremented by 1 for each message sent.
    sequence_number: i64,
    /// Flags is an 8 byte unsigned integer containing a packed array of control flags:
    flags: Flags,
    /// `MessageId` is a 40 byte UTF-8 string containing a random UUID identifying this message
    message_id: uuid::Uuid,
    /// TODO: document
    payload_type: PayloadType,
    payload: Vec<u8>, // TODO: change to use a slice instead of a Vec<u8> to avoid heap allocation
}

impl ClientMessage {
    /// `new` creates a new `ClientMessage` with the given `message_type` and `flags`. The `create_date` is set to the current UTC time.
    #[must_use]
    pub fn new(
        message_type: MessageType,
        flags: Flags,
        payload_type: PayloadType,
        payload: Vec<u8>,
        sequence_number: i64,
    ) -> Self {
        Self {
            message_type,
            schema_version: 1,
            create_date: Utc::now(),
            flags,
            message_id: uuid::Uuid::new_v4(),
            payload_type,
            payload,
            sequence_number,
        }
    }
}

#[allow(dead_code)]
impl ClientMessage {
    const HEADER_LENGTH: u32 = 4;
    const MESSAGE_TYPE_LENGTH: u32 = 32;

    const SCHEMA_VERSION_LENGTH: u32 = 4;
    const CREATED_DATE_LENGTH: u32 = 8;
    const SEQUENCE_NUMBER_LENGTH: u32 = 8;
    const FLAGS_LENGTH: u32 = 8;
    const MESSAGE_ID_LENGTH: u32 = 16;
    const PAYLOAD_DIGEST_LENGTH: u32 = 32;
    const PAYLOAD_TYPE_LENGTH: u32 = 4;
    const PAYLOAD_LENGTH_LENGTH: u32 = 4;

    const HEADER_OFFSET: u32 = 0;
    const MESSAGE_TYPE_OFFSET: u32 = Self::HEADER_OFFSET + Self::HEADER_LENGTH;
    const SCHEMA_VERSION_OFFSET: u32 = Self::MESSAGE_TYPE_OFFSET + Self::MESSAGE_TYPE_LENGTH;
    const CREATED_DATE_OFFSET: u32 = Self::SCHEMA_VERSION_OFFSET + Self::SCHEMA_VERSION_LENGTH;
    const SEQUENCE_NUMBER_OFFSET: u32 = Self::CREATED_DATE_OFFSET + Self::CREATED_DATE_LENGTH;
    const FLAGS_OFFSET: u32 = Self::SEQUENCE_NUMBER_OFFSET + Self::SEQUENCE_NUMBER_LENGTH;
    const MESSAGE_ID_OFFSET: u32 = Self::FLAGS_OFFSET + Self::FLAGS_LENGTH;
    const PAYLOAD_DIGEST_OFFSET: u32 = Self::MESSAGE_ID_OFFSET + Self::MESSAGE_ID_LENGTH;
    const PAYLOAD_TYPE_OFFSET: u32 = Self::PAYLOAD_DIGEST_OFFSET + Self::PAYLOAD_DIGEST_LENGTH;
    const PAYLOAD_LENGTH_OFFSET: u32 = Self::PAYLOAD_TYPE_OFFSET + Self::PAYLOAD_TYPE_LENGTH;
    const PAYLOAD_OFFSET: u32 = Self::PAYLOAD_LENGTH_OFFSET + Self::PAYLOAD_LENGTH_LENGTH;
}

bitflags! {
    /// Flags is an 8 byte unsigned integer containing a packed array of control flags:
    pub struct Flags: u64 {
        /// Bit 0 is SYN - SYN is set (1) when the recipient should consider Seq to be the first message number in the stream
        const SYN = 0b01;
        /// Bit 1 is FIN - FIN is set (1) when this message is the final message in the sequence.
        const FIN = 0b10;
    }
}

impl Debug for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flags")
            .field("SYN", &self.contains(Flags::SYN))
            .field("FIN", &self.contains(Flags::FIN))
            .finish()
    }
}
