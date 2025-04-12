//! Implements a data channel for interactive session.

use crate::{
    config,
    message::{self, MessageType},
    service,
    websocket_channel::{DefaultWebsocketChannel, WebsocketChannel},
};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// TODO: Add a description of the data channel.
#[mockall::automock]
#[allow(clippy::ref_option_ref)] // warning in generated code
pub trait DataChannel {
    /// TODO: document
    ///
    /// ## Errors
    ///
    /// TODO: doc errors
    fn reconnect(&self) -> Result<(), crate::Error>;

    /// TODO: document
    ///
    /// ## Errors
    ///
    /// TODO: document errors
    fn close(&self) -> Result<(), crate::Error>;

    /// TODO: document
    ///
    /// ## Errors
    ///
    /// TODO: doc errors
    fn open(&self) -> Result<(), crate::Error>;

    /// TODO: document
    ///
    /// ## Errors
    ///
    /// TODO: doc errors
    fn finalize_data_channel_handshake(&self, channel_token: &str) -> Result<(), crate::Error>;

    /// TODO: document
    ///
    /// ## Errors
    ///
    /// TODO: doc errors
    fn send_message(&self, input: &[u8], input_type: u32) -> Result<(), crate::Error>;

    /// TODO: document
    ///
    /// ## Errors
    ///
    /// TODO: doc errors
    fn send_input_data_message(
        &self,
        payload_type: message::PayloadType,
        input_data: &[u8],
    ) -> Result<(), crate::Error>;

    /// TODO: document
    fn add_data_to_outgoing_message_buffer(&self, streaming_message: StreamingMessage);

    /// Although we always remove messages from the front of the queue, for concurrency reasons
    /// we need to provide the message to ensure that we remove only the message we aimed to remove.
    /// This is because the application logic will read an item from the buffer, process it, and then
    /// remove it. However, the item may be removed by another thread while it is being processed.
    /// This means we need to confirm that the item we want to remove from the front of the buffer
    /// is actually the one we took first.
    ///
    /// I feel like there could be some way to improve the performance by altering this logic, but will
    /// wait until the implementation is complete so we can set up some performance testing before
    /// making changes.
    fn remove_data_from_outgoing_message_buffer<'a>(
        &'a self,
        streaming_message: Option<&'a StreamingMessage>,
    );
}

/// TODO: Add a description of the default data channel.
#[derive(Default)]
pub struct DefaultDataChannel<Channel = DefaultWebsocketChannel>
where
    Channel: WebsocketChannel,
{
    role: String,
    client_id: String,
    expected_sequence_number: u32,
    /// Use [`RefCell`] to allow interior mutability since callers do not need to know
    /// or care about the mutability of this field as it is an internal implementation detail.
    /// May consider the runtime cost of this in the future.
    stream_data_sequence_number: RefCell<u32>,
    outgoing_message_buffer: Arc<Mutex<ListMessageBuffer>>,
    incoming_message_buffer: MapMessageBuffer,
    round_trip_time: Duration,
    round_trip_time_variation: Duration,
    retransmission_timeout: Duration,
    ws_channel: Channel,
    session_id: String,
    instance_id: String,
    is_aws_cli_upgrade_needed: bool, // TODO: I don't like that this is here; feels like an outer layer should track and handle this
    encryption_enabled: bool,
    /// The original Go project allowed replacing `send_message` at runtime in tests to inject some additional
    /// tracking logic. This is an attempt to reproduce this behavior without impacting runtime performance.
    #[cfg(test)]
    send_message_test_hook: Option<test::TestHook>,
}

