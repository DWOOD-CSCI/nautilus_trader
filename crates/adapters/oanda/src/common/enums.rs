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

//! OANDA-specific enums and type mappings.
//!
//! These enums map to OANDA v20 API types and are used throughout
//! the adapter for type-safe representation of API values.
//!
//! # Reference
//!
//! Enum values informed by oanda-v20-openapi crate (MIT license).
//! See: <https://github.com/jxcv0/oanda-v20-openapi>

use std::fmt;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// OANDA trading environment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.oanda", eq, eq_int)
)]
pub enum OANDAEnvironment {
    /// Practice (demo) trading environment.
    #[default]
    Practice,
    /// Live trading environment.
    Live,
}

impl OANDAEnvironment {
    /// Returns true if this is the live environment.
    #[must_use]
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live)
    }
}

/// OANDA instrument type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OANDAInstrumentType {
    /// Currency pair (forex).
    Currency,
    /// Contract for difference.
    Cfd,
    /// Metal (gold, silver, etc.).
    Metal,
}

/// OANDA order type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OANDAOrderType {
    /// Market order - executed immediately at best available price.
    Market,
    /// Limit order - executed at specified price or better.
    Limit,
    /// Stop order - becomes market order when price reaches trigger.
    Stop,
    /// Market if touched - becomes market order when price touches level.
    MarketIfTouched,
    /// Take profit order.
    TakeProfit,
    /// Stop loss order.
    StopLoss,
    /// Guaranteed stop loss order.
    GuaranteedStopLoss,
    /// Trailing stop loss order.
    TrailingStopLoss,
    /// Fixed price order (internal use).
    FixedPrice,
}

/// OANDA order state.
///
/// The current state of an Order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OANDAOrderState {
    /// Order is pending execution.
    Pending,
    /// Order has been filled.
    Filled,
    /// Order was triggered.
    Triggered,
    /// Order was cancelled.
    Cancelled,
}

/// OANDA trade state.
///
/// The current state of a Trade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OANDATradeState {
    /// Trade is currently open.
    Open,
    /// Trade has been fully closed.
    Closed,
    /// Only for internal orders - the Trade was closed with a marketIfTouched.
    CloseWhenTradeable,
}

/// OANDA guaranteed stop loss order mode.
///
/// The overall behaviour of the Account regarding guaranteed Stop Loss Orders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OANDAGuaranteedStopLossOrderMode {
    /// Account is not permitted to create guaranteed Stop Loss Orders.
    Disabled,
    /// Account is able to create guaranteed Stop Loss Orders.
    Allowed,
    /// Account is required to have guaranteed Stop Loss Orders for all open Trades.
    Required,
}

/// OANDA time in force.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OANDATimeInForce {
    /// Good till cancelled.
    Gtc,
    /// Good till date.
    Gtd,
    /// Good for day.
    Gfd,
    /// Fill or kill.
    Fok,
    /// Immediate or cancel.
    Ioc,
}

impl OANDATimeInForce {
    /// Returns the OANDA API string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gtc => "GTC",
            Self::Gtd => "GTD",
            Self::Gfd => "GFD",
            Self::Fok => "FOK",
            Self::Ioc => "IOC",
        }
    }
}

/// OANDA position side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OANDAPositionSide {
    /// Long position.
    Long,
    /// Short position.
    Short,
}

/// OANDA candle granularity (timeframe).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OANDAGranularity {
    /// 5 second candles.
    S5,
    /// 10 second candles.
    S10,
    /// 15 second candles.
    S15,
    /// 30 second candles.
    S30,
    /// 1 minute candles.
    M1,
    /// 2 minute candles.
    M2,
    /// 4 minute candles.
    M4,
    /// 5 minute candles.
    M5,
    /// 10 minute candles.
    M10,
    /// 15 minute candles.
    M15,
    /// 30 minute candles.
    M30,
    /// 1 hour candles.
    H1,
    /// 2 hour candles.
    H2,
    /// 3 hour candles.
    H3,
    /// 4 hour candles.
    H4,
    /// 6 hour candles.
    H6,
    /// 8 hour candles.
    H8,
    /// 12 hour candles.
    H12,
    /// Daily candles.
    D,
    /// Weekly candles.
    W,
    /// Monthly candles.
    M,
}

