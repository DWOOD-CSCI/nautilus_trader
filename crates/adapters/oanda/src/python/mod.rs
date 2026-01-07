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

//! Python bindings for the OANDA adapter.
//!
//! This module provides PyO3 bindings enabling Python strategies to:
//! - Connect to OANDA REST API for account and order management
//! - Stream real-time prices via the pricing stream API
//! - Configure data and execution clients

pub mod config;
pub mod enums;
pub mod http;
pub mod streaming;

use pyo3::prelude::*;

use crate::{
    common::enums::OANDAEnvironment,
    config::{OANDADataClientConfig, OANDAExecClientConfig},
    http::client::OANDAHttpClient,
    websocket::client::OANDAStreamClient,
};

/// OANDA adapter Python module.
///
/// Loaded as `nautilus_pyo3.oanda`.
///
/// # Errors
///
/// Returns a `PyErr` if registering any module components fails.
#[pymodule]
pub fn oanda(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Enums
    m.add_class::<OANDAEnvironment>()?;

    // Config
    m.add_class::<OANDADataClientConfig>()?;
    m.add_class::<OANDAExecClientConfig>()?;

    // Clients
    m.add_class::<OANDAHttpClient>()?;
    m.add_class::<OANDAStreamClient>()?;

    // Helper functions
    m.add_function(wrap_pyfunction!(enums::py_oanda_environment_from_str, m)?)?;
    m.add_function(wrap_pyfunction!(enums::py_oanda_order_type_to_str, m)?)?;
    m.add_function(wrap_pyfunction!(enums::py_oanda_time_in_force_to_str, m)?)?;

    Ok(())
}
