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

//! Response models for the OANDA REST API v3.
//!
//! All numeric values are represented as strings as returned by the API.
//! Use parse methods to convert to Decimal when needed.
//!
//! # Model Coverage
//!
//! These models are based on the OANDA v20 API specification. Key references:
//! - Account: Full account details including margin, P&L, and trading state
//! - Trade: Open trade representation with dependent orders
//! - Position: Instrument position with long/short sides
//! - Order: Order creation and fill transactions
//! - Candle: OHLC price data at various granularities
//!
//! # Notes on Field Types
//!
//! OANDA returns most numeric values as strings to preserve precision.
//! Fields like `balance`, `pl`, `price` are strings that should be parsed
//! to `Decimal` for calculations.
//!
//! # Reference
//!
//! Model structure informed by oanda-v20-openapi crate (MIT license).
//! See: <https://github.com/jxcv0/oanda-v20-openapi>

use serde::{Deserialize, Serialize};

use crate::common::enums::{
    OANDAGranularity, OANDAGuaranteedStopLossOrderMode, OANDAInstrumentType, OANDAOrderState,
    OANDATradeState,
};

// ================================================================================================
// Account Models
// ================================================================================================

/// OANDA account details response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    /// The account details.
    pub account: AccountDetails,
}

/// OANDA account details.
///
/// The full details of a client's Account including open Trade,
/// open Position and pending Order representation.
///
/// Reference: OANDA v20 API Account object
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetails {
    /// The account ID.
    pub id: String,

    /// The account alias (name).
    #[serde(default)]
    pub alias: Option<String>,

    /// The account currency (e.g., "USD", "EUR").
    pub currency: String,

    /// ID of the user that created the Account.
    #[serde(default, rename = "createdByUserID")]
    pub created_by_user_id: Option<i32>,

    /// The date/time when the Account was created (RFC3339 format).
    #[serde(default)]
    pub created_time: Option<String>,

    /// The current guaranteed Stop Loss Order mode of the Account.
    #[serde(default)]
    pub guaranteed_stop_loss_order_mode: Option<OANDAGuaranteedStopLossOrderMode>,

    /// The current balance of the Account.
    pub balance: String,

    /// The total profit/loss realized over the lifetime of the Account.
    #[serde(default)]
    pub pl: Option<String>,

    /// The total realized profit/loss for the Account since it was last reset by the client.
    #[serde(default, rename = "resettablePL")]
    pub resettable_pl: Option<String>,

    /// The total amount of financing paid/collected over the lifetime of the Account.
    #[serde(default)]
    pub financing: Option<String>,

    /// The total amount of commission paid over the lifetime of the Account.
    #[serde(default)]
    pub commission: Option<String>,

    /// The total amount of fees charged for guaranteed Stop Loss Orders.
    #[serde(default)]
    pub guaranteed_execution_fees: Option<String>,

    /// Client-provided margin rate override for the Account.
    #[serde(default)]
    pub margin_rate: Option<String>,

    /// The date/time when the Account entered a margin call state (RFC3339 format).
    #[serde(default)]
    pub margin_call_enter_time: Option<String>,

    /// The number of times the Account's margin call was extended.
    #[serde(default)]
    pub margin_call_extension_count: Option<i32>,

    /// The date/time of the Account's last margin call extension (RFC3339 format).
    #[serde(default)]
    pub last_margin_call_extension_time: Option<String>,

    /// Unrealized profit/loss for all open Trades.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: String,

    /// Net asset value (balance + unrealizedPL).
    #[serde(rename = "NAV")]
    pub nav: String,

    /// Margin currently used by the Account.
    pub margin_used: String,

    /// Margin available for Account currency.
    pub margin_available: String,

    /// The value of the Account's open positions in home currency.
    pub position_value: String,

    /// The Account's margin closeout unrealized P&L.
    #[serde(default, rename = "marginCloseoutUnrealizedPL")]
    pub margin_closeout_unrealized_pl: Option<String>,

    /// The Account's margin closeout NAV.
    #[serde(default, rename = "marginCloseoutNAV")]
    pub margin_closeout_nav: Option<String>,

    /// The Account's margin closeout margin used.
    #[serde(default)]
    pub margin_closeout_margin_used: Option<String>,

    /// The Account's margin closeout percentage (1.0 = margin call).
    pub margin_closeout_percent: String,

    /// The value of the Account's open positions for margin closeout.
    #[serde(default)]
    pub margin_closeout_position_value: Option<String>,

    /// The current WithdrawalLimit for the Account.
    #[serde(default)]
    pub withdrawal_limit: Option<String>,

    /// The Account's margin call margin used.
    #[serde(default)]
    pub margin_call_margin_used: Option<String>,

    /// The Account's margin call percentage (1.0 = margin call).
    #[serde(default)]
    pub margin_call_percent: Option<String>,

    /// Number of open trades.
    pub open_trade_count: i32,

    /// Number of open positions.
    pub open_position_count: i32,

    /// Number of pending orders.
    pub pending_order_count: i32,

    /// Whether hedging is enabled.
    pub hedging_enabled: bool,

    /// The date/time of the last order fill (RFC3339 format).
    #[serde(default)]
    pub last_order_fill_timestamp: Option<String>,

    /// The ID of the last Transaction created for the Account.
    #[serde(default, rename = "lastTransactionID")]
    pub last_transaction_id: Option<String>,

    /// The details of Trades currently open in the Account.
    #[serde(default)]
    pub trades: Option<Vec<TradeSummary>>,

    /// The details all Account Positions.
    #[serde(default)]
    pub positions: Option<Vec<Position>>,

    /// The details of Orders currently pending in the Account.
    #[serde(default)]
    pub orders: Option<Vec<Order>>,
}

