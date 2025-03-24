use std::time::Duration;

use crate::config;

#[derive(Debug)]
pub struct RepeatableExponentialRetryer {
    geometric_ratio: f64,
    initial_delay: Duration,
    max_delay: Duration,
    max_attempts: u32,
}

impl Default for RepeatableExponentialRetryer {
    fn default() -> Self {
        RepeatableExponentialRetryer {
            geometric_ratio: config::RETRY_BASE,
            // TODO: finish the following
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_attempts: 5,
        }
    }
}
