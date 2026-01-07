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

//! Python bindings for OANDA enums.

use pyo3::prelude::*;
use std::str::FromStr;

use crate::common::enums::{
    OANDAEnvironment, OANDAOrderType, OANDATimeInForce,
};

/// Parse an OANDA environment from a string.
///
/// # Parameters
///
/// * `value` - The string value to parse ("practice" or "live")
///
/// # Returns
///
/// The corresponding `OANDAEnvironment` enum variant.
///
/// # Errors
///
/// Returns a `PyErr` if the string cannot be parsed.
#[pyfunction(name = "oanda_environment_from_str")]
pub fn py_oanda_environment_from_str(value: &str) -> PyResult<OANDAEnvironment> {
    OANDAEnvironment::from_str(value)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid environment: {e}")))
}

/// Convert an OANDA order type to its string representation.
///
/// # Parameters
///
/// * `order_type` - The order type to convert
///
/// # Returns
///
/// The string representation of the order type.
#[pyfunction(name = "oanda_order_type_to_str")]
pub fn py_oanda_order_type_to_str(order_type: &str) -> PyResult<String> {
    let parsed = OANDAOrderType::from_str(order_type)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid order type: {e}")))?;
    Ok(parsed.to_string())
}

/// Convert an OANDA time in force to its string representation.
///
/// # Parameters
///
/// * `tif` - The time in force to convert
///
/// # Returns
///
/// The string representation of the time in force.
#[pyfunction(name = "oanda_time_in_force_to_str")]
pub fn py_oanda_time_in_force_to_str(tif: &str) -> PyResult<String> {
    let parsed = OANDATimeInForce::from_str(tif)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid time in force: {e}")))?;
    Ok(parsed.to_string())
}