/// OANDA account summary response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummaryResponse {
    /// The account summary.
    pub account: AccountSummary,
}

/// OANDA account summary.
///
/// A summary representation of a client's Account. Does not include
/// full specification of pending Orders, open Trades and Positions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    /// The account ID.
    pub id: String,

    /// The account alias (name).
    #[serde(default)]
    pub alias: Option<String>,

    /// The account currency.
    pub currency: String,

    /// The account balance.
    pub balance: String,

    /// The total profit/loss realized over the lifetime of the Account.
    #[serde(default)]
    pub pl: Option<String>,

    /// The total realized profit/loss since last reset.
    #[serde(default, rename = "resettablePL")]
    pub resettable_pl: Option<String>,

    /// The total amount of financing paid/collected.
    #[serde(default)]
    pub financing: Option<String>,

    /// The total amount of commission paid.
    #[serde(default)]
    pub commission: Option<String>,

    /// Unrealized profit/loss for all open Trades.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: String,

    /// Net asset value (balance + unrealizedPL).
    #[serde(rename = "NAV")]
    pub nav: String,

    /// Margin currently used.
    #[serde(default)]
    pub margin_used: Option<String>,

    /// Margin available.
    #[serde(default)]
    pub margin_available: Option<String>,

    /// The value of open positions in home currency.
    #[serde(default)]
    pub position_value: Option<String>,

    /// The margin closeout percentage.
    #[serde(default)]
    pub margin_closeout_percent: Option<String>,

    /// Number of open trades.
    #[serde(default)]
    pub open_trade_count: Option<i32>,

    /// Number of open positions.
    #[serde(default)]
    pub open_position_count: Option<i32>,

    /// Number of pending orders.
    #[serde(default)]
    pub pending_order_count: Option<i32>,

    /// Whether hedging is enabled.
    #[serde(default)]
    pub hedging_enabled: Option<bool>,

    /// The ID of the last Transaction.
    #[serde(default, rename = "lastTransactionID")]
    pub last_transaction_id: Option<String>,
}

