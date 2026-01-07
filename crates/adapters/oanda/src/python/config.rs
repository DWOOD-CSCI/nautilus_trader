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

//! Python bindings for OANDA configuration types.

use pyo3::prelude::*;

use crate::{
    common::enums::OANDAEnvironment,
    config::{OANDADataClientConfig, OANDAExecClientConfig},
};

#[pymethods]
impl OANDADataClientConfig {
    /// Create a new OANDA data client configuration.
    ///
    /// # Parameters
    ///
    /// * `api_token` - OANDA API access token (optional, can use env var)
    /// * `account_id` - OANDA account ID (optional, can use env var)
    /// * `environment` - Trading environment (Practice or Live)
    /// * `instruments` - List of instruments to subscribe to
    /// * `use_streaming` - Whether to use streaming API for real-time data
    /// * `timeout_secs` - HTTP request timeout in seconds
    #[new]
    #[pyo3(signature = (
        api_token = None,
        account_id = None,
        environment = OANDAEnvironment::Practice,
        instruments = Vec::new(),
        use_streaming = true,
        timeout_secs = 30
    ))]
    fn py_new(
        api_token: Option<String>,
        account_id: Option<String>,
        environment: OANDAEnvironment,
        instruments: Vec<String>,
        use_streaming: bool,
        timeout_secs: u64,
    ) -> Self {
        Self {
            api_token,
            account_id,
            environment,
            instruments,
            use_streaming,
            timeout_secs,
        }
    }

    #[getter]
    #[pyo3(name = "api_token")]
    fn py_api_token(&self) -> Option<String> {
        self.api_token.clone()
    }

    #[getter]
    #[pyo3(name = "account_id")]
    fn py_account_id(&self) -> Option<String> {
        self.account_id.clone()
    }

    #[getter]
    #[pyo3(name = "environment")]
    const fn py_environment(&self) -> OANDAEnvironment {
        self.environment
    }

    #[getter]
    #[pyo3(name = "instruments")]
    fn py_instruments(&self) -> Vec<String> {
        self.instruments.clone()
    }

    #[getter]
    #[pyo3(name = "use_streaming")]
    const fn py_use_streaming(&self) -> bool {
        self.use_streaming
    }

    #[getter]
    #[pyo3(name = "timeout_secs")]
    const fn py_timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    fn __repr__(&self) -> String {
        format!(
            "OANDADataClientConfig(environment={}, instruments={:?})",
            self.environment, self.instruments
        )
    }
}

#[pymethods]
impl OANDAExecClientConfig {
    /// Create a new OANDA execution client configuration.
    ///
    /// # Parameters
    ///
    /// * `api_token` - OANDA API access token (optional, can use env var)
    /// * `account_id` - OANDA account ID (optional, can use env var)
    /// * `environment` - Trading environment (Practice or Live)
    /// * `timeout_secs` - HTTP request timeout in seconds
    /// * `max_retries` - Maximum number of retries for failed requests
    /// * `reject_on_disconnect` - Whether to reject orders when disconnected
    #[new]
    #[pyo3(signature = (
        api_token = None,
        account_id = None,
        environment = OANDAEnvironment::Practice,
        timeout_secs = 30,
        max_retries = 3,
        reject_on_disconnect = true
    ))]
    fn py_new(
        api_token: Option<String>,
        account_id: Option<String>,
        environment: OANDAEnvironment,
        timeout_secs: u64,
        max_retries: u32,
        reject_on_disconnect: bool,
    ) -> Self {
        Self {
            api_token,
            account_id,
            environment,
            timeout_secs,
            max_retries,
            reject_on_disconnect,
        }
    }

    #[getter]
    #[pyo3(name = "api_token")]
    fn py_api_token(&self) -> Option<String> {
        self.api_token.clone()
    }

    #[getter]
    #[pyo3(name = "account_id")]
    fn py_account_id(&self) -> Option<String> {
        self.account_id.clone()
    }

    #[getter]
    #[pyo3(name = "environment")]
    const fn py_environment(&self) -> OANDAEnvironment {
        self.environment
    }

    #[getter]
    #[pyo3(name = "timeout_secs")]
    const fn py_timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    #[getter]
    #[pyo3(name = "max_retries")]
    const fn py_max_retries(&self) -> u32 {
        self.max_retries
    }

    #[getter]
    #[pyo3(name = "reject_on_disconnect")]
    const fn py_reject_on_disconnect(&self) -> bool {
        self.reject_on_disconnect
    }

    fn __repr__(&self) -> String {
        format!(
            "OANDAExecClientConfig(environment={}, max_retries={})",
            self.environment, self.max_retries
        )
    }
}
