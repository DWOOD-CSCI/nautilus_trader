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

//! Error types for OANDA HTTP operations.

use thiserror::Error;

/// Error type for OANDA HTTP client operations.
#[derive(Debug, Error)]
pub enum OANDAHttpError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Request(String),

    /// Failed to deserialize response.
    #[error("Failed to deserialize response: {0}")]
    Deserialization(String),

    /// OANDA API returned an error.
    #[error("OANDA API error: {error_type} - {message}")]
    Api {
        error_type: String,
        message: String,
    },

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimited,

    /// Invalid request parameters.
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    /// Instrument not found.
    #[error("Instrument not found: {0}")]
    InstrumentNotFound(String),

    /// Order rejected.
    #[error("Order rejected: {0}")]
    OrderRejected(String),

    /// Insufficient funds.
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Timeout error.
    #[error("Request timeout")]
    Timeout,

    /// Unknown error.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl OANDAHttpError {
    /// Returns true if this error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::Connection(_) | Self::Timeout
        )
    }

    /// Creates an API error from OANDA error response fields.
    #[must_use]
    pub fn from_api_response(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Api {
            error_type: error_type.into(),
            message: message.into(),
        }
    }
}

/// OANDA API error response structure.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OANDAErrorResponse {
    /// Error type identifier.
    pub error_type: Option<String>,

    /// Error message.
    pub error_message: Option<String>,

    /// Reject reason (for order rejections).
    pub reject_reason: Option<String>,
}

impl OANDAErrorResponse {
    /// Converts this response into an `OANDAHttpError`.
    #[must_use]
    pub fn into_error(self) -> OANDAHttpError {
        let error_type = self.error_type.unwrap_or_else(|| "UNKNOWN".to_string());
        let message = self
            .error_message
            .or(self.reject_reason)
            .unwrap_or_else(|| "Unknown error".to_string());

        match error_type.as_str() {
            "INVALID_ARGUMENT" | "MISSING_ARGUMENT" => {
                OANDAHttpError::InvalidParameters(message)
            }
            "UNAUTHORIZED" | "FORBIDDEN" => OANDAHttpError::Authentication(message),
            "RATE_LIMIT_EXCEEDED" => OANDAHttpError::RateLimited,
            "INSUFFICIENT_FUNDS" | "INSUFFICIENT_MARGIN" => {
                OANDAHttpError::InsufficientFunds(message)
            }
            "ORDER_REJECTED" => OANDAHttpError::OrderRejected(message),
            _ => OANDAHttpError::Api { error_type, message },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable() {
        assert!(OANDAHttpError::RateLimited.is_retryable());
        assert!(OANDAHttpError::Timeout.is_retryable());
        assert!(OANDAHttpError::Connection("test".to_string()).is_retryable());
        assert!(!OANDAHttpError::Authentication("test".to_string()).is_retryable());
    }

    #[test]
    fn test_error_response_into_error() {
        let response = OANDAErrorResponse {
            error_type: Some("INSUFFICIENT_FUNDS".to_string()),
            error_message: Some("Not enough margin".to_string()),
            reject_reason: None,
        };

        let error = response.into_error();
        assert!(matches!(error, OANDAHttpError::InsufficientFunds(_)));
    }
}
