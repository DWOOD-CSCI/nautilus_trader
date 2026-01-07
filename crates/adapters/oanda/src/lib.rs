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

//! OANDA forex broker adapter for NautilusTrader.
//!
//! This adapter provides integration with OANDA, a major forex and CFD broker,
//! supporting both demo (practice) and live trading environments.
//!
//! # Features
//!
//! - REST API v3 client for account and trading operations
//! - Streaming API client for real-time price feeds
//! - Support for forex pairs and CFD instruments
//! - Full order lifecycle management
//! - Position and account state tracking
//!
//! # API Documentation
//!
//! - [OANDA REST API v3](https://developer.oanda.com/rest-live-v20/introduction/)
//! - [OANDA Streaming API](https://developer.oanda.com/rest-live-v20/pricing-ep/)
//!
//! # Python Bindings
//!
//! Enable the `python` feature to use this adapter from Python:
//!
//! ```toml
//! nautilus-oanda = { version = "0.53.0", features = ["python"] }
//! ```

pub mod common;
pub mod config;
pub mod http;
pub mod websocket;

#[cfg(feature = "python")]
pub mod python;

pub use config::{OANDADataClientConfig, OANDAExecClientConfig};
pub use http::{OANDAHttpClient, OANDAHttpError};
pub use websocket::{OANDAStreamClient, StreamConfig, StreamMessage, StreamError};
