// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! OANDA transaction streaming client implementation.
//!
//! This client uses HTTP streaming (chunked transfer encoding) to receive
//! real-time transaction updates from OANDA (order fills, cancels, etc.).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use log::{debug, error, info, warn};
use reqwest::Client;
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::types::{
    RawTransactionStreamMessage, StreamError, StreamState, StreamStats,
    TransactionStreamConfig, TransactionStreamMessage,
};

/// Timeout for reading a message (including heartbeats).
const MESSAGE_TIMEOUT_SECS: u64 = 30;

/// OANDA transaction streaming client for real-time account events.
///
/// This client connects to OANDA's transaction streaming API and provides
/// an async stream of transaction events (order fills, cancels, etc.).
///
/// # Example
///
/// ```rust,ignore
/// use nautilus_oanda::websocket::{OANDATransactionStreamClient, TransactionStreamConfig};
///
/// let config = TransactionStreamConfig::new(
///     OANDAEnvironment::Practice,
///     "your-api-key",
///     "your-account-id",
/// );
///
/// let mut client = OANDATransactionStreamClient::new(config);
/// client.connect().await?;
///
/// while let Some(msg) = client.next().await {
///     match msg {
///         Ok(TransactionStreamMessage::Transaction(tx)) => {
///             println!("Transaction: {:?} - {:?}", tx.transaction_type, tx.instrument);
///         }
///         Ok(TransactionStreamMessage::Heartbeat(hb)) => {
///             println!("Heartbeat at {}", hb.time);
///         }
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
#[derive(Debug)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.oanda")
)]
pub struct OANDATransactionStreamClient {
    /// Configuration for the stream.
    config: TransactionStreamConfig,

    /// HTTP client for the streaming connection.
    http_client: Client,

    /// Current connection state.
    state: StreamState,

    /// Statistics for this stream.
    stats: StreamStats,

    /// Whether the client should be running.
    running: Arc<AtomicBool>,

    /// Channel receiver for stream messages.
    receiver: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<Result<TransactionStreamMessage, StreamError>>>>>,
}

impl Clone for OANDATransactionStreamClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            state: self.state,
            stats: self.stats.clone(),
            running: Arc::clone(&self.running),
            receiver: Arc::clone(&self.receiver),
        }
    }
}