// ================================================================================================
// Instrument Models
// ================================================================================================

/// OANDA instruments response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentsResponse {
    /// List of instruments.
    pub instruments: Vec<InstrumentDetails>,
}

/// OANDA instrument details.
///
/// Full specification of an instrument available for trading.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentDetails {
    /// The instrument name (e.g., "EUR_USD").
    pub name: String,

    /// The instrument type.
    #[serde(rename = "type")]
    pub instrument_type: OANDAInstrumentType,

    /// Display name (e.g., "EUR/USD").
    pub display_name: String,

    /// Pip location (e.g., -4 for 0.0001).
    pub pip_location: i32,

    /// Display precision (number of decimal places).
    pub display_precision: i32,

    /// Trade units precision.
    pub trade_units_precision: i32,

    /// Minimum trade size.
    pub minimum_trade_size: String,

    /// Maximum trailing stop distance.
    pub maximum_trailing_stop_distance: String,

    /// Minimum trailing stop distance.
    pub minimum_trailing_stop_distance: String,

    /// Maximum position size (0 = unlimited).
    pub maximum_position_size: String,

    /// Maximum order units.
    pub maximum_order_units: String,

    /// Margin rate for the instrument.
    pub margin_rate: String,

    /// The current guaranteed Stop Loss Order mode.
    #[serde(default)]
    pub guaranteed_stop_loss_order_mode: Option<OANDAGuaranteedStopLossOrderMode>,

    /// Minimum guaranteed stop loss distance.
    #[serde(default)]
    pub minimum_guaranteed_stop_loss_distance: Option<String>,

    /// Guaranteed stop loss order execution premium.
    #[serde(default)]
    pub guaranteed_stop_loss_order_execution_premium: Option<String>,

    /// Commission structure for the instrument.
    #[serde(default)]
    pub commission: Option<InstrumentCommission>,

    /// Financing rates for the instrument.
    #[serde(default)]
    pub financing: Option<InstrumentFinancing>,
}

/// OANDA instrument commission.
///
/// Specifies commission charged per trade.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentCommission {
    /// Commission amount per units traded.
    #[serde(default)]
    pub commission: Option<String>,

    /// Number of units the commission is based on.
    #[serde(default)]
    pub units_traded: Option<String>,

    /// Minimum commission amount.
    #[serde(default)]
    pub minimum_commission: Option<String>,
}

/// OANDA instrument financing.
///
/// Specifies financing rates for holding positions overnight.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentFinancing {
    /// Financing rate for long positions.
    #[serde(default)]
    pub long_rate: Option<String>,

    /// Financing rate for short positions.
    #[serde(default)]
    pub short_rate: Option<String>,

    /// Days financing is charged (0=Sunday, 6=Saturday).
    #[serde(default)]
    pub financing_days_of_week: Option<Vec<FinancingDayOfWeek>>,
}

/// OANDA financing day of week.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancingDayOfWeek {
    /// Day of week (SUNDAY, MONDAY, etc.).
    #[serde(default)]
    pub day_of_week: Option<String>,

    /// Number of days charged.
    #[serde(default)]
    pub days_charged: Option<i32>,
}

// ================================================================================================
// Pricing Models
// ================================================================================================

/// OANDA pricing response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingResponse {
    /// List of prices.
    pub prices: Vec<Price>,

    /// Response time.
    pub time: String,
}

/// OANDA price.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    /// Instrument name.
    pub instrument: String,

    /// Price timestamp.
    pub time: String,

    /// Whether the instrument is tradeable.
    pub tradeable: bool,

    /// Bid prices.
    #[serde(default)]
    pub bids: Vec<PriceLevel>,

    /// Ask prices.
    #[serde(default)]
    pub asks: Vec<PriceLevel>,

    /// Closeout bid.
    #[serde(default)]
    pub closeout_bid: Option<String>,

    /// Closeout ask.
    #[serde(default)]
    pub closeout_ask: Option<String>,

    /// Status (e.g., "tradeable", "non-tradeable").
    #[serde(default)]
    pub status: Option<String>,
}