impl OANDAGranularity {
    /// Returns the duration of this granularity in seconds.
    #[must_use]
    pub fn as_seconds(&self) -> u64 {
        match self {
            Self::S5 => 5,
            Self::S10 => 10,
            Self::S15 => 15,
            Self::S30 => 30,
            Self::M1 => 60,
            Self::M2 => 120,
            Self::M4 => 240,
            Self::M5 => 300,
            Self::M10 => 600,
            Self::M15 => 900,
            Self::M30 => 1800,
            Self::H1 => 3600,
            Self::H2 => 7200,
            Self::H3 => 10800,
            Self::H4 => 14400,
            Self::H6 => 21600,
            Self::H8 => 28800,
            Self::H12 => 43200,
            Self::D => 86400,
            Self::W => 604800,
            Self::M => 2592000, // Approximate (30 days)
        }
    }

    /// Returns the OANDA API string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::S5 => "S5",
            Self::S10 => "S10",
            Self::S15 => "S15",
            Self::S30 => "S30",
            Self::M1 => "M1",
            Self::M2 => "M2",
            Self::M4 => "M4",
            Self::M5 => "M5",
            Self::M10 => "M10",
            Self::M15 => "M15",
            Self::M30 => "M30",
            Self::H1 => "H1",
            Self::H2 => "H2",
            Self::H3 => "H3",
            Self::H4 => "H4",
            Self::H6 => "H6",
            Self::H8 => "H8",
            Self::H12 => "H12",
            Self::D => "D",
            Self::W => "W",
            Self::M => "M",
        }
    }
}

/// OANDA price component for candles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString)]
pub enum OANDAPriceComponent {
    /// Mid prices.
    #[default]
    #[serde(rename = "M")]
    #[strum(serialize = "M", serialize = "mid", serialize = "Mid")]
    Mid,
    /// Bid prices.
    #[serde(rename = "B")]
    #[strum(serialize = "B", serialize = "bid", serialize = "Bid")]
    Bid,
    /// Ask prices.
    #[serde(rename = "A")]
    #[strum(serialize = "A", serialize = "ask", serialize = "Ask")]
    Ask,
    /// Bid and Ask prices.
    #[serde(rename = "BA")]
    #[strum(serialize = "BA", serialize = "bidask", serialize = "BidAsk")]
    BidAsk,
    /// Mid, Bid and Ask prices.
    #[serde(rename = "MBA")]
    #[strum(serialize = "MBA", serialize = "all", serialize = "All")]
    All,
}

impl fmt::Display for OANDAPriceComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mid => write!(f, "M"),
            Self::Bid => write!(f, "B"),
            Self::Ask => write!(f, "A"),
            Self::BidAsk => write!(f, "BA"),
            Self::All => write!(f, "MBA"),
        }
    }
}

impl OANDAPriceComponent {
    /// Returns the OANDA API string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mid => "M",
            Self::Bid => "B",
            Self::Ask => "A",
            Self::BidAsk => "BA",
            Self::All => "MBA",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_is_live() {
        assert!(!OANDAEnvironment::Practice.is_live());
        assert!(OANDAEnvironment::Live.is_live());
    }

    #[test]
    fn test_granularity_seconds() {
        assert_eq!(OANDAGranularity::M1.as_seconds(), 60);
        assert_eq!(OANDAGranularity::H1.as_seconds(), 3600);
        assert_eq!(OANDAGranularity::D.as_seconds(), 86400);
    }

    #[test]
    fn test_order_type_display() {
        assert_eq!(OANDAOrderType::Market.to_string(), "MARKET");
        assert_eq!(OANDAOrderType::Limit.to_string(), "LIMIT");
    }
}
