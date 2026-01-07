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

//! OANDA streaming client implementation.
//!
//! This client uses HTTP streaming (chunked transfer encoding) to receive
//! real-time price updates from OANDA. Messages are newline-delimited JSON.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use log::{debug, error, info, warn};
use reqwest::Client;
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::types::{
    RawStreamMessage, StreamConfig, StreamError, StreamMessage, StreamState, StreamStats,
};

/// Timeout for reading a message (including heartbeats).
const MESSAGE_TIMEOUT_SECS: u64 = 30;

/// OANDA streaming client for real-time price feeds.
///
/// This client connects to OANDA's streaming API and provides an async stream
/// of price updates and heartbeat messages.
///
/// # Example
///
/// ```rust,ignore
/// use nautilus_oanda::websocket::{OANDAStreamClient, StreamConfig};
///
/// let config = StreamConfig::new(
///     OANDAEnvironment::Practice,
///     "your-api-key",
///     "your-account-id",
///     vec!["EUR_USD".into()],
/// );
///
/// let mut client = OANDAStreamClient::new(config);
/// client.connect().await?;
///
/// while let Some(msg) = client.next().await {
///     println!("Received: {:?}", msg);
/// }
/// ```
#[derive(Debug)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.oanda")
)]
pub struct OANDAStreamClient {
    /// Configuration for the stream.
    config: StreamConfig,

    /// HTTP client for the streaming connection.
    http_client: Client,

    /// Current connection state.
    state: StreamState,

    /// Statistics for this stream.
    stats: StreamStats,

    /// Whether the client should be running.
    running: Arc<AtomicBool>,

    /// Channel receiver for stream messages (wrapped for Clone).
    receiver: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<Result<StreamMessage, StreamError>>>>>,
}