/// OANDA price level (bid or ask).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceLevel {
    /// Price.
    pub price: String,

    /// Liquidity at this price level.
    pub liquidity: i64,
}

// ================================================================================================
// Order Models
// ================================================================================================

/// OANDA order response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    /// The order that was created.
    #[serde(default)]
    pub order_create_transaction: Option<OrderTransaction>,

    /// The order fill transaction (if filled immediately).
    #[serde(default)]
    pub order_fill_transaction: Option<OrderFillTransaction>,

    /// The order cancel transaction (if cancelled).
    #[serde(default)]
    pub order_cancel_transaction: Option<OrderCancelTransaction>,

    /// Related transaction IDs.
    #[serde(default, rename = "relatedTransactionIDs")]
    pub related_transaction_ids: Vec<String>,

    /// Last transaction ID.
    #[serde(default, rename = "lastTransactionID")]
    pub last_transaction_id: Option<String>,
}

/// OANDA order transaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderTransaction {
    /// Transaction ID.
    pub id: String,

    /// Account ID.
    #[serde(rename = "accountID")]
    pub account_id: String,

    /// Transaction time.
    pub time: String,

    /// Order type.
    #[serde(rename = "type")]
    pub transaction_type: String,

    /// Instrument.
    pub instrument: String,

    /// Units (positive = buy, negative = sell).
    pub units: String,

    /// Price (for limit orders).
    #[serde(default)]
    pub price: Option<String>,

    /// Time in force.
    #[serde(default)]
    pub time_in_force: Option<String>,
}

/// OANDA order fill transaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFillTransaction {
    /// Transaction ID.
    pub id: String,

    /// Account ID.
    #[serde(rename = "accountID")]
    pub account_id: String,

    /// Transaction time.
    pub time: String,

    /// Order ID that was filled.
    #[serde(rename = "orderID")]
    pub order_id: String,

    /// Instrument.
    pub instrument: String,

    /// Units filled.
    pub units: String,

    /// Fill price.
    pub price: String,

    /// Profit/loss from this fill.
    pub pl: String,

    /// Commission.
    #[serde(default)]
    pub commission: Option<String>,

    /// Account balance after fill.
    pub account_balance: String,
}

/// OANDA order cancel transaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancelTransaction {
    /// Transaction ID.
    pub id: String,

    /// Order ID that was cancelled.
    #[serde(rename = "orderID")]
    pub order_id: String,

    /// Transaction time.
    pub time: String,

    /// Reason for cancellation.
    #[serde(default)]
    pub reason: Option<String>,
}

// ================================================================================================
// Position Models
// ================================================================================================

/// OANDA positions response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionsResponse {
    /// List of positions.
    pub positions: Vec<Position>,

    /// Last transaction ID.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: String,
}

/// OANDA position.
///
/// The specification of a Position within an Account.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Instrument.
    pub instrument: String,

    /// Profit/loss realized over the lifetime of the Account.
    pub pl: String,

    /// The unrealized profit/loss of all open Trades.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: String,

    /// Margin currently used by the Position.
    #[serde(default)]
    pub margin_used: Option<String>,

    /// Profit/loss since the Account's resettablePL was last reset.
    #[serde(rename = "resettablePL")]
    pub resettable_pl: String,

    /// The total amount of financing paid/collected for this instrument.
    #[serde(default)]
    pub financing: Option<String>,

    /// The total amount of commission paid for this instrument.
    pub commission: String,

    /// Fees charged for guaranteed Stop Loss Orders.
    #[serde(default)]
    pub guaranteed_execution_fees: Option<String>,

    /// Long position details.
    pub long: PositionSide,

    /// Short position details.
    pub short: PositionSide,
}

