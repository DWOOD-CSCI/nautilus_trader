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

//! WebSocket streaming client for OANDA real-time price feeds.
//!
//! OANDA provides streaming endpoints for:
//! - **Pricing Stream**: Real-time bid/ask quotes for subscribed instruments
//! - **Transaction Stream**: Account events (fills, cancels, etc.)
//!
//! # Connection Model
//!
//! OANDA uses HTTP streaming (chunked transfer encoding) rather than WebSocket.
//! The client maintains a persistent HTTP connection and reads newline-delimited
//! JSON messages.
//!
//! # Heartbeats
//!
//! OANDA sends heartbeat messages every 5 seconds when no price updates occur.
//! The client uses these to detect stale connections and trigger reconnection.
//!
//! # Rate Limits
//!
//! - Maximum 2 concurrent pricing streams per account
//! - Maximum 20 instruments per pricing stream
//!
//! # Example
//!
//! ```rust,ignore
//! use nautilus_oanda::websocket::{OANDAStreamClient, StreamConfig};
//!
//! let config = StreamConfig {
//!     environment: OANDAEnvironment::Practice,
//!     api_key: "your-api-key".into(),
//!     account_id: "your-account-id".into(),
//!     instruments: vec!["EUR_USD".into(), "GBP_USD".into()],
//! };
//!
//! let mut client = OANDAStreamClient::new(config);
//!
//! while let Some(message) = client.next().await {
//!     match message {
//!         StreamMessage::Price(price) => {
//!             println!("Price: {} bid={} ask={}", 
//!                 price.instrument, price.bids[0].price, price.asks[0].price);
//!         }
//!         StreamMessage::Heartbeat(hb) => {
//!             println!("Heartbeat at {}", hb.time);
//!         }
//!     }
//! }
//! ```

pub mod client;
pub mod transaction_client;
pub mod types;

pub use client::OANDAStreamClient;
pub use transaction_client::OANDATransactionStreamClient;
pub use types::{
    StreamConfig, StreamError, StreamHeartbeat, StreamMessage, StreamPrice,
    StreamTransaction, TransactionStreamConfig, TransactionStreamMessage, TransactionType,
};
