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

//! OANDA HTTP client for REST API v3.

use std::collections::HashMap;
use std::time::Duration;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    Client, Method, Response, StatusCode,
};
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde_json::json;
use log::{debug, error, trace, warn};

use crate::common::{
    consts::OANDA_API_VERSION,
    credential::OANDACredential,
    enums::{OANDAEnvironment, OANDAGranularity, OANDAPriceComponent, OANDATimeInForce},
    urls::get_oanda_rest_url,
};
use crate::http::{
    error::{OANDAErrorResponse, OANDAHttpError},
    models::{
        AccountDetails, AccountResponse, AccountSummary, AccountSummaryResponse, Candle,
        CandlesResponse, ClientExtensions, InstrumentDetails, InstrumentsResponse, OrderResponse,
        Position, PositionsResponse, Price, PricingResponse, Trade, TradesResponse,
    },
};

/// HTTP client for the OANDA REST API.
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.oanda", frozen)
)]
#[derive(Debug, Clone)]
pub struct OANDAHttpClient {
    /// HTTP client.
    client: Client,
    /// Base URL for API requests.
    base_url: String,
    /// Account ID.
    pub account_id: String,
    /// Maximum retry attempts.
    max_retries: u32,
    /// Initial retry delay in milliseconds.
    initial_retry_delay_ms: u64,
}

