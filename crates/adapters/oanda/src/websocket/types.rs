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

//! Types for OANDA streaming API.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::common::enums::OANDAEnvironment;
use crate::http::models::PriceLevel;

// ================================================================================================
// Configuration
// ================================================================================================

/// Configuration for OANDA streaming client.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Trading environment (Practice or Live).
    pub environment: OANDAEnvironment,

    /// OANDA API key for authentication.
    pub api_key: String,

    /// OANDA account ID.
    pub account_id: String,

    /// Instruments to subscribe to (e.g., ["EUR_USD", "GBP_USD"]).
    /// Maximum 20 instruments per stream.
    pub instruments: Vec<String>,

    /// Whether to include heartbeat messages (default: true).
    pub include_heartbeats: bool,

    /// Snapshot mode - if true, only latest price is returned when catching up.
    pub snapshot: bool,

    /// Reconnection settings.
    pub reconnect_on_error: bool,

    /// Maximum reconnection attempts (0 = unlimited).
    pub max_reconnect_attempts: u32,

    /// Delay between reconnection attempts in milliseconds.
    pub reconnect_delay_ms: u64,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            environment: OANDAEnvironment::Practice,
            api_key: String::new(),
            account_id: String::new(),
            instruments: Vec::new(),
            include_heartbeats: true,
            snapshot: true,
            reconnect_on_error: true,
            max_reconnect_attempts: 10,
            reconnect_delay_ms: 1000,
        }
    }
}

impl StreamConfig {
    /// Create a new stream configuration.
    pub fn new(
        environment: OANDAEnvironment,
        api_key: impl Into<String>,
        account_id: impl Into<String>,
        instruments: Vec<String>,
    ) -> Self {
        Self {
            environment,
            api_key: api_key.into(),
            account_id: account_id.into(),
            instruments,
            ..Default::default()
        }
    }

    /// Get the streaming URL for this configuration.
    #[must_use]
    pub fn streaming_url(&self) -> String {
        let base = match self.environment {
            OANDAEnvironment::Practice => "https://stream-fxpractice.oanda.com",
            OANDAEnvironment::Live => "https://stream-fxtrade.oanda.com",
        };

        let instruments = self.instruments.join(",");
        format!(
            "{}/v3/accounts/{}/pricing/stream?instruments={}",
            base, self.account_id, instruments
        )
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), StreamError> {
        if self.api_key.is_empty() {
            return Err(StreamError::Configuration("API key is required".into()));
        }
        if self.account_id.is_empty() {
            return Err(StreamError::Configuration("Account ID is required".into()));
        }
        if self.instruments.is_empty() {
            return Err(StreamError::Configuration(
                "At least one instrument is required".into(),
            ));
        }
        if self.instruments.len() > 20 {
            return Err(StreamError::Configuration(
                "Maximum 20 instruments per stream".into(),
            ));
        }
        Ok(())
    }
}

// ================================================================================================
// Stream Messages
// ================================================================================================

/// Message received from OANDA streaming API.
#[derive(Debug, Clone)]
pub enum StreamMessage {
    /// Price update for an instrument.
    Price(StreamPrice),

    /// Heartbeat message (connection keepalive).
    Heartbeat(StreamHeartbeat),
}

/// Raw message from OANDA streaming API (for deserialization).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum RawStreamMessage {
    /// Price update.
    #[serde(rename = "PRICE")]
    Price(RawStreamPrice),

    /// Heartbeat.
    #[serde(rename = "HEARTBEAT")]
    Heartbeat(StreamHeartbeat),
}

impl From<RawStreamMessage> for StreamMessage {
    fn from(raw: RawStreamMessage) -> Self {
        match raw {
            RawStreamMessage::Price(p) => StreamMessage::Price(p.into()),
            RawStreamMessage::Heartbeat(h) => StreamMessage::Heartbeat(h),
        }
    }
}

/// Raw price message from OANDA (string values).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawStreamPrice {
    /// Instrument name (e.g., "EUR_USD").
    pub instrument: String,

    /// Price timestamp (RFC3339 format).
    pub time: String,

    /// Whether the instrument is currently tradeable.
    pub tradeable: bool,

    /// Bid prices (best bid first).
    #[serde(default)]
    pub bids: Vec<PriceLevel>,

    /// Ask prices (best ask first).
    #[serde(default)]
    pub asks: Vec<PriceLevel>,

    /// Closeout bid price.
    #[serde(default)]
    pub closeout_bid: Option<String>,

    /// Closeout ask price.
    #[serde(default)]
    pub closeout_ask: Option<String>,

    /// Price status.
    #[serde(default)]
    pub status: Option<String>,
}

