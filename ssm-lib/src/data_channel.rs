//! Implements a data channel for interactive session.

use crate::{
    config,
    websocket_channel::{DefaultWebsocketChannel, WebsocketChannel},
};
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

/// TODO: Add a description of the data channel.
#[mockall::automock]
pub trait DataChannel {}

/// TODO: Add a description of the default data channel.
#[derive(Debug, Default)]
#[allow(dead_code)] // TODO: remove once implementation complete
pub struct DefaultDataChannel<Channel = DefaultWebsocketChannel>
where
    Channel: WebsocketChannel,
{
    role: String,
    client_id: String,
    expected_sequence_number: u32,
    stream_data_sequence_number: u32,
    outoging_message_buffer: ListMessageBuffer,
    incoming_message_buffer: MapMessageBuffer,
    round_trip_time: Duration,
    round_trip_time_variation: Duration,
    retransmission_timeout: Duration,
    ws_channel: Channel,
    session_id: String,
    instance_id: String,
    is_aws_cli_upgrade_needed: bool, // TODO: I don't like that this is here; feels like an outer layer should track and handle this
}

impl DefaultDataChannel {
    const INITIAL_EXPECTED_SEQUENCE_NUMBER: u32 = 0;
    const INITIAL_STREAM_DATA_SEQUENCE_NUMBER: u32 = 0;

    /// TODO: document
    #[must_use]
    pub fn new<C>(
        client_id: String,
        session_id: String,
        instance_id: String,
        ws_channel: C,
    ) -> DefaultDataChannel<C>
    where
        C: WebsocketChannel,
    {
        DefaultDataChannel {
            role: config::ROLE_PUBLISH_SUBSCRIBE.to_string(),
            client_id,
            expected_sequence_number: Self::INITIAL_EXPECTED_SEQUENCE_NUMBER,
            stream_data_sequence_number: Self::INITIAL_STREAM_DATA_SEQUENCE_NUMBER,
            outoging_message_buffer: ListMessageBuffer::default(),
            incoming_message_buffer: MapMessageBuffer::default(),
            round_trip_time: Duration::from_millis(config::DEFAULT_ROUND_TRIP_TIME_MILLIS),
            round_trip_time_variation: Duration::from_millis(
                config::DEFAULT_ROUND_TRIP_TIME_VARIATION_MILLIS,
            ),
            retransmission_timeout: Duration::from_millis(
                config::DEFAULT_TRANSMISSION_TIMEOUT_MILLIS,
            ),
            ws_channel,
            session_id,
            instance_id,
            is_aws_cli_upgrade_needed: false,
        }
    }

    /// Indicate that the parameters passed were provided by an outated version of the AWS CLI.
    ///
    /// ## Note
    ///
    /// This field was from the original implementation, but references to the AWS CLI should not appear
    /// in lib crate. Need to determine how this is used and lift it out to the CLI crate.
    pub fn set_aws_cli_upgrade_needed(&mut self) {
        self.is_aws_cli_upgrade_needed = true;
    }
}

impl DataChannel for DefaultDataChannel {}

#[derive(Debug, Default)]
#[allow(dead_code)] // TODO: remove after implementation complete
struct ListMessageBuffer {
    messages: VecDeque<StreamingMessage>, // Wrap in mutex when it becomes necessary
}

#[allow(dead_code)] // TODO: remove after implementation complete
impl ListMessageBuffer {
    fn new() -> Self {
        Self {
            messages: VecDeque::with_capacity(config::OUTGOING_MESSAGE_BUFFER_CAPACITY),
        }
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)] // TODO: remove after implementation complete
struct MapMessageBuffer {
    messages: HashMap<u32, StreamingMessage>, // Wrap in mutex when it becomes necessary
}

#[allow(dead_code)] // TODO: remove after implementation complete
impl MapMessageBuffer {
    fn new() -> Self {
        Self {
            messages: HashMap::with_capacity(config::INCOMING_MESSAGE_BUFFER_CAPACITY),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)] // TODO: remove after implementation complete
struct StreamingMessage {
    content: Vec<u8>, // TODO: check characterics of message to determine whether a vec or array is more appropriate
    sequence_number: u64,
    last_sent_time: Instant,
    resent_attempt: u32,
}

impl Default for StreamingMessage {
    fn default() -> Self {
        Self {
            content: Vec::default(),
            sequence_number: Default::default(),
            last_sent_time: Instant::now(),
            resent_attempt: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::websocket_channel::MockWebsocketChannel;

    use super::DefaultDataChannel;
    use super::config;

    const CLIENT_ID: &str = "client-id";
    const SESSION_ID: &str = "session-id";
    const INSTANCE_ID: &str = "instance-id";

    #[test]
    fn initialize() {
        let mock_ws_channel = MockWebsocketChannel::new();

        let data_channel = DefaultDataChannel::new(
            CLIENT_ID.to_owned(),
            SESSION_ID.to_owned(),
            INSTANCE_ID.to_owned(),
            mock_ws_channel,
        );

        assert_eq!(config::ROLE_PUBLISH_SUBSCRIBE, data_channel.role);
        assert_eq!(CLIENT_ID, data_channel.client_id);
        assert_eq!(SESSION_ID, data_channel.session_id);
        assert_eq!(INSTANCE_ID, data_channel.instance_id);
        assert!(!data_channel.is_aws_cli_upgrade_needed);
        assert_eq!(0, data_channel.expected_sequence_number);
        assert_eq!(0, data_channel.stream_data_sequence_number);
        assert_eq!(
            u128::from(config::DEFAULT_ROUND_TRIP_TIME_MILLIS),
            data_channel.round_trip_time.as_millis()
        );
        assert_eq!(
            u128::from(config::DEFAULT_ROUND_TRIP_TIME_VARIATION_MILLIS),
            data_channel.round_trip_time_variation.as_millis()
        );
        assert_eq!(
            u128::from(config::DEFAULT_TRANSMISSION_TIMEOUT_MILLIS),
            data_channel.retransmission_timeout.as_millis()
        );

        // assert.NotNil(t, datachannel.wsChannel)
    }
}
