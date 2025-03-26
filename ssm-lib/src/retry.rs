use crate::config;
use rand::Rng;
use std::time::Duration;

#[derive(Debug)]
#[allow(dead_code)] // TODO: remove this once the struct is fully implemented
pub struct RepeatableExponentialRetryer {
    geometric_ratio: f64,
    initial_delay: Duration,
    max_delay: Duration,
    max_attempts: u64,
}

impl Default for RepeatableExponentialRetryer {
    fn default() -> Self {
        RepeatableExponentialRetryer {
            geometric_ratio: config::RETRY_BASE,
            initial_delay: Duration::from_millis(
                rand::rng().random_range(0..config::DATA_CHANNEL_RETRY_INITIAL_DELAY_MILLIS)
                    + config::DATA_CHANNEL_RETRY_INITIAL_DELAY_MILLIS,
            ),
            max_delay: Duration::from_millis(config::DATA_CHANNEL_RETRY_MAX_INTERVAL_MILLIS),
            max_attempts: config::DATA_CHANNEL_NUM_MAX_RETRIES,
        }
    }
}