/// OANDA position side (long or short).
///
/// The representation of a Position for a single direction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionSide {
    /// Number of units in the position.
    pub units: String,

    /// Volume-weighted average of entry prices.
    #[serde(default)]
    pub average_price: Option<String>,

    /// Profit/loss realized for this side.
    pub pl: String,

    /// Unrealized profit/loss for this side.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: String,

    /// Resettable profit/loss for this side.
    #[serde(rename = "resettablePL")]
    pub resettable_pl: String,

    /// Total financing paid/collected for this side.
    #[serde(default)]
    pub financing: Option<String>,

    /// Fees charged for guaranteed Stop Loss Orders.
    #[serde(default)]
    pub guaranteed_execution_fees: Option<String>,

    /// List of Trade IDs contributing to this side.
    #[serde(default, rename = "tradeIDs")]
    pub trade_ids: Option<Vec<String>>,
}

// ================================================================================================
// Candle Models
// ================================================================================================

/// OANDA candles response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlesResponse {
    /// Instrument name.
    pub instrument: String,

    /// Candle granularity.
    pub granularity: OANDAGranularity,

    /// List of candles.
    pub candles: Vec<Candle>,
}

/// OANDA candle.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candle {
    /// Candle timestamp.
    pub time: String,

    /// Whether the candle is complete.
    pub complete: bool,

    /// Volume (number of ticks).
    pub volume: i64,

    /// Mid prices (if requested).
    #[serde(default)]
    pub mid: Option<CandleOHLC>,

    /// Bid prices (if requested).
    #[serde(default)]
    pub bid: Option<CandleOHLC>,

    /// Ask prices (if requested).
    #[serde(default)]
    pub ask: Option<CandleOHLC>,
}

/// OANDA candle OHLC values.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandleOHLC {
    /// Open price.
    #[serde(rename = "o")]
    pub open: String,

    /// High price.
    #[serde(rename = "h")]
    pub high: String,

    /// Low price.
    #[serde(rename = "l")]
    pub low: String,

    /// Close price.
    #[serde(rename = "c")]
    pub close: String,
}

// ================================================================================================
// Trade Models
// ================================================================================================

/// OANDA trades response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradesResponse {
    /// List of trades.
    pub trades: Vec<Trade>,

    /// Last transaction ID.
    #[serde(rename = "lastTransactionID")]
    pub last_transaction_id: String,
}

/// OANDA trade.
///
/// The specification of a Trade within an Account. This includes the
/// full representation of the Trade's dependent Orders.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Trade ID.
    pub id: String,

    /// Instrument.
    pub instrument: String,

    /// Execution price of the Trade.
    pub price: String,

    /// Open time (RFC3339 format).
    pub open_time: String,

    /// Trade state.
    pub state: OANDATradeState,

    /// Initial units when the Trade was opened.
    pub initial_units: String,

    /// Initial margin required for the Trade.
    #[serde(default)]
    pub initial_margin_required: Option<String>,

    /// Current number of units (positive = long, negative = short).
    pub current_units: String,

    /// Realized profit/loss on the closed portion.
    #[serde(rename = "realizedPL")]
    pub realized_pl: String,

    /// Unrealized profit/loss on the open portion.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: String,

    /// Margin currently used by the Trade.
    #[serde(default)]
    pub margin_used: Option<String>,

    /// Average closing price (if partially or fully closed).
    #[serde(default)]
    pub average_close_price: Option<String>,

    /// List of closing Transaction IDs.
    #[serde(default, rename = "closingTransactionIDs")]
    pub closing_transaction_ids: Option<Vec<String>>,

    /// Total financing paid/collected for this Trade.
    #[serde(default)]
    pub financing: Option<String>,

    /// Close time (RFC3339 format, if closed).
    #[serde(default)]
    pub close_time: Option<String>,

    /// Client extensions.
    #[serde(default)]
    pub client_extensions: Option<ClientExtensions>,

    /// The Trade's Take Profit Order.
    #[serde(default)]
    pub take_profit_order: Option<TakeProfitOrder>,

    /// The Trade's Stop Loss Order.
    #[serde(default)]
    pub stop_loss_order: Option<StopLossOrder>,

    /// The Trade's Trailing Stop Loss Order.
    #[serde(default)]
    pub trailing_stop_loss_order: Option<TrailingStopLossOrder>,
}

