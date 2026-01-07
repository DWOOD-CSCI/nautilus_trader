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

//! Python bindings for the OANDA HTTP client.

use nautilus_core::python::to_pyvalue_err;
use pyo3::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::{
    common::{
        credential::OANDACredential,
        enums::{OANDAEnvironment, OANDAGranularity, OANDAPriceComponent},
    },
    http::client::OANDAHttpClient,
};

#[pymethods]
impl OANDAHttpClient {
    /// Create a new OANDA HTTP client.
    ///
    /// # Parameters
    ///
    /// * `api_key` - OANDA API access token
    /// * `account_id` - OANDA account ID
    /// * `environment` - The OANDA environment (Practice or Live)
    /// * `timeout_secs` - Optional HTTP request timeout in seconds
    /// * `rate_limit_per_second` - Optional rate limit for API calls
    ///
    /// # Errors
    ///
    /// Returns a `PyErr` if the client cannot be created.
    #[new]
    #[pyo3(signature = (
        api_key,
        account_id,
        environment = OANDAEnvironment::Practice,
        timeout_secs = None,
        rate_limit_per_second = None
    ))]
    fn py_new(
        api_key: String,
        account_id: String,
        environment: OANDAEnvironment,
        timeout_secs: Option<u64>,
        rate_limit_per_second: Option<u32>,
    ) -> PyResult<Self> {
        let credential = OANDACredential::new(api_key, account_id.clone());
        Self::new(
            credential,
            account_id,
            environment,
            timeout_secs,
            rate_limit_per_second,
        )
        .map_err(to_pyvalue_err)
    }

    /// Get the account ID.
    #[getter]
    #[pyo3(name = "account_id")]
    fn py_account_id(&self) -> String {
        self.account_id().to_string()
    }

    /// Get account summary information.
    ///
    /// Returns account balance, margin, and other details as a JSON string.
    #[pyo3(name = "get_account_summary")]
    fn py_get_account_summary<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let summary = client.get_account_summary().await.map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&summary).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Get all instruments available for trading.
    ///
    /// Returns a list of tradeable instruments as a JSON string.
    #[pyo3(name = "get_instruments")]
    fn py_get_instruments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let instruments = client.get_instruments().await.map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&instruments).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Get current prices for specified instruments.
    ///
    /// # Parameters
    ///
    /// * `instruments` - List of instrument names (e.g., ["EUR_USD", "GBP_USD"])
    #[pyo3(name = "get_pricing")]
    fn py_get_pricing<'py>(
        &self,
        py: Python<'py>,
        instruments: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let refs: Vec<&str> = instruments.iter().map(|s| s.as_str()).collect();
            let prices = client.get_pricing(&refs).await.map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&prices).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Get historical candles for an instrument.
    ///
    /// # Parameters
    ///
    /// * `instrument` - Instrument name (e.g., "EUR_USD")
    /// * `granularity` - Candle granularity (e.g., "M1", "H1", "D")
    /// * `price_component` - Optional price component (e.g., "M", "B", "A", "BA", "MBA")
    /// * `count` - Optional number of candles to retrieve (max 5000)
    /// * `from_time` - Optional start time (RFC3339 format)
    /// * `to_time` - Optional end time (RFC3339 format)
    #[pyo3(name = "get_candles")]
    #[pyo3(signature = (instrument, granularity, price_component = None, count = None, from_time = None, to_time = None))]
    fn py_get_candles<'py>(
        &self,
        py: Python<'py>,
        instrument: String,
        granularity: String,
        price_component: Option<String>,
        count: Option<i32>,
        from_time: Option<String>,
        to_time: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();

        // Parse granularity
        let gran = OANDAGranularity::from_str(&granularity).map_err(to_pyvalue_err)?;

        // Parse price component if provided
        let price = price_component
            .map(|p| OANDAPriceComponent::from_str(&p))
            .transpose()
            .map_err(to_pyvalue_err)?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let candles = client
                .get_candles(
                    &instrument,
                    gran,
                    price,
                    count,
                    from_time.as_deref(),
                    to_time.as_deref(),
                )
                .await
                .map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&candles).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Get open positions for the account.
    #[pyo3(name = "get_open_positions")]
    fn py_get_open_positions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let positions = client.get_open_positions().await.map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&positions).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Get open trades for the account.
    #[pyo3(name = "get_open_trades")]
    fn py_get_open_trades<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let trades = client.get_open_trades().await.map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&trades).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Submit a market order.
    ///
    /// # Parameters
    ///
    /// * `instrument` - Instrument to trade (e.g., "EUR_USD")
    /// * `units` - Number of units as string (positive for buy, negative for sell)
    #[pyo3(name = "create_market_order")]
    fn py_create_market_order<'py>(
        &self,
        py: Python<'py>,
        instrument: String,
        units: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        let units_decimal = Decimal::from_str(&units).map_err(to_pyvalue_err)?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = client
                .create_market_order(&instrument, units_decimal, None, None)
                .await
                .map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&response).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Submit a limit order.
    ///
    /// # Parameters
    ///
    /// * `instrument` - Instrument to trade (e.g., "EUR_USD")
    /// * `units` - Number of units as string (positive for buy, negative for sell)
    /// * `price` - Limit price as string
    #[pyo3(name = "create_limit_order")]
    fn py_create_limit_order<'py>(
        &self,
        py: Python<'py>,
        instrument: String,
        units: String,
        price: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        let units_decimal = Decimal::from_str(&units).map_err(to_pyvalue_err)?;
        let price_decimal = Decimal::from_str(&price).map_err(to_pyvalue_err)?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = client
                .create_limit_order(&instrument, units_decimal, price_decimal, None, None, None)
                .await
                .map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&response).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Cancel an order.
    ///
    /// # Parameters
    ///
    /// * `order_id` - ID of the order to cancel
    #[pyo3(name = "cancel_order")]
    fn py_cancel_order<'py>(
        &self,
        py: Python<'py>,
        order_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = client.cancel_order(&order_id).await.map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&response).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    /// Close a trade.
    ///
    /// # Parameters
    ///
    /// * `trade_id` - ID of the trade to close
    /// * `units` - Optional number of units to close as string (closes all if None)
    #[pyo3(name = "close_trade")]
    #[pyo3(signature = (trade_id, units = None))]
    fn py_close_trade<'py>(
        &self,
        py: Python<'py>,
        trade_id: String,
        units: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        let units_decimal = units
            .map(|u| Decimal::from_str(&u))
            .transpose()
            .map_err(to_pyvalue_err)?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = client
                .close_trade(&trade_id, units_decimal)
                .await
                .map_err(to_pyvalue_err)?;
            let json_str = serde_json::to_string(&response).map_err(to_pyvalue_err)?;
            Ok(json_str)
        })
    }

    fn __repr__(&self) -> String {
        format!("OANDAHttpClient(account_id='{}')", self.account_id())
    }
}
