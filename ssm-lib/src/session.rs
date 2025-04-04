//! Types and functions for managing ssm sessions. This is the high-level API for the session manager.
//! Most users will want to use the functions in this module.
//!
//! Roughly corresponds to the code in [this folder](https://github.com/aws/session-manager-plugin/tree/mainline/src/sessionmanagerplugin/session),
//! although input validation logic has been extracted to the main session-manager-plugin crate.

use session_util::DisplayMode;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    data_channel::{DataChannel, DefaultDataChannel},
    error::Error,
    retry::RepeatableExponentialRetryer,
};

mod session_util;

/// A session represents a connection to a target.
#[derive(Debug)]
#[allow(dead_code)] // TODO: remove this once the struct is fully implemented
pub struct Session<Channel>
where
    Channel: DataChannel,
{
    data_channel: Channel,
    session_id: String,
    stream_url: String,
    token_value: String,
    is_aws_cli_upgrade_needed: bool,
    endpoint: String,
    client_id: Uuid,
    target_id: String,
    // sdk: SSM TODO: Implement this
    retry_params: RepeatableExponentialRetryer,
    session_type: String,
    session_properties: HashMap<String, String>,
    display_mode: DisplayMode,
}

impl<Channel> Session<Channel>
where
    Channel: DataChannel,
{
    /// Execute the session.
    ///
    /// ## Errors
    ///
    /// TODO: Fill this in with actual error conditions
    #[allow(clippy::unused_async)] // TODO: Remove this once the function is fully implemented
    pub async fn execute(&self) -> Result<(), Error> {
        println!("\nStarting session with SessionId: {}\n", self.session_id);

        Ok(())
    }

    /// Open a data channel for the session.
    ///
    /// ## Errors
    /// TODO: Fill this in with actual error conditions
    pub fn open_data_channel(&self) -> Result<(), Error> {
        println!(
            "\nOpening data channel for session with SessionId: {}\n",
            self.session_id
        );

        Ok(())
    }
}

/// A builder for creating a [Session].
#[derive(Debug, Default)]
pub struct SessionBuilder<Channel = DefaultDataChannel>
where
    Channel: DataChannel,
{
    data_channel: Channel,
    stream_url: String,
    token_value: String,
    is_aws_cli_upgrade_needed: bool,
    endpoint: String,
    session_id: String,
    target_id: String,
    session_type: String,
    session_properties: HashMap<String, String>,
}

impl SessionBuilder<DefaultDataChannel> {
    /// Create a new session builder with default values.
    #[must_use]
    pub fn new() -> SessionBuilder<DefaultDataChannel> {
        SessionBuilder::default()
    }
}

impl<Channel> SessionBuilder<Channel>
where
    Channel: DataChannel,
{
    /// Set the websockets stream url. This value should be output from the call to the `StartSession` API.
    #[must_use]
    pub fn with_stream_url(mut self, stream_url: String) -> Self {
        self.stream_url = stream_url;
        self
    }

    /// Set the token value. This value should be output from the call to the `StartSession` API.
    #[must_use]
    pub fn with_token_value(mut self, token_value: String) -> Self {
        self.token_value = token_value;
        self
    }

    /// Indicate whether an AWS CLI upgrade is needed. This value should not ordinarily be set, but exists
    /// for compatibility with the original implementation.
    #[must_use]
    #[deprecated]
    pub fn with_aws_cli_upgrade_needed(mut self, is_aws_cli_upgrade_needed: bool) -> Self {
        self.is_aws_cli_upgrade_needed = is_aws_cli_upgrade_needed;
        self
    }

    /// Set the endpoint. This should be the ssm API's endpoint.
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }

    /// Set the session id. This value should be output from the call to the `StartSession` API.
    #[must_use]
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = session_id;
        self
    }

    /// Set the target id. This value should be the EC2 instance ID that was passed to the `StartSession` API.
    #[must_use]
    pub fn with_target_id(mut self, target_id: String) -> Self {
        self.target_id = target_id;
        self
    }

    /// Set a custom data channel. This allows for custom implementations of the data channel.
    pub fn with_data_channel<C>(self, data_channel: C) -> SessionBuilder<C>
    where
        C: DataChannel,
    {
        SessionBuilder {
            data_channel,
            stream_url: self.stream_url,
            token_value: self.token_value,
            is_aws_cli_upgrade_needed: self.is_aws_cli_upgrade_needed,
            endpoint: self.endpoint,
            session_id: self.session_id,
            target_id: self.target_id,
            session_type: self.session_type,
            session_properties: self.session_properties,
        }
    }

    /// Convert the builder into a [Session].
    #[must_use]
    pub fn build(self) -> Session<Channel> {
        Session {
            client_id: Uuid::new_v4(), // original implementation uses golang's uuid.CleanHyphen format; TODO: verify compatibility
            stream_url: self.stream_url,
            token_value: self.token_value,
            is_aws_cli_upgrade_needed: self.is_aws_cli_upgrade_needed,
            endpoint: self.endpoint,
            session_id: self.session_id,
            target_id: self.target_id,
            session_type: self.session_type,
            session_properties: self.session_properties,
            display_mode: DisplayMode::new(), // Note: consider making DisplayMode generic to allow for custom implementations
            retry_params: RepeatableExponentialRetryer::default(),
            data_channel: self.data_channel,
        }
    }
}

// todo: port these two tests
// func TestExecuteAndStreamMessageResendTimesOut(t *testing.T) {
//     sessionMock := &Session{}
//     sessionMock.DataChannel = mockDataChannel
//     SetupMockActions()
//     mockDataChannel.On("Open", mock.Anything).Return(nil)

//     isStreamMessageResendTimeout := make(chan bool, 1)
//     mockDataChannel.On("IsStreamMessageResendTimeout").Return(isStreamMessageResendTimeout)

//     var wg sync.WaitGroup
//     wg.Add(1)
//     handleStreamMessageResendTimeout = func(session *Session, log log.T) {
//         time.Sleep(10 * time.Millisecond)
//         isStreamMessageResendTimeout <- true
//         wg.Done()
//         return
//     }

//     isSessionTypeSetMock := make(chan bool, 1)
//     isSessionTypeSetMock <- true
//     mockDataChannel.On("IsSessionTypeSet").Return(isSessionTypeSetMock)
//     mockDataChannel.On("GetSessionType").Return("Standard_Stream")
//     mockDataChannel.On("GetSessionProperties").Return("SessionProperties")

//     setSessionHandlersWithSessionType = func(session *Session, log log.T) error {
//         return nil
//     }

//     var err error
//     go func() {
//         err = sessionMock.Execute(logger)
//         time.Sleep(200 * time.Millisecond)
//     }()
//     wg.Wait()
//     assert.Nil(t, err)
// }

// func SetupMockActions() {
//     mockDataChannel.On("Initialize", mock.Anything, mock.Anything, mock.Anything, mock.Anything, mock.Anything).Return()
//     mockDataChannel.On("SetWebsocket", mock.Anything, mock.Anything, mock.Anything).Return()
//     mockDataChannel.On("GetWsChannel").Return(mockWsChannel)
//     mockDataChannel.On("RegisterOutputStreamHandler", mock.Anything, mock.Anything)
//     mockDataChannel.On("ResendStreamDataMessageScheduler", mock.Anything).Return(nil)

//     mockWsChannel.On("SetOnMessage", mock.Anything)
//     mockWsChannel.On("SetOnError", mock.Anything)
// }