/// OANDA trade summary.
///
/// Simplified representation used in Account details.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSummary {
    /// Trade ID.
    pub id: String,

    /// Instrument.
    pub instrument: String,

    /// Execution price.
    pub price: String,

    /// Open time (RFC3339 format).
    pub open_time: String,

    /// Trade state.
    pub state: OANDATradeState,

    /// Initial units when opened.
    pub initial_units: String,

    /// Current number of units.
    pub current_units: String,

    /// Realized profit/loss.
    #[serde(rename = "realizedPL")]
    pub realized_pl: String,

    /// Unrealized profit/loss.
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: String,

    /// Margin used.
    #[serde(default)]
    pub margin_used: Option<String>,

    /// Client extensions.
    #[serde(default)]
    pub client_extensions: Option<ClientExtensions>,

    /// Take Profit Order ID.
    #[serde(default, rename = "takeProfitOrderID")]
    pub take_profit_order_id: Option<String>,

    /// Stop Loss Order ID.
    #[serde(default, rename = "stopLossOrderID")]
    pub stop_loss_order_id: Option<String>,

    /// Trailing Stop Loss Order ID.
    #[serde(default, rename = "trailingStopLossOrderID")]
    pub trailing_stop_loss_order_id: Option<String>,
}

/// OANDA client extensions (for custom order/trade IDs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientExtensions {
    /// Client-specified ID.
    #[serde(default)]
    pub id: Option<String>,

    /// Client-specified tag.
    #[serde(default)]
    pub tag: Option<String>,

    /// Client-specified comment.
    #[serde(default)]
    pub comment: Option<String>,
}

// ================================================================================================
// Order Models (Dependent Orders)
// ================================================================================================

/// OANDA order (generic representation).
///
/// The base representation of an Order. Orders are instructions to the
/// broker to perform a trading action.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// The Order's identifier.
    pub id: String,

    /// The time the Order was created (RFC3339 format).
    pub create_time: String,

    /// The current state of the Order.
    pub state: OANDAOrderState,

    /// The type of the Order.
    #[serde(rename = "type")]
    pub order_type: String,

    /// The instrument for the Order.
    #[serde(default)]
    pub instrument: Option<String>,

    /// The quantity requested to be filled by the Order.
    #[serde(default)]
    pub units: Option<String>,

    /// The price threshold for the Order.
    #[serde(default)]
    pub price: Option<String>,

    /// The time-in-force requested for the Order.
    #[serde(default)]
    pub time_in_force: Option<String>,

    /// The date/time when the Order will be cancelled (for GTD orders).
    #[serde(default, rename = "gtdTime")]
    pub gtd_time: Option<String>,

    /// ID of the Trade this Order is attached to (for dependent orders).
    #[serde(default, rename = "tradeID")]
    pub trade_id: Option<String>,

    /// Client extensions.
    #[serde(default)]
    pub client_extensions: Option<ClientExtensions>,

    /// Transaction ID that filled this Order.
    #[serde(default, rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<String>,

    /// Date/time when the Order was filled (RFC3339 format).
    #[serde(default)]
    pub filled_time: Option<String>,

    /// Transaction ID that cancelled this Order.
    #[serde(default, rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<String>,

    /// Date/time when the Order was cancelled (RFC3339 format).
    #[serde(default)]
    pub cancelled_time: Option<String>,
}

