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

//! OANDA API URLs for different environments.

/// OANDA REST API base URL for practice (demo) accounts.
pub const OANDA_REST_PRACTICE_URL: &str = "https://api-fxpractice.oanda.com";

/// OANDA REST API base URL for live accounts.
pub const OANDA_REST_LIVE_URL: &str = "https://api-fxtrade.oanda.com";

/// OANDA Streaming API base URL for practice (demo) accounts.
pub const OANDA_STREAM_PRACTICE_URL: &str = "https://stream-fxpractice.oanda.com";

/// OANDA Streaming API base URL for live accounts.
pub const OANDA_STREAM_LIVE_URL: &str = "https://stream-fxtrade.oanda.com";

/// Returns the REST API base URL for the given environment.
#[must_use]
pub fn get_oanda_rest_url(is_live: bool) -> &'static str {
    if is_live {
        OANDA_REST_LIVE_URL
    } else {
        OANDA_REST_PRACTICE_URL
    }
}

/// Returns the Streaming API base URL for the given environment.
#[must_use]
pub fn get_oanda_stream_url(is_live: bool) -> &'static str {
    if is_live {
        OANDA_STREAM_LIVE_URL
    } else {
        OANDA_STREAM_PRACTICE_URL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rest_url_practice() {
        assert_eq!(get_oanda_rest_url(false), OANDA_REST_PRACTICE_URL);
    }

    #[test]
    fn test_get_rest_url_live() {
        assert_eq!(get_oanda_rest_url(true), OANDA_REST_LIVE_URL);
    }

    #[test]
    fn test_get_stream_url_practice() {
        assert_eq!(get_oanda_stream_url(false), OANDA_STREAM_PRACTICE_URL);
    }

    #[test]
    fn test_get_stream_url_live() {
        assert_eq!(get_oanda_stream_url(true), OANDA_STREAM_LIVE_URL);
    }
}