impl<Channel> Debug for DefaultDataChannel<Channel>
where
    Channel: Debug + WebsocketChannel,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut it = f.debug_struct("DefaultDataChannel");

        it.field("role", &self.role)
            .field("client_id", &self.client_id)
            .field("expected_sequence_number", &self.expected_sequence_number)
            .field(
                "stream_data_sequence_number",
                &self.stream_data_sequence_number,
            )
            .field("outgoing_message_buffer", &self.outgoing_message_buffer)
            .field("incoming_message_buffer", &self.incoming_message_buffer)
            .field("round_trip_time", &self.round_trip_time)
            .field("round_trip_time_variation", &self.round_trip_time_variation)
            .field("retransmission_timeout", &self.retransmission_timeout)
            .field("ws_channel", &self.ws_channel)
            .field("session_id", &self.session_id)
            .field("instance_id", &self.instance_id)
            .field("is_aws_cli_upgrade_needed", &self.is_aws_cli_upgrade_needed)
            .field("encryption_enabled", &self.encryption_enabled);

        #[cfg(test)]
        it.field(
            "send_message_test_hook",
            &self
                .send_message_test_hook
                .as_ref()
                .map_or("None", |_| "Some(Box<dyn Fn(&str)>)"),
        );

        it.finish()
    }
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
            stream_data_sequence_number: RefCell::new(Self::INITIAL_STREAM_DATA_SEQUENCE_NUMBER),
            outgoing_message_buffer: Arc::new(Mutex::new(ListMessageBuffer::default())),
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
            encryption_enabled: false,
            #[cfg(test)]
            send_message_test_hook: None,
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

impl<Channel> DataChannel for DefaultDataChannel<Channel>
where
    Channel: WebsocketChannel,
{
    fn reconnect(&self) -> Result<(), crate::Error> {
        if let Err(err) = self.close() {
            log::error!("Closing datachannel failed with error: {err}");
        }

        self.open()?;

        log::info!(
            "Successfully reconnected to data channel: {}",
            self.ws_channel.get_stream_url()
        );

        Ok(())
    }

    fn close(&self) -> Result<(), crate::Error> {
        log::info!(
            "Closing datachannel with url {}",
            self.ws_channel.get_stream_url()
        );
        self.ws_channel.close()
    }

    fn open(&self) -> Result<(), crate::Error> {
        self.ws_channel.open()?;

        self.finalize_data_channel_handshake(self.ws_channel.get_channel_token())?;

        Ok(())
    }

    fn finalize_data_channel_handshake(&self, channel_token: &str) -> Result<(), crate::Error> {
        log::info!(
            "Sending token through data channel {} to acknowledge connection",
            self.ws_channel.get_stream_url()
        );

        let open_data_channel_input =
            service::OpenDataChannelInput::new(channel_token.to_string(), self.client_id.clone());

        let open_data_channel_input = serde_json::to_string(&open_data_channel_input)
            .map_err(crate::Error::OpenDataChannelInputSerialization)?;

        // TODO: need to check tokio-tungstenite and handle this the way it should be handled
        self.send_message(open_data_channel_input.as_bytes(), 0)
    }

    fn send_message(&self, input: &[u8], input_type: u32) -> Result<(), crate::Error> {
        // Original code included a test hook like this, in a go-specific way. This is an attempt to reproduce the pattern
        // in Rust without affecting runtime performance.
        #[cfg(test)]
        if let Some(hook) = &self.send_message_test_hook {
            hook(input, input_type);
        }
        self.ws_channel.send_message(input, input_type)
    }

    fn send_input_data_message(
        &self,
        payload_type: message::PayloadType,
        input_data: &[u8],
    ) -> Result<(), crate::Error> {
        // below is exact comment from original code
        // today 'enter' is taken as 'next line' in winpty shell. so hardcoding 'next line' byte to actual 'enter' byte
        let input_data = if input_data == [10] {
            &[13]
        } else {
            input_data
        };

        if self.encryption_enabled && payload_type == message::PayloadType::Output {
            todo!()
        }

        let client_message = message::ClientMessage::new(
            MessageType::InputStreamMessage,
            message::Flags::empty(),
            payload_type,
            input_data.to_vec(), // TODO: remove allocations by using a slice or array instead of a vector
            (*self.stream_data_sequence_number.borrow()).into(), // TODO: understand why message uses a i64 and not a u32
        );

        log::trace!(
            "Sending message with seq number: {}",
            self.stream_data_sequence_number.borrow()
        );

        // TODO: need to make an error log message here to match the original implementation
        let msg = client_message.serialize()?;

        // TODO: log an error message if error as with original
        self.send_message(&msg, 0)?;

        let streaming_message =
            StreamingMessage::new(msg, (*self.stream_data_sequence_number.borrow()).into());

        self.add_data_to_outgoing_message_buffer(streaming_message);

        self.stream_data_sequence_number.replace_with(|x| *x + 1);
        dbg!(&self.stream_data_sequence_number);
        Ok(())
    }

    fn add_data_to_outgoing_message_buffer(&self, stream_message: StreamingMessage) {
        let mut messages = match self.outgoing_message_buffer.lock() {
            Ok(a) => a,
            Err(e) => {
                log::error!(
                    "Thread panicked while holding a Mutex lock. Please report to the crate's maintainers: {e}"
                );
                e.into_inner()
            }
        };

        if messages.is_full() {
            let message = messages.pop_front();
            log::warn!("Outgoing message buffer full. Dropping message: {message:#?}");
        }

        messages.push_back(stream_message);
    }

    fn remove_data_from_outgoing_message_buffer(
        &self,
        _streaming_message: Option<&StreamingMessage>,
    ) {
        todo!()
    }
}

