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

//! API credentials for OANDA authentication.

use zeroize::{Zeroize, ZeroizeOnDrop};

/// API credentials for OANDA authentication.
///
/// OANDA uses a simple Bearer token authentication scheme.
/// Tokens can be generated from the OANDA account portal.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct OANDACredential {
    api_token: String,
    account_id: String,
}

impl OANDACredential {
    /// Creates a new credential with the given API token and account ID.
    #[must_use]
    pub fn new(api_token: impl Into<String>, account_id: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
            account_id: account_id.into(),
        }
    }

    /// Load credentials from environment variables.
    ///
    /// Looks for `OANDA_API_KEY` (or `OANDA_API_TOKEN`) and `OANDA_ACCOUNT_ID`.
    ///
    /// Returns `None` if either token or account ID is not set.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("OANDA_API_KEY")
            .or_else(|_| std::env::var("OANDA_API_TOKEN"))
            .ok()?;
        let account_id = std::env::var("OANDA_ACCOUNT_ID").ok()?;

        Some(Self::new(token, account_id))
    }

    /// Load credentials from environment variables for a specific environment.
    ///
    /// For practice: `OANDA_PRACTICE_API_KEY` and `OANDA_PRACTICE_ACCOUNT_ID`
    /// For live: `OANDA_LIVE_API_KEY` and `OANDA_LIVE_ACCOUNT_ID`
    ///
    /// Falls back to generic `OANDA_API_KEY` and `OANDA_ACCOUNT_ID` if specific vars not found.
    #[must_use]
    pub fn from_env_with_environment(is_live: bool) -> Option<Self> {
        let prefix = if is_live { "OANDA_LIVE" } else { "OANDA_PRACTICE" };

        let token = std::env::var(format!("{prefix}_API_KEY"))
            .or_else(|_| std::env::var("OANDA_API_KEY"))
            .or_else(|_| std::env::var("OANDA_API_TOKEN"))
            .ok()?;

        let account_id = std::env::var(format!("{prefix}_ACCOUNT_ID"))
            .or_else(|_| std::env::var("OANDA_ACCOUNT_ID"))
            .ok()?;

        Some(Self::new(token, account_id))
    }

    /// Resolves credentials from provided values or environment.
    ///
    /// If both `api_token` and `account_id` are provided, uses those.
    /// Otherwise, attempts to load from environment variables.
    #[must_use]
    pub fn resolve(
        api_token: Option<String>,
        account_id: Option<String>,
        is_live: bool,
    ) -> Option<Self> {
        match (api_token, account_id) {
            (Some(token), Some(account)) => Some(Self::new(token, account)),
            _ => Self::from_env_with_environment(is_live),
        }
    }

    /// Returns the API token.
    #[must_use]
    pub fn api_token(&self) -> &str {
        &self.api_token
    }

    /// Returns the account ID.
    #[must_use]
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    /// Returns the Authorization header value for HTTP requests.
    #[must_use]
    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_credential() {
        let cred = OANDACredential::new("test-token", "test-account");
        assert_eq!(cred.api_token(), "test-token");
        assert_eq!(cred.account_id(), "test-account");
    }

    #[test]
    fn test_auth_header() {
        let cred = OANDACredential::new("my-secret-token", "123-456");
        assert_eq!(cred.auth_header(), "Bearer my-secret-token");
    }

    #[test]
    fn test_resolve_with_values() {
        let cred = OANDACredential::resolve(
            Some("token".to_string()),
            Some("account".to_string()),
            false,
        );
        assert!(cred.is_some());
        let cred = cred.unwrap();
        assert_eq!(cred.api_token(), "token");
        assert_eq!(cred.account_id(), "account");
    }
}