impl OANDAHttpClient {
    /// Creates a new `OANDAHttpClient`.
    ///
    /// # Arguments
    /// * `credential` - OANDA API credential
    /// * `account_id` - OANDA account ID
    /// * `environment` - Trading environment (Practice or Live)
    /// * `timeout_secs` - Request timeout in seconds (default: 30)
    /// * `max_retries` - Maximum retry attempts for retryable errors (default: 3)
    pub fn new(
        credential: OANDACredential,
        account_id: String,
        environment: OANDAEnvironment,
        timeout_secs: Option<u64>,
        max_retries: Option<u32>,
    ) -> Result<Self, OANDAHttpError> {
        let is_live = matches!(environment, OANDAEnvironment::Live);
        let base_url = get_oanda_rest_url(is_live).to_string();
        let timeout = Duration::from_secs(timeout_secs.unwrap_or(30));

        // Build default headers
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&credential.auth_header())
                .map_err(|e| OANDAHttpError::Authentication(e.to_string()))?,
        );
        default_headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        default_headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json"),
        );
        default_headers.insert(
            USER_AGENT,
            HeaderValue::from_static("nautilus-trader/1.0"),
        );
        default_headers.insert(
            HeaderName::from_static("accept-datetime-format"),
            HeaderValue::from_static("RFC3339"),
        );

        // Build HTTP client
        let client = Client::builder()
            .timeout(timeout)
            .default_headers(default_headers)
            .build()
            .map_err(|e| OANDAHttpError::Connection(e.to_string()))?;

        debug!(
            "Created OANDA HTTP client for account {} ({} environment)",
            account_id,
            if is_live { "live" } else { "practice" }
        );

        Ok(Self {
            client,
            base_url,
            account_id,
            max_retries: max_retries.unwrap_or(3),
            initial_retry_delay_ms: 100,
        })
    }

    /// Returns the account ID.
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    /// Returns the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // ============================================================================================
    // HTTP Request Methods
    // ============================================================================================

    /// Sends a GET request to the specified endpoint.
    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, OANDAHttpError> {
        self.request(Method::GET, endpoint, None::<&()>).await
    }

    /// Sends a GET request with query parameters.
    async fn get_with_params<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T, OANDAHttpError> {
        let url = self.build_url_with_params(endpoint, params);
        self.request_url(Method::GET, &url, None::<&()>).await
    }

    /// Sends a POST request with JSON body.
    async fn post<T: DeserializeOwned, B: serde::Serialize + std::fmt::Debug>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, OANDAHttpError> {
        self.request(Method::POST, endpoint, Some(body)).await
    }

    /// Sends a PUT request with JSON body.
    async fn put<T: DeserializeOwned, B: serde::Serialize + std::fmt::Debug>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, OANDAHttpError> {
        self.request(Method::PUT, endpoint, Some(body)).await
    }

    /// Core request method with retry logic.
    async fn request<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&B>,
    ) -> Result<T, OANDAHttpError> {
        let url = format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'));
        self.request_url(method, &url, body).await
    }

    /// Core request method with full URL.
    async fn request_url<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        method: Method,
        url: &str,
        body: Option<&B>,
    ) -> Result<T, OANDAHttpError> {
        let mut attempts = 0;

        loop {
            attempts += 1;
            trace!("Request attempt {}/{}: {} {}", attempts, self.max_retries, method, url);

            let mut request = self.client.request(method.clone(), url);

            if let Some(b) = body {
                request = request.json(b);
            }

            let response: Response = request.send().await.map_err(|e| {
                if e.is_timeout() {
                    OANDAHttpError::Timeout
                } else if e.is_connect() {
                    OANDAHttpError::Connection(e.to_string())
                } else {
                    OANDAHttpError::Request(e.to_string())
                }
            })?;

            match self.handle_response(response).await {
                Ok(result) => return Ok(result),
                Err(e) if e.is_retryable() && attempts < self.max_retries => {
                    // Exponential backoff: initial_delay * 2^attempts
                    let delay = Duration::from_millis(self.initial_retry_delay_ms * 2u64.pow(attempts));
                    warn!("Retryable error (attempt {}): {:?}, retrying in {:?}", attempts, e, delay);
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Handles API response, parsing errors if status is not success.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T, OANDAHttpError> {
        let status = response.status();
        let url = response.url().to_string();

        if status.is_success() {
            let text = response.text().await.map_err(|e| {
                OANDAHttpError::Deserialization(format!("Failed to read response body: {}", e))
            })?;

            trace!("Response body: {}", &text[..text.len().min(500)]);

            serde_json::from_str(&text).map_err(|e| {
                error!("Failed to deserialize response: {}", e);
                error!("Response text: {}", text);
                OANDAHttpError::Deserialization(format!("{}: {}", e, &text[..text.len().min(200)]))
            })
        } else {
            let text = response.text().await.unwrap_or_default();

            // Try to parse structured OANDA error response
            if let Ok(error_response) = serde_json::from_str::<OANDAErrorResponse>(&text) {
                return Err(error_response.into_error());
            }

            // Handle HTTP status codes when no structured error is available
            match status {
                StatusCode::UNAUTHORIZED => {
                    Err(OANDAHttpError::Authentication("Invalid or expired API token".to_string()))
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    Err(OANDAHttpError::RateLimited)
                }
                StatusCode::BAD_REQUEST => {
                    Err(OANDAHttpError::InvalidParameters(text))
                }
                StatusCode::NOT_FOUND => {
                    Err(OANDAHttpError::from_api_response(status.to_string(), format!("Resource not found: {}", url)))
                }
                _ => {
                    Err(OANDAHttpError::from_api_response(status.to_string(), text))
                }
            }
        }
    }

    /// Builds URL with query parameters.
    ///
    /// Parameters are URL-encoded to handle special characters safely.
    fn build_url_with_params(&self, endpoint: &str, params: &[(&str, &str)]) -> String {
        let base = format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'));
        if params.is_empty() {
            base
        } else {
            let query = serde_urlencoded::to_string(params).unwrap_or_default();
            format!("{}?{}", base, query)
        }
    }

    // ============================================================================================
    // Account Endpoints
    // ============================================================================================

    /// Gets full account details.
    pub async fn get_account(&self) -> Result<AccountDetails, OANDAHttpError> {
        let endpoint = format!("{}/accounts/{}", OANDA_API_VERSION, self.account_id);
        let response: AccountResponse = self.get(&endpoint).await?;
        Ok(response.account)
    }

    /// Gets account summary.
    pub async fn get_account_summary(&self) -> Result<AccountSummary, OANDAHttpError> {
        let endpoint = format!("{}/accounts/{}/summary", OANDA_API_VERSION, self.account_id);
        let response: AccountSummaryResponse = self.get(&endpoint).await?;
        Ok(response.account)
    }

    // ============================================================================================
    // Instrument Endpoints
    // ============================================================================================

    /// Gets all tradeable instruments for the account.
    pub async fn get_instruments(&self) -> Result<Vec<InstrumentDetails>, OANDAHttpError> {
        let endpoint = format!("{}/accounts/{}/instruments", OANDA_API_VERSION, self.account_id);
        let response: InstrumentsResponse = self.get(&endpoint).await?;
        Ok(response.instruments)
    }

    /// Gets specific instruments.
    pub async fn get_instruments_by_name(
        &self,
        instruments: &[&str],
    ) -> Result<Vec<InstrumentDetails>, OANDAHttpError> {
        let instruments_str = instruments.join(",");
        let endpoint = format!("{}/accounts/{}/instruments", OANDA_API_VERSION, self.account_id);
        let params = [("instruments", instruments_str.as_str())];
        let response: InstrumentsResponse = self.get_with_params(&endpoint, &params).await?;
        Ok(response.instruments)
    }

    // ============================================================================================
    // Pricing Endpoints
    // ============================================================================================

    /// Gets current prices for specified instruments.
    pub async fn get_pricing(&self, instruments: &[&str]) -> Result<Vec<Price>, OANDAHttpError> {
        let instruments_str = instruments.join(",");
        let endpoint = format!("{}/accounts/{}/pricing", OANDA_API_VERSION, self.account_id);
        let params = [("instruments", instruments_str.as_str())];
        let response: PricingResponse = self.get_with_params(&endpoint, &params).await?;
        Ok(response.prices)
    }

    /// Gets historical candles for an instrument.
    pub async fn get_candles(
        &self,
        instrument: &str,
        granularity: OANDAGranularity,
        price_component: Option<OANDAPriceComponent>,
        count: Option<i32>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<Candle>, OANDAHttpError> {
        let endpoint = format!(
            "{}/instruments/{}/candles",
            OANDA_API_VERSION,
            instrument
        );

        let mut params: Vec<(&str, String)> = vec![
            ("granularity", granularity.as_str().to_string()),
        ];

        if let Some(price) = price_component {
            params.push(("price", price.as_str().to_string()));
        }

        if let Some(c) = count {
            params.push(("count", c.to_string()));
        }

        if let Some(f) = from {
            params.push(("from", f.to_string()));
        }

        if let Some(t) = to {
            params.push(("to", t.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let response: CandlesResponse = self.get_with_params(&endpoint, &params_ref).await?;
        Ok(response.candles)
    }

    // ============================================================================================
    // Order Endpoints
    // ============================================================================================

    /// Creates a market order.
    pub async fn create_market_order(
        &self,
        instrument: &str,
        units: Decimal,
        time_in_force: Option<OANDATimeInForce>,
        client_extensions: Option<ClientExtensions>,
    ) -> Result<OrderResponse, OANDAHttpError> {
        let endpoint = format!("{}/accounts/{}/orders", OANDA_API_VERSION, self.account_id);

        let mut order = json!({
            "type": "MARKET",
            "instrument": instrument,
            "units": units.to_string(),
            "timeInForce": time_in_force.unwrap_or(OANDATimeInForce::Fok).as_str(),
        });

        if let Some(ext) = client_extensions {
            order["clientExtensions"] = serde_json::to_value(ext).unwrap();
        }

        let body = json!({ "order": order });
        self.post(&endpoint, &body).await
    }

    /// Creates a limit order.
    pub async fn create_limit_order(
        &self,
        instrument: &str,
        units: Decimal,
        price: Decimal,
        time_in_force: Option<OANDATimeInForce>,
        gtd_time: Option<&str>,
        client_extensions: Option<ClientExtensions>,
    ) -> Result<OrderResponse, OANDAHttpError> {
        let endpoint = format!("{}/accounts/{}/orders", OANDA_API_VERSION, self.account_id);
        let tif = time_in_force.unwrap_or(OANDATimeInForce::Gtc);

        let mut order = json!({
            "type": "LIMIT",
            "instrument": instrument,
            "units": units.to_string(),
            "price": price.to_string(),
            "timeInForce": tif.as_str(),
        });

        if matches!(tif, OANDATimeInForce::Gtd) {
            if let Some(gtd) = gtd_time {
                order["gtdTime"] = json!(gtd);
            }
        }

        if let Some(ext) = client_extensions {
            order["clientExtensions"] = serde_json::to_value(ext).unwrap();
        }

        let body = json!({ "order": order });
        self.post(&endpoint, &body).await
    }

    /// Creates a stop order.
    pub async fn create_stop_order(
        &self,
        instrument: &str,
        units: Decimal,
        price: Decimal,
        time_in_force: Option<OANDATimeInForce>,
        gtd_time: Option<&str>,
        client_extensions: Option<ClientExtensions>,
    ) -> Result<OrderResponse, OANDAHttpError> {
        let endpoint = format!("{}/accounts/{}/orders", OANDA_API_VERSION, self.account_id);
        let tif = time_in_force.unwrap_or(OANDATimeInForce::Gtc);

        let mut order = json!({
            "type": "STOP",
            "instrument": instrument,
            "units": units.to_string(),
            "price": price.to_string(),
            "timeInForce": tif.as_str(),
        });

        if matches!(tif, OANDATimeInForce::Gtd) {
            if let Some(gtd) = gtd_time {
                order["gtdTime"] = json!(gtd);
            }
        }

        if let Some(ext) = client_extensions {
            order["clientExtensions"] = serde_json::to_value(ext).unwrap();
        }

        let body = json!({ "order": order });
        self.post(&endpoint, &body).await
    }

    /// Cancels a pending order.
    pub async fn cancel_order(&self, order_id: &str) -> Result<OrderResponse, OANDAHttpError> {
        let endpoint = format!(
            "{}/accounts/{}/orders/{}/cancel",
            OANDA_API_VERSION, self.account_id, order_id
        );
        let body: HashMap<String, String> = HashMap::new();
        self.put(&endpoint, &body).await
    }

    // ============================================================================================
    // Position Endpoints
    // ============================================================================================

    /// Gets all open positions.
    pub async fn get_open_positions(&self) -> Result<Vec<Position>, OANDAHttpError> {
        let endpoint = format!(
            "{}/accounts/{}/openPositions",
            OANDA_API_VERSION, self.account_id
        );
        let response: PositionsResponse = self.get(&endpoint).await?;
        Ok(response.positions)
    }

    /// Gets all positions (including closed).
    pub async fn get_positions(&self) -> Result<Vec<Position>, OANDAHttpError> {
        let endpoint = format!(
            "{}/accounts/{}/positions",
            OANDA_API_VERSION, self.account_id
        );
        let response: PositionsResponse = self.get(&endpoint).await?;
        Ok(response.positions)
    }

    /// Closes a position for an instrument.
    pub async fn close_position(
        &self,
        instrument: &str,
        long_units: Option<&str>,
        short_units: Option<&str>,
    ) -> Result<OrderResponse, OANDAHttpError> {
        let endpoint = format!(
            "{}/accounts/{}/positions/{}/close",
            OANDA_API_VERSION, self.account_id, instrument
        );

        let mut body = json!({});
        if let Some(lu) = long_units {
            body["longUnits"] = json!(lu);
        }
        if let Some(su) = short_units {
            body["shortUnits"] = json!(su);
        }

        self.put(&endpoint, &body).await
    }

    // ============================================================================================
    // Trade Endpoints
    // ============================================================================================

    /// Gets all open trades.
    pub async fn get_open_trades(&self) -> Result<Vec<Trade>, OANDAHttpError> {
        let endpoint = format!(
            "{}/accounts/{}/openTrades",
            OANDA_API_VERSION, self.account_id
        );
        let response: TradesResponse = self.get(&endpoint).await?;
        Ok(response.trades)
    }

    /// Closes a specific trade.
    pub async fn close_trade(
        &self,
        trade_id: &str,
        units: Option<Decimal>,
    ) -> Result<OrderResponse, OANDAHttpError> {
        let endpoint = format!(
            "{}/accounts/{}/trades/{}/close",
            OANDA_API_VERSION, self.account_id, trade_id
        );

        let body = if let Some(u) = units {
            json!({ "units": u.to_string() })
        } else {
            json!({ "units": "ALL" })
        };

        self.put(&endpoint, &body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client() -> OANDAHttpClient {
        let credential = OANDACredential::new("test-token".to_string(), "test-account".to_string());
        OANDAHttpClient::new(
            credential,
            "test-account".to_string(),
            OANDAEnvironment::Practice,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_client_creation() {
        let client = create_test_client();
        assert_eq!(client.account_id(), "test-account");
        assert!(client.base_url().contains("fxpractice.oanda.com"));
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.initial_retry_delay_ms, 100);
    }

    #[test]
    fn test_client_creation_live() {
        let credential = OANDACredential::new("test-token".to_string(), "test-account".to_string());
        let client = OANDAHttpClient::new(
            credential,
            "test-account".to_string(),
            OANDAEnvironment::Live,
            Some(60),
            Some(5),
        )
        .unwrap();
        assert!(client.base_url().contains("fxtrade.oanda.com"));
        assert_eq!(client.max_retries, 5);
    }

    #[test]
    fn test_build_url_with_params() {
        let client = create_test_client();

        let url = client.build_url_with_params(
            "/v3/accounts/test/pricing",
            &[("instruments", "EUR_USD,GBP_USD")],
        );
        // Commas are URL-encoded to %2C by serde_urlencoded
        assert!(url.contains("instruments=EUR_USD%2CGBP_USD"));

        let url_no_params = client.build_url_with_params("/v3/accounts/test", &[]);
        assert!(!url_no_params.contains('?'));
    }

    #[test]
    fn test_build_url_with_multiple_params() {
        let client = create_test_client();

        let url = client.build_url_with_params(
            "/v3/instruments/EUR_USD/candles",
            &[("granularity", "M1"), ("count", "100"), ("price", "MBA")],
        );
        assert!(url.contains("granularity=M1"));
        assert!(url.contains("count=100"));
        assert!(url.contains("price=MBA"));
        assert!(url.contains("&"));
    }

    #[test]
    fn test_build_url_strips_leading_slash() {
        let client = create_test_client();

        let url1 = client.build_url_with_params("/v3/accounts", &[]);
        let url2 = client.build_url_with_params("v3/accounts", &[]);
        
        // Both should produce valid URLs without double slashes
        assert!(!url1.contains("//v3"));
        assert!(!url2.contains("//v3"));
    }

    #[test]
    fn test_build_url_with_special_characters() {
        let client = create_test_client();

        // Test that special characters are properly URL-encoded
        let url = client.build_url_with_params(
            "/v3/test",
            &[("param", "value with spaces"), ("other", "a=b&c=d")],
        );
        // Spaces become + or %20, & becomes %26, = becomes %3D
        assert!(url.contains("value+with+spaces") || url.contains("value%20with%20spaces"));
        assert!(url.contains("%26") || url.contains("a%3Db"));
    }

    #[test]
    fn test_client_accessors() {
        let client = create_test_client();
        assert_eq!(client.account_id(), "test-account");
        assert!(client.base_url().starts_with("https://"));
        assert!(client.base_url().contains("oanda.com"));
    }

    #[test]
    fn test_client_retry_config() {
        let credential = OANDACredential::new("test-token".to_string(), "test-account".to_string());
        
        // Default retries
        let client1 = OANDAHttpClient::new(
            credential.clone(),
            "test-account".to_string(),
            OANDAEnvironment::Practice,
            None,
            None,
        ).unwrap();
        assert_eq!(client1.max_retries, 3);
        assert_eq!(client1.initial_retry_delay_ms, 100);

        // Custom retries
        let client2 = OANDAHttpClient::new(
            credential,
            "test-account".to_string(),
            OANDAEnvironment::Practice,
            None,
            Some(10),
        ).unwrap();
        assert_eq!(client2.max_retries, 10);
    }
}