/// Processed streaming price with parsed values.
#[derive(Debug, Clone)]
pub struct StreamPrice {
    /// Instrument name (e.g., "EUR_USD").
    pub instrument: String,

    /// Price timestamp (RFC3339 format).
    pub time: String,

    /// Whether the instrument is currently tradeable.
    pub tradeable: bool,

    /// Best bid price (None if no bids).
    pub bid: Option<f64>,

    /// Best ask price (None if no asks).
    pub ask: Option<f64>,

    /// Bid liquidity at best price.
    pub bid_liquidity: Option<i64>,

    /// Ask liquidity at best price.
    pub ask_liquidity: Option<i64>,

    /// All bid levels.
    pub bids: Vec<PriceLevel>,

    /// All ask levels.
    pub asks: Vec<PriceLevel>,

    /// Closeout bid price.
    pub closeout_bid: Option<f64>,

    /// Closeout ask price.
    pub closeout_ask: Option<f64>,

    /// Mid price (average of bid and ask).
    pub mid: Option<f64>,

    /// Spread in pips (ask - bid).
    pub spread: Option<f64>,
}

impl From<RawStreamPrice> for StreamPrice {
    fn from(raw: RawStreamPrice) -> Self {
        let bid = raw
            .bids
            .first()
            .and_then(|p| p.price.parse::<f64>().ok());
        let ask = raw
            .asks
            .first()
            .and_then(|p| p.price.parse::<f64>().ok());
        let bid_liquidity = raw.bids.first().map(|p| p.liquidity);
        let ask_liquidity = raw.asks.first().map(|p| p.liquidity);
        let closeout_bid = raw.closeout_bid.as_ref().and_then(|s| s.parse::<f64>().ok());
        let closeout_ask = raw.closeout_ask.as_ref().and_then(|s| s.parse::<f64>().ok());

        let mid = match (bid, ask) {
            (Some(b), Some(a)) => Some((b + a) / 2.0),
            _ => None,
        };

        let spread = match (bid, ask) {
            (Some(b), Some(a)) => Some(a - b),
            _ => None,
        };

        Self {
            instrument: raw.instrument,
            time: raw.time,
            tradeable: raw.tradeable,
            bid,
            ask,
            bid_liquidity,
            ask_liquidity,
            bids: raw.bids,
            asks: raw.asks,
            closeout_bid,
            closeout_ask,
            mid,
            spread,
        }
    }
}

/// Heartbeat message from OANDA streaming API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamHeartbeat {
    /// Heartbeat timestamp (RFC3339 format).
    pub time: String,
}

// ================================================================================================
// Transaction Streaming Types
// ================================================================================================

/// Configuration for OANDA transaction streaming client.
#[derive(Debug, Clone)]
pub struct TransactionStreamConfig {
    /// Trading environment (Practice or Live).
    pub environment: OANDAEnvironment,

    /// OANDA API key for authentication.
    pub api_key: String,

    /// OANDA account ID.
    pub account_id: String,

    /// Whether to include heartbeat messages (default: true).
    pub include_heartbeats: bool,

    /// Reconnection settings.
    pub reconnect_on_error: bool,

    /// Maximum reconnection attempts (0 = unlimited).
    pub max_reconnect_attempts: u32,

    /// Delay between reconnection attempts in milliseconds.
    pub reconnect_delay_ms: u64,
}

impl Default for TransactionStreamConfig {
    fn default() -> Self {
        Self {
            environment: OANDAEnvironment::Practice,
            api_key: String::new(),
            account_id: String::new(),
            include_heartbeats: true,
            reconnect_on_error: true,
            max_reconnect_attempts: 10,
            reconnect_delay_ms: 1000,
        }
    }
}

impl TransactionStreamConfig {
    /// Create a new transaction stream configuration.
    pub fn new(
        environment: OANDAEnvironment,
        api_key: impl Into<String>,
        account_id: impl Into<String>,
    ) -> Self {
        Self {
            environment,
            api_key: api_key.into(),
            account_id: account_id.into(),
            ..Default::default()
        }
    }

    /// Get the transaction streaming URL for this configuration.
    #[must_use]
    pub fn streaming_url(&self) -> String {
        let base = match self.environment {
            OANDAEnvironment::Practice => "https://stream-fxpractice.oanda.com",
            OANDAEnvironment::Live => "https://stream-fxtrade.oanda.com",
        };

        format!(
            "{}/v3/accounts/{}/transactions/stream",
            base, self.account_id
        )
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), StreamError> {
        if self.api_key.is_empty() {
            return Err(StreamError::Configuration("API key is required".into()));
        }
        if self.account_id.is_empty() {
            return Err(StreamError::Configuration("Account ID is required".into()));
        }
        Ok(())
    }
}