impl Clone for OANDAStreamClient {
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

impl OANDAStreamClient {
    /// Create a new OANDA streaming client.
    pub fn new(config: StreamConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(MESSAGE_TIMEOUT_SECS + 10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            config,
            http_client,
            state: StreamState::Disconnected,
            stats: StreamStats::default(),
            running: Arc::new(AtomicBool::new(false)),
            receiver: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Get the list of subscribed instruments.
    #[must_use]
    pub fn instruments(&self) -> &[String] {
        &self.config.instruments
    }

    /// Get the current connection state.
    #[must_use]
    pub fn state(&self) -> StreamState {
        self.state
    }

    /// Get a reference to the stream statistics.
    #[must_use]
    pub fn stats(&self) -> &StreamStats {
        &self.stats
    }

    /// Check if the stream is connected.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.state == StreamState::Connected
    }

    /// Connect to the OANDA streaming API.
    ///
    /// This starts the streaming connection in a background task and returns
    /// immediately. Use `next()` to receive messages.
    pub async fn connect(&mut self) -> Result<(), StreamError> {
        // Validate configuration
        self.config.validate()?;

        if self.state == StreamState::Connected {
            return Ok(());
        }

        self.state = StreamState::Connecting;
        self.running.store(true, Ordering::SeqCst);

        let (tx, rx) = mpsc::channel(1000);
        {
            let mut receiver = self.receiver.lock().await;
            *receiver = Some(rx);
        }

        // Clone values needed for the spawned task
        let config = self.config.clone();
        let http_client = self.http_client.clone();
        let running = self.running.clone();

        // Spawn the streaming task
        tokio::spawn(async move {
            stream_task(config, http_client, tx, running).await;
        });

        // Wait a bit for the connection to establish
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.state = StreamState::Connected;
        info!(
            "OANDA stream connected for instruments: {:?}",
            self.config.instruments
        );

        Ok(())
    }

    /// Disconnect from the streaming API.
    pub async fn disconnect(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.state = StreamState::Closed;
        {
            let mut receiver = self.receiver.lock().await;
            *receiver = None;
        }
        info!("OANDA stream disconnected");
    }

    /// Get the next message from the stream.
    ///
    /// Returns `None` if the stream is closed or disconnected.
    pub async fn next(&mut self) -> Option<Result<StreamMessage, StreamError>> {
        let mut receiver = self.receiver.lock().await;
        if let Some(ref mut rx) = *receiver {
            match rx.recv().await {
                Some(Ok(msg)) => {
                    // Update statistics
                    match &msg {
                        StreamMessage::Price(p) => self.stats.record_price(&p.time),
                        StreamMessage::Heartbeat(h) => self.stats.record_heartbeat(&h.time),
                    }
                    Some(Ok(msg))
                }
                Some(Err(e)) => {
                    if matches!(e, StreamError::Closed) {
                        self.state = StreamState::Closed;
                        None
                    } else {
                        Some(Err(e))
                    }
                }
                None => {
                    self.state = StreamState::Disconnected;
                    None
                }
            }
        } else {
            None
        }
    }

    /// Subscribe to additional instruments (reconnects if necessary).
    pub async fn subscribe(&mut self, instruments: Vec<String>) -> Result<(), StreamError> {
        for inst in instruments {
            if !self.config.instruments.contains(&inst) {
                self.config.instruments.push(inst);
            }
        }

        // Validate new configuration
        self.config.validate()?;

        // Reconnect with new instruments
        if self.state == StreamState::Connected {
            self.disconnect().await;
            self.connect().await?;
        }

        Ok(())
    }

    /// Unsubscribe from instruments.
    pub async fn unsubscribe(&mut self, instruments: Vec<String>) -> Result<(), StreamError> {
        self.config
            .instruments
            .retain(|i| !instruments.contains(i));

        if self.config.instruments.is_empty() {
            self.disconnect().await;
            return Err(StreamError::Configuration(
                "No instruments remaining".into(),
            ));
        }

        // Reconnect with remaining instruments
        if self.state == StreamState::Connected {
            self.disconnect().await;
            self.connect().await?;
        }

        Ok(())
    }
}

impl Drop for OANDAStreamClient {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Background task that handles the streaming connection.
async fn stream_task(
    config: StreamConfig,
    http_client: Client,
    tx: mpsc::Sender<Result<StreamMessage, StreamError>>,
    running: Arc<AtomicBool>,
) {
    let mut reconnect_count = 0u32;

    while running.load(Ordering::SeqCst) {
        match connect_and_stream(&config, &http_client, &tx, &running).await {
            Ok(()) => {
                // Clean exit
                debug!("Stream ended cleanly");
                break;
            }
            Err(e) => {
                error!("Stream error: {}", e);

                if !running.load(Ordering::SeqCst) {
                    break;
                }

                if !config.reconnect_on_error {
                    let _ = tx.send(Err(e)).await;
                    break;
                }

                reconnect_count += 1;

                if config.max_reconnect_attempts > 0
                    && reconnect_count >= config.max_reconnect_attempts
                {
                    let _ = tx
                        .send(Err(StreamError::MaxReconnects(reconnect_count)))
                        .await;
                    break;
                }

                warn!(
                    "Reconnecting in {}ms (attempt {}/{})",
                    config.reconnect_delay_ms,
                    reconnect_count,
                    if config.max_reconnect_attempts > 0 {
                        config.max_reconnect_attempts.to_string()
                    } else {
                        "∞".to_string()
                    }
                );

                tokio::time::sleep(Duration::from_millis(config.reconnect_delay_ms)).await;
            }
        }
    }

    // Signal stream closed
    let _ = tx.send(Err(StreamError::Closed)).await;
}

/// Connect to OANDA and stream messages.
async fn connect_and_stream(
    config: &StreamConfig,
    http_client: &Client,
    tx: &mpsc::Sender<Result<StreamMessage, StreamError>>,
    running: &Arc<AtomicBool>,
) -> Result<(), StreamError> {
    let url = config.streaming_url();
    debug!("Connecting to streaming URL: {}", url);

    // Make the streaming request
    let response = http_client
        .get(&url)
        .bearer_auth(&config.api_key)
        .header("Accept-Datetime-Format", "RFC3339")
        .send()
        .await
        .map_err(|e| StreamError::Connection(e.to_string()))?;

    // Check response status
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        if status.as_u16() == 401 {
            return Err(StreamError::Authentication(body));
        }

        return Err(StreamError::Http {
            status: status.as_u16(),
            message: body,
        });
    }

    info!("OANDA stream connected successfully");

