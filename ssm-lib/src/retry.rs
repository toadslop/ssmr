use crate::config;
use rand::Rng;
use std::{thread, time::Duration};

#[derive(Debug)]
#[allow(dead_code)] // TODO: remove this once the struct is fully implemented
pub struct RepeatableExponentialRetryer {
    geometric_ratio: u32,
    initial_delay: Duration,
    max_delay: Duration,
    max_attempts: u64,
}

#[allow(dead_code)] // TODO: remove this once the struct is fully implemented
impl RepeatableExponentialRetryer {
    /// For the given fallible function, retry it until it succeeds or the maximum number of attempts is reached.
    pub fn retry<T, E>(&self, mut func: impl FnMut() -> Result<T, E>) -> Result<T, E> {
        let mut attempt = 1;
        let mut failed_attempts_so_far = 0;

        loop {
            match func() {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempt += 1;
                    failed_attempts_so_far += 1;
                    if failed_attempts_so_far >= self.max_attempts {
                        dbg!(failed_attempts_so_far);
                        return Err(err);
                    }

                    let mut sleep = self.next_sleep_time(attempt);

                    if sleep > self.max_delay {
                        attempt = 0;
                        sleep = self.next_sleep_time(attempt);
                    }

                    thread::sleep(sleep);
                }
            }
        }
    }

    /// Calculate the next sleep time based on the current attempt number.
    fn next_sleep_time(&self, attempt: u32) -> Duration {
        self.initial_delay * self.geometric_ratio.pow(attempt)
    }
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

#[cfg(test)]
mod test {
    use crate::{config, retry::RepeatableExponentialRetryer};

    #[test]
    fn repeatable_exponential_retryer_retries_for_given_number_of_max_retries() {
        let retryer = RepeatableExponentialRetryer::default();
        let mut attempts = 0;

        let result: Result<_, &str> = retryer.retry(|| {
            attempts += 1;
            Err::<(), &str>("error")
        });

        assert_eq!(result, Err("error"));
        assert_eq!(attempts, config::DATA_CHANNEL_NUM_MAX_RETRIES);
    }
}