/// Message received from OANDA transaction streaming API.
#[derive(Debug, Clone)]
pub enum TransactionStreamMessage {
    /// Transaction event.
    Transaction(StreamTransaction),

    /// Heartbeat message (connection keepalive).
    Heartbeat(StreamHeartbeat),
}

/// Raw message from OANDA transaction streaming API (for deserialization).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RawTransactionStreamMessage {
    /// Heartbeat message.
    Heartbeat(TransactionHeartbeat),

    /// Transaction message.
    Transaction(RawStreamTransaction),
}

/// Heartbeat for transaction stream (different format than price stream).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHeartbeat {
    /// Message type (should be "HEARTBEAT").
    #[serde(rename = "type")]
    pub msg_type: String,

    /// Last transaction ID at heartbeat time.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: String,

    /// Heartbeat timestamp (RFC3339 format).
    pub time: String,
}

/// Raw transaction from OANDA streaming API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawStreamTransaction {
    /// Transaction ID.
    pub id: String,

    /// Account ID.
    #[serde(rename = "accountID")]
    pub account_id: String,

    /// Transaction time (RFC3339 format).
    pub time: String,

    /// Transaction type (ORDER_FILL, ORDER_CANCEL, STOP_LOSS_ORDER, etc.).
    #[serde(rename = "type")]
    pub transaction_type: String,

    // Order-related fields
    /// Order ID (for order transactions).
    #[serde(default, rename = "orderID")]
    pub order_id: Option<String>,

    /// Trade ID (for trade-related transactions).
    #[serde(default, rename = "tradeID")]
    pub trade_id: Option<String>,

    /// Instrument.
    #[serde(default)]
    pub instrument: Option<String>,

    /// Units.
    #[serde(default)]
    pub units: Option<String>,

    /// Price.
    #[serde(default)]
    pub price: Option<String>,

    /// Profit/Loss.
    #[serde(default)]
    pub pl: Option<String>,

    /// Reason for the transaction.
    #[serde(default)]
    pub reason: Option<String>,

    /// Account balance after transaction.
    #[serde(default)]
    pub account_balance: Option<String>,

    /// Time in force.
    #[serde(default)]
    pub time_in_force: Option<String>,

    /// Trade opened ID (for fills that open new trades).
    #[serde(default, rename = "tradeOpened")]
    pub trade_opened: Option<TradeOpened>,

    /// Trade reduced (for partial closes).
    #[serde(default, rename = "tradeReduced")]
    pub trade_reduced: Option<TradeReduced>,

    /// Trades closed (for full closes).
    #[serde(default, rename = "tradesClosed")]
    pub trades_closed: Option<Vec<TradeClosed>>,

    /// Full price (for order fills).
    #[serde(default, rename = "fullPrice")]
    pub full_price: Option<serde_json::Value>,
}

/// Trade opened in a fill transaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOpened {
    /// Trade ID.
    #[serde(rename = "tradeID")]
    pub trade_id: String,

    /// Units opened.
    pub units: String,

    /// Initial margin required.
    #[serde(default)]
    pub initial_margin_required: Option<String>,
}

/// Trade reduced in a fill transaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeReduced {
    /// Trade ID.
    #[serde(rename = "tradeID")]
    pub trade_id: String,

    /// Units reduced.
    pub units: String,

    /// Realized P/L.
    #[serde(default, rename = "realizedPL")]
    pub realized_pl: Option<String>,
}

/// Trade closed in a fill transaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeClosed {
    /// Trade ID.
    #[serde(rename = "tradeID")]
    pub trade_id: String,

    /// Units closed.
    pub units: String,

    /// Realized P/L.
    #[serde(default, rename = "realizedPL")]
    pub realized_pl: Option<String>,
}

/// Processed streaming transaction.
#[derive(Debug, Clone)]
pub struct StreamTransaction {
    /// Transaction ID.
    pub id: String,

    /// Account ID.
    pub account_id: String,

    /// Transaction time (RFC3339 format).
    pub time: String,

    /// Transaction type.
    pub transaction_type: TransactionType,

    /// Order ID (if applicable).
    pub order_id: Option<String>,

    /// Trade ID (if applicable).
    pub trade_id: Option<String>,

    /// Instrument (if applicable).
    pub instrument: Option<String>,

    /// Units (if applicable).
    pub units: Option<f64>,

    /// Price (if applicable).
    pub price: Option<f64>,