    // Read the streaming response
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while running.load(Ordering::SeqCst) {
        // Read with timeout
        let chunk_result = timeout(
            Duration::from_secs(MESSAGE_TIMEOUT_SECS),
            stream.next(),
        )
        .await;

        match chunk_result {
            Ok(Some(Ok(chunk))) => {
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
                    match serde_json::from_str::<RawStreamMessage>(&line) {
                        Ok(raw_msg) => {
                            let msg: StreamMessage = raw_msg.into();

                            // Filter heartbeats if configured
                            if !config.include_heartbeats
                                && matches!(msg, StreamMessage::Heartbeat(_))
                            {
                                continue;
                            }

                            if tx.send(Ok(msg)).await.is_err() {
                                // Receiver dropped
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse stream message: {} - line: {}", e, line);
                        }
                    }
                }
            }
            Ok(Some(Err(e))) => {
                return Err(StreamError::Connection(e.to_string()));
            }
            Ok(None) => {
                // Stream ended
                return Err(StreamError::Disconnected("Stream ended".into()));
            }
            Err(_) => {
                // Timeout
                return Err(StreamError::Timeout(MESSAGE_TIMEOUT_SECS));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::enums::OANDAEnvironment;

    #[test]
    fn test_client_creation() {
        let config = StreamConfig::new(
            OANDAEnvironment::Practice,
            "test-api-key",
            "test-account",
            vec!["EUR_USD".into()],
        );

        let client = OANDAStreamClient::new(config);
        assert_eq!(client.state(), StreamState::Disconnected);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_client_creation_live() {
        let config = StreamConfig::new(
            OANDAEnvironment::Live,
            "test-api-key",
            "test-account",
            vec!["EUR_USD".into()],
        );

        let client = OANDAStreamClient::new(config);
        assert_eq!(client.state(), StreamState::Disconnected);
        
        // Verify it's configured for live environment
        let url = client.config.streaming_url();
        assert!(url.contains("stream-fxtrade.oanda.com"));
    }

    #[test]
    fn test_client_stats() {
        let config = StreamConfig::default();
        let client = OANDAStreamClient::new(config);

        let stats = client.stats();
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.reconnections, 0);
        assert_eq!(stats.prices_received, 0);
        assert_eq!(stats.heartbeats_received, 0);
    }

    #[test]
    fn test_client_multiple_instruments() {
        let config = StreamConfig::new(
            OANDAEnvironment::Practice,
            "test-api-key",
            "test-account",
            vec!["EUR_USD".into(), "GBP_USD".into(), "USD_JPY".into()],
        );

        let client = OANDAStreamClient::new(config);
        let url = client.config.streaming_url();
        
        // All instruments should be in the URL
        assert!(url.contains("EUR_USD"));
        assert!(url.contains("GBP_USD"));
        assert!(url.contains("USD_JPY"));
    }

    #[test]
    fn test_stream_state_debug() {
        // StreamState should have Debug trait
        assert_eq!(format!("{:?}", StreamState::Disconnected), "Disconnected");
        assert_eq!(format!("{:?}", StreamState::Connecting), "Connecting");
        assert_eq!(format!("{:?}", StreamState::Connected), "Connected");
        assert_eq!(format!("{:?}", StreamState::Reconnecting), "Reconnecting");
    }

    #[test]
    fn test_stream_error_display() {
        let config_err = StreamError::Configuration("missing api key".into());
        assert!(config_err.to_string().contains("missing api key"));

        let conn_err = StreamError::Connection("network failure".into());
        assert!(conn_err.to_string().contains("network failure"));

        let timeout_err = StreamError::Timeout(30);
        assert!(timeout_err.to_string().contains("30"));

        let parse_err = StreamError::Parse("invalid json".into());
        assert!(parse_err.to_string().contains("invalid json"));

        let disc_err = StreamError::Disconnected("server closed".into());
        assert!(disc_err.to_string().contains("server closed"));

        let auth_err = StreamError::Authentication("invalid token".into());
        assert!(auth_err.to_string().contains("invalid token"));

        let http_err = StreamError::Http { status: 401, message: "Unauthorized".into() };
        assert!(http_err.to_string().contains("401"));

        let closed_err = StreamError::Closed;
        assert!(closed_err.to_string().contains("closed"));

        let max_err = StreamError::MaxReconnects(5);
        assert!(max_err.to_string().contains("5"));
    }

    #[test]
    fn test_client_config_access() {
        let config = StreamConfig::new(
            OANDAEnvironment::Practice,
            "my-api-key",
            "my-account",
            vec!["EUR_USD".into()],
        );

        let client = OANDAStreamClient::new(config);
        
        // Config should be accessible
        assert_eq!(client.config.account_id, "my-account");
        assert_eq!(client.config.instruments, vec!["EUR_USD"]);
        assert!(client.config.include_heartbeats);
    }

    #[test]
    fn test_client_state_equality() {
        assert_eq!(StreamState::Disconnected, StreamState::Disconnected);
        assert_ne!(StreamState::Disconnected, StreamState::Connected);
        assert_ne!(StreamState::Connecting, StreamState::Reconnecting);
    }
}
