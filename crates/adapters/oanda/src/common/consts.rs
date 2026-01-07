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

//! Core constants shared across the OANDA adapter components.

use std::sync::LazyLock;

use nautilus_model::identifiers::Venue;
use ustr::Ustr;

/// OANDA venue identifier string.
pub const OANDA: &str = "OANDA";

/// OANDA venue identifier.
pub static OANDA_VENUE: LazyLock<Venue> = LazyLock::new(|| Venue::new(Ustr::from(OANDA)));

/// OANDA REST API v3 version path.
pub const OANDA_API_VERSION: &str = "v3";

/// Default rate limit for OANDA REST API (requests per second).
/// OANDA allows up to 120 requests per second for most endpoints.
pub const OANDA_DEFAULT_RATE_LIMIT_PER_SECOND: u32 = 100;

/// Maximum number of candles that can be requested in a single request.
pub const OANDA_MAX_CANDLES_PER_REQUEST: usize = 5000;

/// Heartbeat interval for streaming connections (seconds).
pub const OANDA_STREAM_HEARTBEAT_INTERVAL: u64 = 5;

/// User-Agent header value for HTTP requests.
pub const OANDA_USER_AGENT: &str = "nautilus-trader/1.0";

/// Header name for specifying datetime format in OANDA API responses.
pub const OANDA_DATETIME_HEADER: &str = "Accept-Datetime-Format";