    /// Profit/Loss (if applicable).
    pub pl: Option<f64>,

    /// Reason for the transaction.
    pub reason: Option<String>,

    /// Account balance after transaction.
    pub account_balance: Option<f64>,

    /// Raw transaction data for full access.
    pub raw: RawStreamTransaction,
}

/// Transaction types from OANDA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    /// Market order fill.
    OrderFill,

    /// Order cancelled.
    OrderCancel,

    /// Market order created.
    MarketOrder,

    /// Limit order created.
    LimitOrder,

    /// Stop order created.
    StopOrder,

    /// Take profit order created.
    TakeProfitOrder,

    /// Stop loss order created.
    StopLossOrder,

    /// Trailing stop loss order created.
    TrailingStopLossOrder,

    /// Market-if-touched order created.
    MarketIfTouchedOrder,

    /// Trade client extensions modified.
    TradeClientExtensionsModify,

    /// Order client extensions modified.
    OrderClientExtensionsModify,

    /// Daily financing.
    DailyFinancing,

    /// Dividend adjustment.
    DividendAdjustment,

    /// Transfer funds.
    TransferFunds,

    /// Reset resettable PL.
    ResetResettablePL,

    /// Unknown transaction type.
    Unknown,
}

impl From<&str> for TransactionType {
    fn from(s: &str) -> Self {
        match s {
            "ORDER_FILL" => TransactionType::OrderFill,
            "ORDER_CANCEL" => TransactionType::OrderCancel,
            "MARKET_ORDER" => TransactionType::MarketOrder,
            "LIMIT_ORDER" => TransactionType::LimitOrder,
            "STOP_ORDER" => TransactionType::StopOrder,
            "TAKE_PROFIT_ORDER" => TransactionType::TakeProfitOrder,
            "STOP_LOSS_ORDER" => TransactionType::StopLossOrder,
            "TRAILING_STOP_LOSS_ORDER" => TransactionType::TrailingStopLossOrder,
            "MARKET_IF_TOUCHED_ORDER" => TransactionType::MarketIfTouchedOrder,
            "TRADE_CLIENT_EXTENSIONS_MODIFY" => TransactionType::TradeClientExtensionsModify,
            "ORDER_CLIENT_EXTENSIONS_MODIFY" => TransactionType::OrderClientExtensionsModify,
            "DAILY_FINANCING" => TransactionType::DailyFinancing,
            "DIVIDEND_ADJUSTMENT" => TransactionType::DividendAdjustment,
            "TRANSFER_FUNDS" => TransactionType::TransferFunds,
            "RESET_RESETTABLE_PL" => TransactionType::ResetResettablePL,
            _ => TransactionType::Unknown,
        }
    }
}

impl From<RawStreamTransaction> for StreamTransaction {
    fn from(raw: RawStreamTransaction) -> Self {
        let transaction_type = TransactionType::from(raw.transaction_type.as_str());
        let units = raw.units.as_ref().and_then(|s| s.parse::<f64>().ok());
        let price = raw.price.as_ref().and_then(|s| s.parse::<f64>().ok());
        let pl = raw.pl.as_ref().and_then(|s| s.parse::<f64>().ok());
        let account_balance = raw.account_balance.as_ref().and_then(|s| s.parse::<f64>().ok());

        Self {
            id: raw.id.clone(),
            account_id: raw.account_id.clone(),
            time: raw.time.clone(),
            transaction_type,
            order_id: raw.order_id.clone(),
            trade_id: raw.trade_id.clone(),
            instrument: raw.instrument.clone(),
            units,
            price,
            pl,
            reason: raw.reason.clone(),
            account_balance,
            raw,
        }
    }
}

impl From<RawTransactionStreamMessage> for TransactionStreamMessage {
    fn from(raw: RawTransactionStreamMessage) -> Self {
        match raw {
            RawTransactionStreamMessage::Heartbeat(h) => {
                TransactionStreamMessage::Heartbeat(StreamHeartbeat { time: h.time })
            }
            RawTransactionStreamMessage::Transaction(t) => {
                TransactionStreamMessage::Transaction(t.into())
            }
        }
    }
}

// ================================================================================================
// Errors
// ================================================================================================

/// Errors that can occur during streaming.
#[derive(Debug, Error)]
pub enum StreamError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// HTTP error from OANDA.
    #[error("HTTP error: {status} - {message}")]
    Http { status: u16, message: String },

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Failed to parse message.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Stream disconnected.
    #[error("Stream disconnected: {0}")]
    Disconnected(String),

    /// Timeout waiting for message.
    #[error("Timeout: no message received in {0} seconds")]
    Timeout(u64),

    /// Maximum reconnection attempts exceeded.
    #[error("Max reconnection attempts ({0}) exceeded")]
    MaxReconnects(u32),

    /// Stream was closed.
    #[error("Stream closed")]
    Closed,

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ================================================================================================
// Stream State
// ================================================================================================

