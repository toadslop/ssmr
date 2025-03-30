//! Configuration for the SSM library. The values here are currently hardcoded but
//! will be made customizable in the future.

// TODO: make the config available customizable so that callers can provide their own configuration
// for now, use the hardcoded configuration from the original implementation

/// Defines the geometric ratio for the exponential backoff algorithm
pub const RETRY_BASE: u32 = 2;

/// Used to define the initial delay for the exponential backoff algorithm.
pub const DATA_CHANNEL_RETRY_INITIAL_DELAY_MILLIS: u64 = 100;

/// Used to define the maximum delay for the exponential backoff algorithm.
pub const DATA_CHANNEL_RETRY_MAX_INTERVAL_MILLIS: u64 = 5000;

/// Used to define the maximum number of retries for the exponential backoff algorithm.
pub const DATA_CHANNEL_NUM_MAX_RETRIES: u64 = 5;

/// TODO: document
pub const ROLE_PUBLISH_SUBSCRIBE: &str = "publish_subscribe";

/// The size of the buffer for outgoing messages.
pub const OUTGOING_MESSAGE_BUFFER_CAPACITY: usize = 10000;

/// The size of the buffer for incoming messages.
pub const INCOMING_MESSAGE_BUFFER_CAPACITY: usize = 10000;

/// TODO: document
pub const DEFAULT_ROUND_TRIP_TIME_MILLIS: u64 = 100;

/// TODO: document
pub const DEFAULT_ROUND_TRIP_TIME_VARIATION_MILLIS: u64 = 0;

/// TODO: document
pub const DEFAULT_TRANSMISSION_TIMEOUT_MILLIS: u64 = 200;
