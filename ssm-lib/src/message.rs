//! TODO: document

use bitflags::bitflags;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

mod message_parser;

use sha2::{Digest, Sha256};

/// TODO: document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PayloadType {
    /// TODO: document
    #[default]
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

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::Display,
    strum::IntoStaticStr,
)]
/// TODO: document
pub enum MessageType {
    /// `InputStreamMessage` represents message type for input data
    #[serde(rename = "input_stream_data")]
    #[strum(to_string = "input_stream_data")]
    #[default]
    InputStreamMessage,

    /// `OutputStreamMessage` represents message type for output data
    #[serde(rename = "output_stream_data")]
    #[strum(to_string = "output_stream_data")]
    OutputStreamMessage,

    /// `AcknowledgeMessage` represents message type for acknowledge
    #[serde(rename = "acknowledge")]
    #[strum(to_string = "acknowledge")]
    AcknowledgeMessage,

    /// `ChannelClosedMessage` represents message type for `ChannelClosed`
    #[serde(rename = "channel_closed")]
    #[strum(to_string = "channel_closed")]
    ChannelClosedMessage,

    /// `StartPublicationMessage` represents the message type that notifies the CLI to start sending stream messages
    #[serde(rename = "start_publication")]
    #[strum(to_string = "start_publication")]
    StartPublicationMessage,

    /// `PausePublicationMessage` represents the message type that notifies the CLI to pause sending stream messages
    /// as the remote data channel is inactive
    #[serde(rename = "pause_publication")]
    #[strum(to_string = "pause_publication")]
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
#[derive(Default)]
pub struct ClientMessage {
    /// `HeaderLength` is a 4 byte integer that represents the header length.
    header_length: u32,
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
    /// Payload digest is a 32 byte containing the SHA-256 hash of the payload.
    payload_digest: Vec<u8>, // TODO: document
    /// TODO: document
    payload_type: PayloadType,
    /// Payload length is an 4 byte unsigned integer containing the byte length of data in the Payload field.
    payload_length: u32,
    /// Payload is a variable length byte data.
    payload: Vec<u8>, // TODO: change to use a slice instead of a Vec<u8> to avoid heap allocation
}

impl ClientMessage {
    /// `new` creates a new `ClientMessage` with the given `message_type` and `flags`. The `create_date` is set to the current UTC time.
    ///
    /// ## Errors
    ///
    /// The payload length must be less than 2^32 - 1. If the payload length is greater than this, an error will be returned.
    pub fn new(
        message_type: MessageType,
        flags: Flags,
        payload_type: PayloadType,
        payload: Vec<u8>,
        sequence_number: i64,
    ) -> Result<Self, Error> {
        // Create a Sha256 object
        let mut hasher = Sha256::new();

        // Write input message
        hasher.update(&payload);

        // Read hash digest and consume hasher
        let payload_digest = hasher.finalize().to_vec();
        let message = Self {
            header_length: Self::HEADER_LENGTH,
            message_type,
            schema_version: 1,
            create_date: Utc::now(),
            flags,
            message_id: uuid::Uuid::new_v4(),
            payload_type,
            payload_length: u32::try_from(payload.len())?, // TODO: handle error
            payload,
            sequence_number,
            payload_digest,
        };

        Ok(message)
    }