/// OANDA Take Profit Order.
///
/// An order to close a Trade when the price reaches a specified level.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeProfitOrder {
    /// The Order's identifier.
    pub id: String,

    /// The time the Order was created (RFC3339 format).
    pub create_time: String,

    /// The current state of the Order.
    pub state: OANDAOrderState,

    /// The ID of the Trade to close.
    #[serde(rename = "tradeID")]
    pub trade_id: String,

    /// The price threshold (trigger price).
    pub price: String,

    /// The time-in-force requested for the Order.
    #[serde(default)]
    pub time_in_force: Option<String>,

    /// The date/time when the Order will be cancelled (for GTD orders).
    #[serde(default, rename = "gtdTime")]
    pub gtd_time: Option<String>,

    /// Client extensions.
    #[serde(default)]
    pub client_extensions: Option<ClientExtensions>,

    /// Transaction ID that filled this Order.
    #[serde(default, rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<String>,

    /// Date/time when the Order was filled.
    #[serde(default)]
    pub filled_time: Option<String>,

    /// Trade ID opened when the Order was filled.
    #[serde(default, rename = "tradeOpenedID")]
    pub trade_opened_id: Option<String>,

    /// Trade ID reduced when the Order was filled.
    #[serde(default, rename = "tradeReducedID")]
    pub trade_reduced_id: Option<String>,

    /// Trade IDs closed when the Order was filled.
    #[serde(default, rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<String>>,

    /// Transaction ID that cancelled this Order.
    #[serde(default, rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<String>,

    /// Date/time when the Order was cancelled.
    #[serde(default)]
    pub cancelled_time: Option<String>,

    /// ID of the Order that replaced this Order.
    #[serde(default, rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<String>,
}

/// OANDA Stop Loss Order.
///
/// An order to close a Trade when the price reaches a specified level
/// to limit losses.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopLossOrder {
    /// The Order's identifier.
    pub id: String,

    /// The time the Order was created (RFC3339 format).
    pub create_time: String,

    /// The current state of the Order.
    pub state: OANDAOrderState,

    /// The ID of the Trade to close.
    #[serde(rename = "tradeID")]
    pub trade_id: String,

    /// The price threshold (trigger price).
    pub price: String,

    /// The distance (in price units) from the Trade's open price for a dynamic price.
    #[serde(default)]
    pub distance: Option<String>,

    /// The time-in-force requested for the Order.
    #[serde(default)]
    pub time_in_force: Option<String>,

    /// The date/time when the Order will be cancelled (for GTD orders).
    #[serde(default, rename = "gtdTime")]
    pub gtd_time: Option<String>,

    /// Specifies which price component to use for triggering.
    #[serde(default)]
    pub trigger_condition: Option<String>,

    /// Flag indicating if this is a guaranteed stop loss order.
    #[serde(default)]
    pub guaranteed: Option<bool>,

    /// Premium charged for guaranteed stop loss execution.
    #[serde(default)]
    pub guaranteed_execution_premium: Option<String>,

    /// Client extensions.
    #[serde(default)]
    pub client_extensions: Option<ClientExtensions>,

    /// Transaction ID that filled this Order.
    #[serde(default, rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<String>,

    /// Date/time when the Order was filled.
    #[serde(default)]
    pub filled_time: Option<String>,

    /// Trade ID opened when the Order was filled.
    #[serde(default, rename = "tradeOpenedID")]
    pub trade_opened_id: Option<String>,

    /// Trade ID reduced when the Order was filled.
    #[serde(default, rename = "tradeReducedID")]
    pub trade_reduced_id: Option<String>,

    /// Trade IDs closed when the Order was filled.
    #[serde(default, rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<String>>,

    /// Transaction ID that cancelled this Order.
    #[serde(default, rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<String>,

    /// Date/time when the Order was cancelled.
    #[serde(default)]
    pub cancelled_time: Option<String>,

    /// ID of the Order that replaced this Order.
    #[serde(default, rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<String>,
}

