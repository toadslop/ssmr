//! Configuration for the SSM library. The values here are currently hardcoded but
//! will be made customizable in the future.

// TODO: make the config available customizable so that callers can provide their own configuration
// for now, use the hardcoded configuration from the original implementation

/// Defines the geometric ratio for the exponential backoff algorithm
pub const RETRY_BASE: f64 = 2.0;

/// Used to define the initial delay for the exponential backoff algorithm.
pub const DATA_CHANNEL_RETRY_INITIAL_DELAY_MILLIS: u64 = 100;

/// Used to define the maximum delay for the exponential backoff algorithm.
pub const DATA_CHANNEL_RETRY_MAX_INTERVAL_MILLIS: u64 = 5000;

/// Used to define the maximum number of retries for the exponential backoff algorithm.
pub const DATA_CHANNEL_NUM_MAX_RETRIES: u64 = 5;