    /// Confirm whether the message is valid or not. This matches the original implementation,
    /// but we intend to remove this in favor or validataing the message in the parser.
    ///
    /// ## Errors
    ///
    /// TODO
    pub fn validate(&self) -> Result<(), Error> {
        if matches!(
            self.message_type,
            MessageType::StartPublicationMessage | MessageType::PausePublicationMessage
        ) {
            return Ok(());
        }

        if self.header_length == 0 {
            Err(Error::ZeroLengthHeader)?;
        }

        // Original implementation checked if message_type is missing, but we validate it in the parser so we skip checking here
        // Original implementation also checked create_date, but we also validate it in the parser

        if self.payload_length != 0 {
            let mut hasher = Sha256::new();
            hasher.update(self.payload.as_slice());
            let hash = hasher.finalize();

            if hash.to_vec() != self.payload_digest {
                Err(Error::InvalidPayloadHash)?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn deserialize_data_stream_acknowledge_content(&self) -> Result<AcknowledgeContent, Error> {
        if self.message_type != MessageType::AcknowledgeMessage {
            Err(Error::InvalidMessageType {
                expected: MessageType::AcknowledgeMessage,
                actual: self.message_type,
            })?;
        }

        let message: AcknowledgeContent = serde_json::from_slice(self.payload.as_slice())?;

        Ok(message)
    }
}

#[derive(Debug, thiserror::Error)]
/// `Error` represents the error type for `ClientMessage`
pub enum Error {
    /// TODO
    #[error("HeaderLength cannot be zero")]
    ZeroLengthHeader,

    /// TODO
    #[error("payload Hash is not valid")]
    InvalidPayloadHash,

    /// TODO
    #[error("Failed to parse message: {0}")]
    ParseError(#[from] message_parser::Error),

    /// TODO
    #[error("Failed to convert payload length to 32-bit integer: {0}")]
    InvalidPayloadLength(#[from] std::num::TryFromIntError),

    /// TODO
    #[error("Could not deserialize rawMessage: {0}")]
    DeserializeError(#[from] serde_json::Error),

    /// TODO
    #[error("ClientMessage is not of type {expected}. Found message type: {actual}")]
    InvalidMessageType {
        /// The expected message type
        expected: MessageType,
        /// The actual message type
        actual: MessageType,
    },
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

impl Default for Flags {
    fn default() -> Self {
        Flags::empty()
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

/// `AcknowledgeContent` is used to inform the sender of an acknowledge message that the message has been received.
#[derive(Debug, Serialize, Deserialize)]
pub struct AcknowledgeContent {
    /// A 32 byte UTF-8 string containing the message type.
    #[serde(rename = "AcknowledgedMessageType")]
    message_type: MessageType,

    /// a 40 byte UTF-8 string containing the UUID identifying this message being acknowledged.
    #[serde(rename = "AcknowledgedMessageId")]
    message_id: Uuid,

    /// an 8 byte integer containing the message sequence number for serialized message.
    #[serde(rename = "AcknowledgedMessageSequenceNumber", default)]
    sequence_number: i64,

    /// a boolean field representing whether the acknowledged message is part of a sequence
    #[serde(rename = "IsSequentialMessage", default)]
    is_sequential_message: bool,
}

#[cfg(test)]
mod test {
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    use super::ClientMessage;

    const MESSAGE_ID_RAW: &str = "dd01e56b-ff48-483e-a508-b5f073f31b16";
    static MESSAGE_ID: std::sync::LazyLock<Uuid> = std::sync::LazyLock::new(|| {
        Uuid::parse_str(MESSAGE_ID_RAW).expect("message id should be valid uuid")
    });
    const SCHEMA_VERSION: u32 = 1;
    const PAYLOAD: &[u8; 7] = b"payload";

    static ACK_MESSAGE_PAYLOAD: std::sync::LazyLock<Vec<u8>> = std::sync::LazyLock::new(|| {
        format!(
            r#"
        {{
            "AcknowledgedMessageType": "{}",
            "AcknowledgedMessageId": "{MESSAGE_ID_RAW}"
        }}
        "#,
            super::MessageType::AcknowledgeMessage
        )
        .as_bytes()
        .to_vec()
    });

    // TODO: use typestate to make invalid messages impossible to create
    #[test]
    fn test_client_message_validate() {
        let mut message = super::ClientMessage {
            schema_version: SCHEMA_VERSION,
            sequence_number: 1,
            flags: super::Flags::FIN,
            message_id: *MESSAGE_ID,
            payload_length: 3,
            payload: PAYLOAD.to_vec(),
            ..Default::default()
        };

        let result = message.validate().expect_err("message should be invalid");
        assert!(matches!(result, super::Error::ZeroLengthHeader));

        // Note: skipping message type check because it will be handled in the parser

        // Note: skipping create date check because it will be handled in the parser

        message.header_length = 1;

        let result = message.validate().expect_err("message should be invalid");
        assert!(matches!(result, super::Error::InvalidPayloadHash));

        let mut hasher = Sha256::new();
        hasher.update(PAYLOAD.as_slice());
        let hash = hasher.finalize();

        message.payload_digest = hash.to_vec();

        message.validate().expect("message should be valid");
    }

    #[test]
    fn validate_start_publication_message() {
        let message = super::ClientMessage {
            schema_version: SCHEMA_VERSION,
            sequence_number: 1,
            flags: super::Flags::FIN,
            message_id: *MESSAGE_ID,
            payload_length: 3,
            payload: PAYLOAD.to_vec(),
            message_type: super::MessageType::StartPublicationMessage,
            ..Default::default()
        };

        let result = message.validate();

        assert!(result.is_ok(), "message should be valid");
    }

    #[test]
    fn test_deserialize_data_stream_acknowledge_content() {
        let mut test_message = ClientMessage {
            payload: PAYLOAD.to_vec(),
            ..Default::default()
        };

        let result = test_message.deserialize_data_stream_acknowledge_content();

        assert!(matches!(
            result,
            Err(super::Error::InvalidMessageType { .. })
        ));

        test_message.message_type = super::MessageType::AcknowledgeMessage;

        let result = test_message.deserialize_data_stream_acknowledge_content();

        assert!(matches!(result, Err(super::Error::DeserializeError(_))));

        test_message.payload = ACK_MESSAGE_PAYLOAD.clone();

        let result = test_message
            .deserialize_data_stream_acknowledge_content()
            .expect("msg should be valid");

        assert_eq!(result.message_type, super::MessageType::AcknowledgeMessage);
        assert_eq!(result.message_id, *MESSAGE_ID);
    }
}