/// OANDA Trailing Stop Loss Order.
///
/// An order with a dynamic price that follows the market by a specified distance.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingStopLossOrder {
    /// The Order's identifier.
    pub id: String,

    /// The time the Order was created (RFC3339 format).
    pub create_time: String,

    /// The current state of the Order.
    pub state: OANDAOrderState,

    /// The ID of the Trade to close.
    #[serde(rename = "tradeID")]
    pub trade_id: String,

    /// The trailing distance from the current price (in price units).
    pub distance: String,

    /// The trigger price for the Trailing Stop Loss Order.
    /// This is the price at which the order will be triggered.
    #[serde(default)]
    pub trailing_stop_value: Option<String>,

    /// The time-in-force requested for the Order.
    #[serde(default)]
    pub time_in_force: Option<String>,

    /// The date/time when the Order will be cancelled (for GTD orders).
    #[serde(default, rename = "gtdTime")]
    pub gtd_time: Option<String>,

    /// Specifies which price component to use for triggering.
    #[serde(default)]
    pub trigger_condition: Option<String>,

    /// Client extensions.
    #[serde(default)]
    pub client_extensions: Option<ClientExtensions>,

    /// Transaction ID that filled this Order.
    #[serde(default, rename = "fillingTransactionID")]
    pub filling_transaction_id: Option<String>,

    /// Date/time when the Order was filled.
    #[serde(default)]
    pub filled_time: Option<String>,

    /// Trade ID opened when the Order was filled.
    #[serde(default, rename = "tradeOpenedID")]
    pub trade_opened_id: Option<String>,

    /// Trade ID reduced when the Order was filled.
    #[serde(default, rename = "tradeReducedID")]
    pub trade_reduced_id: Option<String>,

    /// Trade IDs closed when the Order was filled.
    #[serde(default, rename = "tradeClosedIDs")]
    pub trade_closed_ids: Option<Vec<String>>,

    /// Transaction ID that cancelled this Order.
    #[serde(default, rename = "cancellingTransactionID")]
    pub cancelling_transaction_id: Option<String>,

    /// Date/time when the Order was cancelled.
    #[serde(default)]
    pub cancelled_time: Option<String>,

    /// ID of the Order that replaced this Order.
    #[serde(default, rename = "replacedByOrderID")]
    pub replaced_by_order_id: Option<String>,
}

// ================================================================================================
// Streaming Models (for WebSocket Phase 1.3)
// ================================================================================================

/// OANDA streaming price (heartbeat or price update).
///
/// Used in Phase 1.3 WebSocket implementation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum StreamingMessage {
    /// Price update message.
    #[serde(rename = "PRICE")]
    Price(StreamingPrice),

    /// Heartbeat message.
    #[serde(rename = "HEARTBEAT")]
    Heartbeat(StreamingHeartbeat),
}

/// OANDA streaming price update.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingPrice {
    /// Instrument name.
    pub instrument: String,

    /// Price timestamp (RFC3339 format).
    pub time: String,

    /// Whether the instrument is currently tradeable.
    pub tradeable: bool,

    /// Bid prices (top of book first).
    #[serde(default)]
    pub bids: Vec<PriceLevel>,

    /// Ask prices (top of book first).
    #[serde(default)]
    pub asks: Vec<PriceLevel>,

    /// Closeout bid price.
    #[serde(default)]
    pub closeout_bid: Option<String>,

    /// Closeout ask price.
    #[serde(default)]
    pub closeout_ask: Option<String>,
}

/// OANDA streaming heartbeat.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingHeartbeat {
    /// Heartbeat timestamp (RFC3339 format).
    pub time: String,
}

// ================================================================================================
// Error Response Models
// ================================================================================================

/// OANDA API error response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Error message.
    #[serde(default)]
    pub error_message: Option<String>,

    /// Error code/type.
    #[serde(default)]
    pub error_code: Option<String>,

    /// Reject reason (for order rejections).
    #[serde(default)]
    pub reject_reason: Option<String>,
}