impl OANDATransactionStreamClient {
    /// Create a new transaction stream client.
    #[must_use]
    pub fn new(config: TransactionStreamConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            state: StreamState::Disconnected,
            stats: StreamStats::default(),
            running: Arc::new(AtomicBool::new(false)),
            receiver: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Get the current connection state.
    #[must_use]
    pub fn state(&self) -> StreamState {
        self.state
    }

    /// Get stream statistics.
    #[must_use]
    pub fn stats(&self) -> &StreamStats {
        &self.stats
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &TransactionStreamConfig {
        &self.config
    }

    /// Check if the client is currently running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Connect to the OANDA transaction streaming API.
    ///
    /// This establishes the HTTP streaming connection and starts receiving
    /// transaction events in the background.
    pub async fn connect(&mut self) -> Result<(), StreamError> {
        // Validate configuration
        self.config.validate()?;

        if self.is_running() {
            return Ok(());
        }

        self.state = StreamState::Connecting;
        self.running.store(true, Ordering::SeqCst);

        let url = self.config.streaming_url();
        info!("Connecting to OANDA transaction stream: {}", url);

        // Create channel for messages
        let (tx, rx) = mpsc::channel(1000);
        *self.receiver.lock().await = Some(rx);

        // Spawn background task to read from stream
        let config = self.config.clone();
        let http_client = self.http_client.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            let mut reconnect_attempts = 0;

            while running.load(Ordering::SeqCst) {
                match Self::run_stream(&config, &http_client, &tx).await {
                    Ok(()) => {
                        // Stream ended normally
                        break;
                    }
                    Err(e) => {
                        warn!("Transaction stream error: {}", e);

                        if !config.reconnect_on_error {
                            let _ = tx.send(Err(e)).await;
                            break;
                        }

                        reconnect_attempts += 1;
                        if config.max_reconnect_attempts > 0
                            && reconnect_attempts >= config.max_reconnect_attempts
                        {
                            error!(
                                "Max reconnection attempts ({}) exceeded",
                                config.max_reconnect_attempts
                            );
                            let _ = tx
                                .send(Err(StreamError::MaxReconnects(
                                    config.max_reconnect_attempts,
                                )))
                                .await;
                            break;
                        }

                        let delay = Duration::from_millis(config.reconnect_delay_ms);
                        info!(
                            "Reconnecting in {:?} (attempt {})",
                            delay, reconnect_attempts
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }

            running.store(false, Ordering::SeqCst);
        });

        self.state = StreamState::Connected;
        Ok(())
    }

    /// Run the streaming connection (internal).
    async fn run_stream(
        config: &TransactionStreamConfig,
        http_client: &Client,
        tx: &mpsc::Sender<Result<TransactionStreamMessage, StreamError>>,
    ) -> Result<(), StreamError> {
        let url = config.streaming_url();

        let response = http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Accept-Datetime-Format", "RFC3339")
            .send()
            .await
            .map_err(|e| StreamError::Connection(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(StreamError::Http {
                status: status.as_u16(),
                message: body,
            });
        }

        debug!("Transaction stream connected");

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        loop {
            let chunk = match timeout(Duration::from_secs(MESSAGE_TIMEOUT_SECS), stream.next()).await
            {
                Ok(Some(Ok(bytes))) => bytes,
                Ok(Some(Err(e))) => {
                    return Err(StreamError::Connection(e.to_string()));
                }
                Ok(None) => {
                    return Err(StreamError::Disconnected("Stream ended".into()));
                }
                Err(_) => {
                    return Err(StreamError::Timeout(MESSAGE_TIMEOUT_SECS));
                }
            };

            // Append to buffer
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                // Parse the JSON message
                match serde_json::from_str::<RawTransactionStreamMessage>(&line) {
                    Ok(raw_msg) => {
                        let msg: TransactionStreamMessage = raw_msg.into();

                        // Skip heartbeats if configured
                        if !config.include_heartbeats {
                            if let TransactionStreamMessage::Heartbeat(_) = msg {
                                continue;
                            }
                        }

                        if tx.send(Ok(msg)).await.is_err() {
                            // Receiver dropped
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        debug!("Failed to parse transaction message: {} - {}", e, line);
                        // Continue processing - don't fail on parse errors
                    }
                }
            }
        }
    }

    /// Disconnect from the streaming API.
    pub async fn disconnect(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.state = StreamState::Closed;

        // Clear receiver
        *self.receiver.lock().await = None;

        info!("Transaction stream disconnected");
    }

    /// Get the next message from the stream.
    ///
    /// Returns `None` if the stream is closed or not connected.
    pub async fn next(&mut self) -> Option<Result<TransactionStreamMessage, StreamError>> {
        let mut guard = self.receiver.lock().await;
        if let Some(ref mut rx) = *guard {
            match rx.recv().await {
                Some(result) => {
                    // Update stats
                    if let Ok(ref msg) = result {
                        match msg {
                            TransactionStreamMessage::Transaction(tx) => {
                                self.stats.record_price(&tx.time);
                            }
                            TransactionStreamMessage::Heartbeat(hb) => {
                                self.stats.record_heartbeat(&hb.time);
                            }
                        }
                    }
                    Some(result)
                }
                None => {
                    self.state = StreamState::Closed;
                    None
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::enums::OANDAEnvironment;

    fn create_test_config() -> TransactionStreamConfig {
        TransactionStreamConfig::new(
            OANDAEnvironment::Practice,
            "test-api-key",
            "test-account-id",
        )
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config();
        let client = OANDATransactionStreamClient::new(config);

        assert_eq!(client.state(), StreamState::Disconnected);
        assert!(!client.is_running());
    }

    #[test]
    fn test_config_url() {
        let config = TransactionStreamConfig::new(
            OANDAEnvironment::Practice,
            "api-key",
            "account-123",
        );

        let url = config.streaming_url();
        assert!(url.contains("stream-fxpractice.oanda.com"));
        assert!(url.contains("account-123"));
        assert!(url.contains("/transactions/stream"));
    }

    #[test]
    fn test_config_validation() {
        let mut config = TransactionStreamConfig::default();

        // Missing API key
        assert!(config.validate().is_err());

        config.api_key = "test".into();
        // Missing account ID
        assert!(config.validate().is_err());

        config.account_id = "test".into();
        // Should pass now
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_client_stats() {
        let config = create_test_config();
        let client = OANDATransactionStreamClient::new(config);

        let stats = client.stats();
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.prices_received, 0);
    }
}
