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

//! Python bindings for the OANDA streaming client.

use nautilus_common::live::get_runtime;
use nautilus_core::python::{call_python, to_pyruntime_err, to_pyvalue_err};
use pyo3::prelude::*;

use crate::{
    common::enums::OANDAEnvironment,
    websocket::{
        client::OANDAStreamClient,
        types::{StreamConfig, StreamError, StreamMessage},
    },
};

#[pymethods]
impl OANDAStreamClient {
    /// Create a new OANDA streaming client.
    ///
    /// # Parameters
    ///
    /// * `api_key` - OANDA API access token
    /// * `account_id` - OANDA account ID
    /// * `instruments` - List of instruments to stream prices for
    /// * `environment` - The OANDA environment (Practice or Live)
    /// * `include_heartbeats` - Whether to include heartbeat messages
    /// * `reconnect_on_error` - Whether to automatically reconnect on errors
    /// * `max_reconnect_attempts` - Maximum number of reconnection attempts
    /// * `reconnect_delay_ms` - Delay between reconnection attempts in milliseconds
    ///
    /// # Errors
    ///
    /// Returns a `PyErr` if the configuration is invalid.
    #[new]
    #[pyo3(signature = (
        api_key,
        account_id,
        instruments,
        environment = OANDAEnvironment::Practice,
        include_heartbeats = false,
        reconnect_on_error = true,
        max_reconnect_attempts = 5,
        reconnect_delay_ms = 1000
    ))]
    #[allow(clippy::too_many_arguments)]
    fn py_new(
        api_key: String,
        account_id: String,
        instruments: Vec<String>,
        environment: OANDAEnvironment,
        include_heartbeats: bool,
        reconnect_on_error: bool,
        max_reconnect_attempts: u32,
        reconnect_delay_ms: u64,
    ) -> PyResult<Self> {
        let mut config = StreamConfig::new(
            environment,
            api_key,
            account_id,
            instruments,
        );
        config.include_heartbeats = include_heartbeats;
        config.reconnect_on_error = reconnect_on_error;
        config.max_reconnect_attempts = max_reconnect_attempts;
        config.reconnect_delay_ms = reconnect_delay_ms;

        config.validate().map_err(to_pyvalue_err)?;

        Ok(Self::new(config))
    }

    /// Check if the stream is currently connected.
    #[pyo3(name = "is_connected")]
    fn py_is_connected(&self) -> bool {
        self.is_connected()
    }

    /// Get the list of subscribed instruments.
    #[pyo3(name = "instruments")]
    fn py_instruments(&self) -> Vec<String> {
        self.instruments().to_vec()
    }

    /// Get streaming statistics.
    ///
    /// Returns a dictionary with messages_received, prices_received, heartbeats_received,
    /// parse_errors, reconnections, and last_message_time.
    #[pyo3(name = "stats")]
    fn py_stats(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let stats = self.stats();
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("messages_received", stats.messages_received)?;
        dict.set_item("prices_received", stats.prices_received)?;
        dict.set_item("heartbeats_received", stats.heartbeats_received)?;
        dict.set_item("parse_errors", stats.parse_errors)?;
        dict.set_item("reconnections", stats.reconnections)?;
        dict.set_item("last_message_time", stats.last_message_time.clone())?;
        dict.set_item("connected_at", stats.connected_at.clone())?;
        Ok(dict.into())
    }

    /// Connect to the OANDA pricing stream and start receiving messages.
    ///
    /// # Parameters
    ///
    /// * `callback` - Python callback function to receive price updates.
    ///   The callback receives a dictionary with price data or heartbeat.
    ///
    /// # Example
    ///
    /// ```python
    /// async def on_price(msg):
    ///     print(f"Price: {msg}")
    ///
    /// await client.connect(on_price)
    /// ```
    #[pyo3(name = "connect")]
    fn py_connect<'py>(
        &mut self,
        py: Python<'py>,
        callback: Py<PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let mut client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client.connect().await.map_err(to_pyruntime_err)?;

            // Spawn task to process stream messages
            let mut stream_client = client.clone();
            get_runtime().spawn(async move {
                loop {
                    match stream_client.next().await {
                        Some(Ok(msg)) => {
                            Python::attach(|py| {
                                let py_obj: Py<PyAny> = match &msg {
                                    StreamMessage::Price(price) => {
                                        let dict = pyo3::types::PyDict::new(py);
                                        let _ = dict.set_item("type", "price");
                                        let _ = dict.set_item("instrument", &price.instrument);
                                        let _ = dict.set_item("time", &price.time);
                                        let _ = dict.set_item("bid", price.bid);
                                        let _ = dict.set_item("ask", price.ask);
                                        let _ = dict.set_item("mid", price.mid);
                                        let _ = dict.set_item("spread", price.spread);
                                        let _ = dict.set_item("tradeable", price.tradeable);
                                        dict.into()
                                    }
                                    StreamMessage::Heartbeat(hb) => {
                                        let dict = pyo3::types::PyDict::new(py);
                                        let _ = dict.set_item("type", "heartbeat");
                                        let _ = dict.set_item("time", &hb.time);
                                        dict.into()
                                    }
                                };
                                call_python(py, &callback, py_obj);
                            });
                        }
                        Some(Err(e)) => {
                            log::warn!("OANDA stream error: {e:?}");
                            if matches!(e, StreamError::Disconnected(_)) {
                                break;
                            }
                        }
                        None => {
                            log::info!("OANDA stream ended");
                            break;
                        }
                    }
                }
            });

            Ok(())
        })
    }

    /// Disconnect from the OANDA pricing stream.
    #[pyo3(name = "disconnect")]
    fn py_disconnect<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let mut client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client.disconnect().await;
            Ok(())
        })
    }

    /// Subscribe to additional instruments.
    ///
    /// Note: This requires reconnecting to the stream with the new instruments.
    ///
    /// # Parameters
    ///
    /// * `instruments` - List of instruments to add to subscription
    #[pyo3(name = "subscribe")]
    fn py_subscribe<'py>(
        &mut self,
        py: Python<'py>,
        instruments: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let mut client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .subscribe(instruments)
                .await
                .map_err(to_pyruntime_err)?;
            Ok(())
        })
    }

    /// Unsubscribe from instruments.
    ///
    /// Note: This requires reconnecting to the stream without the specified instruments.
    ///
    /// # Parameters
    ///
    /// * `instruments` - List of instruments to remove from subscription
    #[pyo3(name = "unsubscribe")]
    fn py_unsubscribe<'py>(
        &mut self,
        py: Python<'py>,
        instruments: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let mut client = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .unsubscribe(instruments)
                .await
                .map_err(to_pyruntime_err)?;
            Ok(())
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "OANDAStreamClient(instruments={:?}, connected={})",
            self.instruments(),
            self.is_connected()
        )
    }
}
