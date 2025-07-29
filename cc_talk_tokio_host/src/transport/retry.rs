use std::time::Duration;

use super::tokio_transport::TransportError;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub retry_on_timeout: bool,
    pub retry_on_checksum_error: bool,
    pub retry_on_nack: bool,
    pub retry_on_socket_error: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            retry_on_timeout: true,
            retry_on_checksum_error: true,
            retry_on_nack: false,
            retry_on_socket_error: true,
        }
    }
}

impl RetryConfig {
    pub fn create_retry_instance(&self) -> RetryInstance {
        RetryInstance::new(
            self.max_retries,
            self.retry_on_timeout,
            self.retry_on_checksum_error,
            self.retry_on_nack,
            self.retry_on_socket_error,
            self.retry_delay,
        )
    }
}

pub struct RetryInstance {
    attempt: u32,
    max_tries: u32,
    last_error: TransportError,
    retry_on_timeout: bool,
    retry_on_checksum_error: bool,
    retry_on_nack: bool,
    retry_on_socket_error: bool,
    can_retry: bool,
    delay: Duration,
}

impl RetryInstance {
    fn new(
        max_tries: u32,
        retry_on_timeout: bool,
        retry_on_checksum_error: bool,
        retry_on_nack: bool,
        retry_on_socket_error: bool,
        delay: Duration,
    ) -> Self {
        RetryInstance {
            attempt: 0,
            can_retry: true,
            last_error: TransportError::Timeout,
            max_tries,
            retry_on_timeout,
            retry_on_checksum_error,
            retry_on_nack,
            retry_on_socket_error,
            delay,
        }
    }

    pub fn should_retry(&self, error: TransportError) -> bool {
        match error {
            TransportError::Timeout => self.retry_on_timeout,
            TransportError::ChecksumError => self.retry_on_checksum_error,
            TransportError::Nack => self.retry_on_nack,
            TransportError::SocketWriteError | TransportError::SocketReadError => {
                self.retry_on_socket_error
            }
            // Never retry these
            TransportError::BufferOverflow
            | TransportError::PacketCreationError
            | TransportError::MaxRetriesExceeded => false,
        }
    }

    pub fn evaluate_error(&mut self, error: TransportError) {
        if !self.should_retry(error) {
            self.can_retry = false;
        }
        self.attempt += 1;
        if self.attempt > self.max_tries {
            self.can_retry = false;
        }
        self.last_error = error;
    }

    pub async fn delay_for_retry(&self) {
        if !self.delay.is_zero() && self.can_retry() {
            tokio::time::sleep(self.delay).await;
        }
    }

    pub async fn evaluate_and_wait(&mut self, error: TransportError) {
        self.evaluate_error(error);
        if self.can_retry {
            self.delay_for_retry().await;
        }
    }

    pub fn last_error(&self) -> TransportError {
        self.last_error
    }

    pub fn can_retry(&self) -> bool {
        self.can_retry
    }
}