#[derive(Debug, Default)]
struct ListMessageBuffer {
    messages: VecDeque<StreamingMessage>, // Wrap in mutex when it becomes necessary
}

impl ListMessageBuffer {
    // fn new() -> Self {
    //     Self {
    //         messages: VecDeque::with_capacity(config::OUTGOING_MESSAGE_BUFFER_CAPACITY),
    //     }
    // }

    fn is_full(&self) -> bool {
        self.messages.len() >= config::OUTGOING_MESSAGE_BUFFER_CAPACITY
    }

    pub fn pop_front(&mut self) -> Option<StreamingMessage> {
        self.messages.pop_front()
    }

    // pub fn front(&self) -> Option<&StreamingMessage> {
    //     self.messages.front()
    // }

    // pub fn pop_back(&mut self) -> Option<StreamingMessage> {
    //     self.messages.pop_back()
    // }

    // pub fn remove(&mut self, index: usize) -> Option<StreamingMessage> {
    //     self.messages.remove(index)
    // }

    pub fn push_back(&mut self, message: StreamingMessage) {
        self.messages.push_back(message);
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

/// TODO: document
#[derive(Debug)]
#[allow(dead_code)] // TODO: remove after implementation complete
pub struct StreamingMessage {
    content: Vec<u8>, // TODO: check characterics of message to determine whether a vec or array is more appropriate
    sequence_number: u64,
    last_sent_time: Instant,
    resent_attempt: u32,
}

impl StreamingMessage {
    /// TODO: document
    #[must_use]
    pub fn new(content: Vec<u8>, sequence_number: u64) -> Self {
        Self {
            content,
            sequence_number,
            last_sent_time: Instant::now(),
            resent_attempt: 0,
        }
    }
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
    use mockall::predicate::eq;

    use super::DataChannel;
    use super::DefaultDataChannel;
    use super::config;
    use crate::message::PayloadType;
    use crate::service::OpenDataChannelInput;
    use crate::websocket_channel::MockWebsocketChannel;

    const CLIENT_ID: &str = "client-id";
    const SESSION_ID: &str = "session-id";
    const INSTANCE_ID: &str = "instance-id";
    const CHANNEL_TOKEN: &str = "channel-token";
    const MESSAGE: &[u8] = b"message";
    const STREAM_DATA_SEQUENCE_NUMBER: u32 = 0;
    const PAYLOAD: &[u8] = b"testPayload";
    // const STREAM_URL: &str = "stream-url";

    pub type TestHook = Box<dyn Fn(&[u8], u32)>;

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
        assert_eq!(0, *data_channel.stream_data_sequence_number.borrow());
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
    }

    // This test is unnecessary in the Rust implementation. Leaving it here in order to track the fact that
    // this test is not implemented due to an oversight.
    // #[test]
    // fn set_websocket() {}

    #[test]
    fn reconnect() {
        let mut ws_channel = MockWebsocketChannel::new();

        ws_channel.expect_close().once().returning(|| Ok(()));
        ws_channel.expect_open().once().returning(|| Ok(()));

        ws_channel
            .expect_get_channel_token()
            .once()
            .return_const(CHANNEL_TOKEN.to_string());

        ws_channel
            .expect_send_message()
            .once()
            .withf(open_data_channel_input)
            .returning(|_, _| Ok(()));

        let data_channel: DefaultDataChannel<MockWebsocketChannel> = get_data_channel(ws_channel);

        data_channel.reconnect().expect("Reconnect should succeed.");
    }

    #[test]
    fn open() {
        let mut ws_channel = MockWebsocketChannel::new();

        ws_channel.expect_open().once().returning(|| Ok(()));
        ws_channel
            .expect_get_channel_token()
            .once()
            .return_const(CHANNEL_TOKEN.to_string());

        ws_channel
            .expect_send_message()
            .once()
            .withf(open_data_channel_input)
            .returning(|_, _| Ok(()));

        let data_channel: DefaultDataChannel<MockWebsocketChannel> = get_data_channel(ws_channel);

        data_channel.open().expect("Open should succeed.");
    }

    #[test]
    fn close() {
        let mut ws_channel = MockWebsocketChannel::new();

        ws_channel.expect_close().once().returning(|| Ok(()));

        let data_channel: DefaultDataChannel<MockWebsocketChannel> = get_data_channel(ws_channel);

        data_channel.close().expect("Close should succeed.");
    }

    #[test]
    fn finalize_data_channel_handshake() {
        let mut ws_channel = MockWebsocketChannel::new();

        ws_channel
            .expect_send_message()
            .once()
            .withf(open_data_channel_input)
            .returning(|_, _| Ok(()));

        // The original code was expecting a call to get_channel_token here, but in Rust, the call to log::info!
        // appears to not be invoked in tests.

        let data_channel: DefaultDataChannel<MockWebsocketChannel> = get_data_channel(ws_channel);

        data_channel
            .finalize_data_channel_handshake(CHANNEL_TOKEN)
            .expect("Finalize data channel handshake should succeed.");
    }

    #[test]
    fn test_send_message() {
        let mut ws_channel = MockWebsocketChannel::new();

        ws_channel
            .expect_send_message()
            .once()
            .with(eq(MESSAGE), eq(0))
            .returning(|_, _| Ok(()));

        let data_channel: DefaultDataChannel<MockWebsocketChannel> = get_data_channel(ws_channel);

        data_channel
            .send_message(MESSAGE, 0)
            .expect("Send message should succeed.");
    }

    #[test]
    fn send_input_data_message() {
        let mut ws_channel = MockWebsocketChannel::new();

        ws_channel
            .expect_send_message()
            .once()
            // We don't test the input now since that would require implementing the serializer, which is too much
            // work for the current unit of work
            // .with(eq(MESSAGE), eq(0))
            .returning(|_, _| Ok(()));

        let data_channel: DefaultDataChannel<MockWebsocketChannel> = get_data_channel(ws_channel);

        data_channel
            .send_input_data_message(PayloadType::Output, PAYLOAD)
            .expect("Send input data message should succeed.");

        assert_eq!(
            STREAM_DATA_SEQUENCE_NUMBER + 1,
            *data_channel.stream_data_sequence_number.borrow()
        );
        assert_eq!(
            1,
            data_channel
                .outgoing_message_buffer
                .lock()
                .unwrap()
                .messages
                .len()
        );
    }

    // Allow trivially_copy_pass_by_ref because the input is a reference and we can't change that.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    // We don't check the message id because its generated internally and we don't care about it
    fn open_data_channel_input(input: &[u8], message_type: &u32) -> bool {
        let input: OpenDataChannelInput =
            serde_json::from_slice(input).expect("Failed to deserialize input");
        input.message_schema_version == config::MESSAGE_SCHEMA_VERSION
            && input.client_id == CLIENT_ID
            && input.token_value == CHANNEL_TOKEN
            && input.client_version == env!("CARGO_PKG_VERSION")
            && *message_type == 0 // TODO: check this value
    }

    fn get_data_channel(
        ws_channel: MockWebsocketChannel,
    ) -> DefaultDataChannel<MockWebsocketChannel> {
        DefaultDataChannel::new(
            CLIENT_ID.to_string(),
            SESSION_ID.to_string(),
            INSTANCE_ID.to_string(),
            ws_channel,
        )
    }
}
