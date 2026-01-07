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
        assert!(!OANDAHttpError::InvalidParameters("test".to_string()).is_retryable());
        assert!(!OANDAHttpError::InsufficientFunds("test".to_string()).is_retryable());
        assert!(!OANDAHttpError::OrderRejected("test".to_string()).is_retryable());
    }

    #[test]
    fn test_error_display_messages() {
        let request_err = OANDAHttpError::Request("network error".to_string());
        assert!(request_err.to_string().contains("network error"));

        let deser_err = OANDAHttpError::Deserialization("invalid JSON".to_string());
        assert!(deser_err.to_string().contains("invalid JSON"));

        let api_err = OANDAHttpError::Api {
            error_type: "INVALID_ORDER".to_string(),
            message: "Order too large".to_string(),
        };
        assert!(api_err.to_string().contains("INVALID_ORDER"));
        assert!(api_err.to_string().contains("Order too large"));

        let auth_err = OANDAHttpError::Authentication("token expired".to_string());
        assert!(auth_err.to_string().contains("token expired"));

        let rate_err = OANDAHttpError::RateLimited;
        assert!(rate_err.to_string().contains("Rate limit"));

        let timeout_err = OANDAHttpError::Timeout;
        assert!(timeout_err.to_string().contains("timeout"));
    }

    #[test]
    fn test_from_api_response() {
        let err = OANDAHttpError::from_api_response("TEST_ERROR", "Test message");
        match err {
            OANDAHttpError::Api { error_type, message } => {
                assert_eq!(error_type, "TEST_ERROR");
                assert_eq!(message, "Test message");
            }
            _ => panic!("Expected Api variant"),
        }
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

    #[test]
    fn test_error_response_insufficient_margin() {
        let response = OANDAErrorResponse {
            error_type: Some("INSUFFICIENT_MARGIN".to_string()),
            error_message: Some("Margin insufficient".to_string()),
            reject_reason: None,
        };
        let error = response.into_error();
        assert!(matches!(error, OANDAHttpError::InsufficientFunds(_)));
    }

    #[test]
    fn test_error_response_invalid_argument() {
        let response = OANDAErrorResponse {
            error_type: Some("INVALID_ARGUMENT".to_string()),
            error_message: Some("Bad param".to_string()),
            reject_reason: None,
        };
        let error = response.into_error();
        assert!(matches!(error, OANDAHttpError::InvalidParameters(_)));
    }

    #[test]
    fn test_error_response_missing_argument() {
        let response = OANDAErrorResponse {
            error_type: Some("MISSING_ARGUMENT".to_string()),
            error_message: Some("Required field missing".to_string()),
            reject_reason: None,
        };
        let error = response.into_error();
        assert!(matches!(error, OANDAHttpError::InvalidParameters(_)));
    }

    #[test]
    fn test_error_response_unauthorized() {
        let response = OANDAErrorResponse {
            error_type: Some("UNAUTHORIZED".to_string()),
            error_message: Some("Invalid token".to_string()),
            reject_reason: None,
        };
        let error = response.into_error();
        assert!(matches!(error, OANDAHttpError::Authentication(_)));
    }

    #[test]
    fn test_error_response_forbidden() {
        let response = OANDAErrorResponse {
            error_type: Some("FORBIDDEN".to_string()),
            error_message: Some("Access denied".to_string()),
            reject_reason: None,
        };
        let error = response.into_error();
        assert!(matches!(error, OANDAHttpError::Authentication(_)));
    }

    #[test]
    fn test_error_response_rate_limit() {
        let response = OANDAErrorResponse {
            error_type: Some("RATE_LIMIT_EXCEEDED".to_string()),
            error_message: Some("Too many requests".to_string()),
            reject_reason: None,
        };
        let error = response.into_error();
        assert!(matches!(error, OANDAHttpError::RateLimited));
    }

    #[test]
    fn test_error_response_order_rejected() {
        let response = OANDAErrorResponse {
            error_type: Some("ORDER_REJECTED".to_string()),
            error_message: None,
            reject_reason: Some("Market closed".to_string()),
        };
        let error = response.into_error();
        match error {
            OANDAHttpError::OrderRejected(msg) => assert_eq!(msg, "Market closed"),
            _ => panic!("Expected OrderRejected variant"),
        }
    }

    #[test]
    fn test_error_response_unknown_type() {
        let response = OANDAErrorResponse {
            error_type: Some("CUSTOM_ERROR".to_string()),
            error_message: Some("Something weird".to_string()),
            reject_reason: None,
        };
        let error = response.into_error();
        match error {
            OANDAHttpError::Api { error_type, message } => {
                assert_eq!(error_type, "CUSTOM_ERROR");
                assert_eq!(message, "Something weird");
            }
            _ => panic!("Expected Api variant"),
        }
    }

    #[test]
    fn test_error_response_no_error_type() {
        let response = OANDAErrorResponse {
            error_type: None,
            error_message: Some("Something went wrong".to_string()),
            reject_reason: None,
        };
        let error = response.into_error();
        match error {
            OANDAHttpError::Api { error_type, message } => {
                assert_eq!(error_type, "UNKNOWN");
                assert_eq!(message, "Something went wrong");
            }
            _ => panic!("Expected Api variant with UNKNOWN error_type"),
        }
    }

    #[test]
    fn test_error_response_no_message() {
        let response = OANDAErrorResponse {
            error_type: Some("SOME_ERROR".to_string()),
            error_message: None,
            reject_reason: None,
        };
        let error = response.into_error();
        match error {
            OANDAHttpError::Api { error_type, message } => {
                assert_eq!(error_type, "SOME_ERROR");
                assert_eq!(message, "Unknown error");
            }
            _ => panic!("Expected Api variant"),
        }
    }

    #[test]
    fn test_error_response_deserialize() {
        let json = r#"{"errorType": "INVALID_ARGUMENT", "errorMessage": "Bad request"}"#;
        let response: OANDAErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.error_type, Some("INVALID_ARGUMENT".to_string()));
        assert_eq!(response.error_message, Some("Bad request".to_string()));
    }

    #[test]
    fn test_error_response_deserialize_with_reject_reason() {
        let json = r#"{"errorType": "ORDER_REJECTED", "rejectReason": "STOP_LOSS_ON_FILL_PRICE_DISTANCE_MAXIMUM_EXCEEDED"}"#;
        let response: OANDAErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.error_type, Some("ORDER_REJECTED".to_string()));
        assert!(response.reject_reason.is_some());
    }

    #[test]
    fn test_error_response_deserialize_empty() {
        let json = r#"{}"#;
        let response: OANDAErrorResponse = serde_json::from_str(json).unwrap();
        assert!(response.error_type.is_none());
        assert!(response.error_message.is_none());
        assert!(response.reject_reason.is_none());
    }
}
