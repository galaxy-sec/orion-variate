use crate::addr::AddrError;
use crate::addr::AddrReason;
use std::future::Future;
use std::time::{Duration, Instant};

use crate::addr::AddrResult;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub total_timeout: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            total_timeout: Duration::from_secs(300),
        }
    }
}

impl RetryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn http_simple() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            total_timeout: Duration::from_secs(300),
        }
    }

    pub fn http_large_file() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(15),
            total_timeout: Duration::from_secs(3600), // 1 hour
        }
    }

    pub fn git_operation() -> Self {
        Self {
            max_attempts: 2,
            base_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(30),
            total_timeout: Duration::from_secs(1800), // 30 minutes
        }
    }
}

#[derive(Debug)]
pub struct RetryOperation {
    config: RetryConfig,
}

impl RetryOperation {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub async fn execute<F, Fut, T>(&self, operation: F) -> AddrResult<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = AddrResult<T>>,
    {
        let start_time = Instant::now();
        let mut last_error = None;

        for attempt in 1..=self.config.max_attempts {
            if start_time.elapsed() >= self.config.total_timeout {
                return Err(AddrError::from(AddrReason::Brief(format!(
                    "Total timeout {}ms exceeded",
                    self.config.total_timeout.as_millis()
                ))));
            }

            match tokio::time::timeout(self.config.total_timeout, operation()).await {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt == self.config.max_attempts {
                        return Err(AddrError::from(AddrReason::Brief(format!(
                            "Retry exhausted after {} attempts",
                            attempt
                        ))));
                    }
                }
                Err(_) => {
                    if attempt == self.config.max_attempts {
                        return Err(AddrError::from(AddrReason::Brief(format!(
                            "Operation timeout after {} attempts",
                            attempt
                        ))));
                    }
                }
            }

            let delay =
                (self.config.base_delay * 2_u32.pow(attempt - 1)).min(self.config.max_delay);

            tracing::debug!(
                "Retry attempt {}/{} after {:?}",
                attempt,
                self.config.max_attempts,
                delay
            );

            if attempt < self.config.max_attempts {
                tokio::time::sleep(delay).await;
            }
        }

        Err(AddrError::from(AddrReason::Brief(
            "All retry attempts exhausted".to_string(),
        )))
    }
}

pub async fn retry_with_config<F, Fut, T>(operation: F, config: RetryConfig) -> AddrResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = AddrResult<T>>,
{
    RetryOperation::new(config).execute(operation).await
}
