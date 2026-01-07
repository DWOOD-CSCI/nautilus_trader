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

//! Configuration for OANDA adapter clients.

use serde::{Deserialize, Serialize};

use crate::common::OANDAEnvironment;

/// Configuration for the OANDA data client.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.oanda")
)]
pub struct OANDADataClientConfig {
    /// OANDA API token (can be loaded from environment).
    #[serde(default)]
    pub api_token: Option<String>,

    /// OANDA account ID (can be loaded from environment).
    #[serde(default)]
    pub account_id: Option<String>,

    /// Trading environment (practice or live).
    #[serde(default)]
    pub environment: OANDAEnvironment,

    /// Instruments to subscribe to on startup.
    #[serde(default)]
    pub instruments: Vec<String>,

    /// Whether to use streaming API for real-time data.
    #[serde(default = "default_use_streaming")]
    pub use_streaming: bool,

    /// Request timeout in seconds.
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for OANDADataClientConfig {
    fn default() -> Self {
        Self {
            api_token: None,
            account_id: None,
            environment: OANDAEnvironment::default(),
            instruments: Vec::new(),
            use_streaming: default_use_streaming(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// Configuration for the OANDA execution client.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.oanda")
)]
pub struct OANDAExecClientConfig {
    /// OANDA API token (can be loaded from environment).
    #[serde(default)]
    pub api_token: Option<String>,

    /// OANDA account ID (can be loaded from environment).
    #[serde(default)]
    pub account_id: Option<String>,

    /// Trading environment (practice or live).
    #[serde(default)]
    pub environment: OANDAEnvironment,

    /// Request timeout in seconds.
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// Maximum retries for failed requests.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to reject orders when disconnected.
    #[serde(default = "default_reject_on_disconnect")]
    pub reject_on_disconnect: bool,
}

impl Default for OANDAExecClientConfig {
    fn default() -> Self {
        Self {
            api_token: None,
            account_id: None,
            environment: OANDAEnvironment::default(),
            timeout_secs: default_timeout_secs(),
            max_retries: default_max_retries(),
            reject_on_disconnect: default_reject_on_disconnect(),
        }
    }
}

fn default_use_streaming() -> bool {
    true
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_reject_on_disconnect() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_config_default() {
        let config = OANDADataClientConfig::default();
        assert!(config.api_token.is_none());
        assert!(config.account_id.is_none());
        assert_eq!(config.environment, OANDAEnvironment::Practice);
        assert!(config.use_streaming);
    }

    #[test]
    fn test_exec_config_default() {
        let config = OANDAExecClientConfig::default();
        assert!(config.api_token.is_none());
        assert!(config.account_id.is_none());
        assert_eq!(config.environment, OANDAEnvironment::Practice);
        assert_eq!(config.max_retries, 3);
    }
}