/// Current state of the streaming connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Not connected.
    Disconnected,

    /// Connecting to OANDA.
    Connecting,

    /// Connected and receiving messages.
    Connected,

    /// Reconnecting after a disconnection.
    Reconnecting,

    /// Stream has been closed.
    Closed,

    /// Stream encountered an error.
    Error,
}

impl Default for StreamState {
    fn default() -> Self {
        Self::Disconnected
    }
}

// ================================================================================================
// Stream Statistics
// ================================================================================================

/// Statistics for the streaming connection.
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    /// Total messages received.
    pub messages_received: u64,

    /// Price messages received.
    pub prices_received: u64,

    /// Heartbeat messages received.
    pub heartbeats_received: u64,

    /// Parse errors encountered.
    pub parse_errors: u64,

    /// Number of reconnections.
    pub reconnections: u32,

    /// Timestamp of last message received.
    pub last_message_time: Option<String>,

    /// Timestamp of connection start.
    pub connected_at: Option<String>,
}

impl StreamStats {
    /// Record a price message.
    pub fn record_price(&mut self, time: &str) {
        self.messages_received += 1;
        self.prices_received += 1;
        self.last_message_time = Some(time.to_string());
    }

    /// Record a heartbeat message.
    pub fn record_heartbeat(&mut self, time: &str) {
        self.messages_received += 1;
        self.heartbeats_received += 1;
        self.last_message_time = Some(time.to_string());
    }

    /// Record a parse error.
    pub fn record_parse_error(&mut self) {
        self.parse_errors += 1;
    }

    /// Record a reconnection.
    pub fn record_reconnection(&mut self) {
        self.reconnections += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.environment, OANDAEnvironment::Practice);
        assert!(config.include_heartbeats);
        assert!(config.reconnect_on_error);
    }

    #[test]
    fn test_stream_config_url() {
        let config = StreamConfig::new(
            OANDAEnvironment::Practice,
            "api-key",
            "account-123",
            vec!["EUR_USD".into(), "GBP_USD".into()],
        );

        let url = config.streaming_url();
        assert!(url.contains("stream-fxpractice.oanda.com"));
        assert!(url.contains("account-123"));
        assert!(url.contains("EUR_USD,GBP_USD"));
    }

    #[test]
    fn test_stream_config_validation() {
        let mut config = StreamConfig::default();

        // Missing API key
        assert!(config.validate().is_err());

        config.api_key = "test".into();
        // Missing account ID
        assert!(config.validate().is_err());

        config.account_id = "test".into();
        // Missing instruments
        assert!(config.validate().is_err());

        config.instruments = vec!["EUR_USD".into()];
        // Should pass now
        assert!(config.validate().is_ok());

        // Too many instruments
        config.instruments = (0..25).map(|i| format!("INST_{}", i)).collect();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_stream_price_conversion() {
        let raw = RawStreamPrice {
            instrument: "EUR_USD".into(),
            time: "2026-01-06T12:00:00.000000000Z".into(),
            tradeable: true,
            bids: vec![PriceLevel {
                price: "1.08500".into(),
                liquidity: 1000000,
            }],
            asks: vec![PriceLevel {
                price: "1.08510".into(),
                liquidity: 1000000,
            }],
            closeout_bid: Some("1.08495".into()),
            closeout_ask: Some("1.08515".into()),
            status: None,
        };

        let price: StreamPrice = raw.into();

        assert_eq!(price.instrument, "EUR_USD");
        assert!(price.tradeable);
        assert!((price.bid.unwrap() - 1.08500).abs() < 0.00001);
        assert!((price.ask.unwrap() - 1.08510).abs() < 0.00001);
        assert!((price.mid.unwrap() - 1.08505).abs() < 0.00001);
        assert!((price.spread.unwrap() - 0.00010).abs() < 0.00001);
    }

    #[test]
    fn test_stream_stats() {
        let mut stats = StreamStats::default();

        stats.record_price("2026-01-06T12:00:00Z");
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.prices_received, 1);

        stats.record_heartbeat("2026-01-06T12:00:05Z");
        assert_eq!(stats.messages_received, 2);
        assert_eq!(stats.heartbeats_received, 1);

        stats.record_reconnection();
        assert_eq!(stats.reconnections, 1);
    }
}
