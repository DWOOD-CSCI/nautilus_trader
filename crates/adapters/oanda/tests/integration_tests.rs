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

//! Integration tests for OANDA HTTP client.
//!
//! These tests require valid OANDA practice credentials:
//! - OANDA_API_KEY: Your OANDA API access token
//! - OANDA_ACCOUNT_ID: Your OANDA practice account ID
//!
//! Run with: cargo test -p nautilus-oanda --test integration_tests --features python
//!
//! Tests are ignored by default. To run them:
//! cargo test -p nautilus-oanda --test integration_tests --features python -- --ignored

use nautilus_oanda::{
    common::{credential::OANDACredential, enums::OANDAEnvironment},
    http::client::OANDAHttpClient,
};

/// Helper to get test credentials from environment
fn get_test_credentials() -> Option<(String, String)> {
    let api_key = std::env::var("OANDA_API_KEY")
        .or_else(|_| std::env::var("OANDA_TOKEN"))
        .ok()?;
    let account_id = std::env::var("OANDA_ACCOUNT_ID").ok()?;

    if api_key.is_empty() || account_id.is_empty() {
        return None;
    }

    Some((api_key, account_id))
}

/// Create a test client using environment credentials
fn create_test_client() -> Option<OANDAHttpClient> {
    let (api_key, account_id) = get_test_credentials()?;
    let credential = OANDACredential::new(api_key, account_id.clone());

    OANDAHttpClient::new(
        credential,
        account_id,
        OANDAEnvironment::Practice,
        Some(30),
        Some(3),
    )
    .ok()
}

// ================================================================================================
// Account Tests
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_account_summary() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_account_summary().await;
    assert!(result.is_ok(), "Failed to get account summary: {:?}", result.err());

    let summary = result.unwrap();
    println!("Account ID: {}", summary.id);
    println!("Balance: {}", summary.balance);
    println!("Currency: {}", summary.currency);
    println!("NAV: {}", summary.nav);

    // Verify account ID matches
    assert_eq!(summary.id, client.account_id());
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_account_details() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_account().await;
    assert!(result.is_ok(), "Failed to get account details: {:?}", result.err());

    let account = result.unwrap();
    println!("Account ID: {}", account.id);
    println!("Alias: {:?}", account.alias);
    println!("Created: {:?}", account.created_time);
}

// ================================================================================================
// Instrument Tests
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_instruments() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_instruments().await;
    assert!(result.is_ok(), "Failed to get instruments: {:?}", result.err());

    let instruments = result.unwrap();
    println!("Total instruments available: {}", instruments.len());

    // Should have many instruments
    assert!(instruments.len() > 10, "Expected more than 10 instruments");

    // Look for common forex pairs
    let eur_usd = instruments.iter().find(|i| i.name == "EUR_USD");
    assert!(eur_usd.is_some(), "EUR_USD should be available");

    if let Some(pair) = eur_usd {
        println!("EUR_USD: display_name={}, pip_location={}", pair.display_name, pair.pip_location);
    }
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_specific_instruments() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_instruments_by_name(&["EUR_USD", "GBP_USD"]).await;
    assert!(result.is_ok(), "Failed to get specific instruments: {:?}", result.err());

    let instruments = result.unwrap();
    assert_eq!(instruments.len(), 2, "Expected exactly 2 instruments");

    let names: Vec<&str> = instruments.iter().map(|i| i.name.as_str()).collect();
    assert!(names.contains(&"EUR_USD"));
    assert!(names.contains(&"GBP_USD"));
}

// ================================================================================================
// Pricing Tests
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_pricing() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_pricing(&["EUR_USD", "GBP_USD"]).await;
    assert!(result.is_ok(), "Failed to get pricing: {:?}", result.err());

    let prices = result.unwrap();
    assert_eq!(prices.len(), 2, "Expected 2 price quotes");

    for price in &prices {
        println!(
            "{}: bid={:?}, ask={:?}, time={}",
            price.instrument, price.bids, price.asks, price.time
        );

        // Prices should have bid/ask levels
        assert!(!price.bids.is_empty() || !price.asks.is_empty(), "Expected bid or ask prices");
    }
}

// ================================================================================================
// Candle Tests
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_candles() {
    use nautilus_oanda::common::enums::OANDAGranularity;

    let client = create_test_client().expect("OANDA credentials required");

    let result = client
        .get_candles("EUR_USD", OANDAGranularity::H1, None, Some(10), None, None)
        .await;

    assert!(result.is_ok(), "Failed to get candles: {:?}", result.err());

    let candles = result.unwrap();
    assert!(!candles.is_empty(), "Expected at least one candle");
    assert!(candles.len() <= 10, "Requested max 10 candles");

    println!("Retrieved {} candles", candles.len());
    if let Some(candle) = candles.first() {
        println!(
            "First candle: time={}, complete={}",
            candle.time, candle.complete
        );
    }
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_candles_with_price_component() {
    use nautilus_oanda::common::enums::{OANDAGranularity, OANDAPriceComponent};

    let client = create_test_client().expect("OANDA credentials required");

    // Request mid, bid, and ask prices
    let result = client
        .get_candles(
            "EUR_USD",
            OANDAGranularity::M1,
            Some(OANDAPriceComponent::All),  // MBA - Mid, Bid, Ask
            Some(5),
            None,
            None,
        )
        .await;

    assert!(result.is_ok(), "Failed to get candles with MBA: {:?}", result.err());

    let candles = result.unwrap();
    println!("Retrieved {} candles with MBA pricing", candles.len());
}

// ================================================================================================
// Position Tests
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_open_positions() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_open_positions().await;
    assert!(result.is_ok(), "Failed to get open positions: {:?}", result.err());

    let positions = result.unwrap();
    println!("Open positions: {}", positions.len());

    for pos in &positions {
        println!("  {} - long: {:?}, short: {:?}", pos.instrument, pos.long, pos.short);
    }
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_all_positions() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_positions().await;
    assert!(result.is_ok(), "Failed to get all positions: {:?}", result.err());

    let positions = result.unwrap();
    println!("Total positions (including closed): {}", positions.len());
}

// ================================================================================================
// Trade Tests
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_open_trades() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_open_trades().await;
    assert!(result.is_ok(), "Failed to get open trades: {:?}", result.err());

    let trades = result.unwrap();
    println!("Open trades: {}", trades.len());

    for trade in &trades {
        println!(
            "  Trade {}: {} units of {} @ {}",
            trade.id, trade.current_units, trade.instrument, trade.price
        );
    }
}

// ================================================================================================
// Order Tests (Read-only - doesn't create actual orders)
// ================================================================================================

// Note: Order creation tests are commented out by default to avoid accidental trades.
// Uncomment and run manually if you want to test order creation on a practice account.

/*
#[tokio::test]
#[ignore = "Requires OANDA practice credentials - CREATES REAL ORDER"]
async fn test_create_market_order() {
    use rust_decimal_macros::dec;

    let client = create_test_client().expect("OANDA credentials required");

    // Small position - 1 unit of EUR_USD
    let result = client
        .create_market_order("EUR_USD", dec!(1), None, None)
        .await;

    assert!(result.is_ok(), "Failed to create market order: {:?}", result.err());

    let response = result.unwrap();
    println!("Order response: {:?}", response);
}
*/

// ================================================================================================
// Error Handling Tests
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_invalid_instrument() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_pricing(&["INVALID_PAIR_XYZ"]).await;

    // Should return an error for invalid instrument
    assert!(result.is_err(), "Expected error for invalid instrument");
    println!("Error as expected: {:?}", result.err());
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_empty_instruments() {
    let client = create_test_client().expect("OANDA credentials required");

    // Empty instrument list should work (returns empty)
    let result = client.get_pricing(&[]).await;
    println!("Result for empty instruments: {:?}", result);
}

// ================================================================================================
// Authentication Test
// ================================================================================================

#[tokio::test]
async fn test_invalid_credentials() {
    let credential = OANDACredential::new("invalid-token".to_string(), "invalid-account".to_string());
    let client = OANDAHttpClient::new(
        credential,
        "invalid-account".to_string(),
        OANDAEnvironment::Practice,
        Some(10),
        Some(1),
    )
    .unwrap();

    let result = client.get_account_summary().await;
    assert!(result.is_err(), "Expected authentication error");

    let err = result.err().unwrap();
    println!("Authentication error: {}", err);
}

// ================================================================================================
// Order Management Tests (Phase 1.5)
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_pending_orders() {
    let client = create_test_client().expect("OANDA credentials required");

    let result = client.get_pending_orders().await;
    assert!(result.is_ok(), "Failed to get pending orders: {:?}", result.err());

    let response = result.unwrap();
    println!("Pending orders: {}", response.orders.len());

    for order in &response.orders {
        println!(
            "  Order {}: type={}, state={:?}, instrument={:?}",
            order.id, order.order_type, order.state, order.instrument
        );
    }
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_orders_filtered() {
    let client = create_test_client().expect("OANDA credentials required");

    // Get all orders (not just pending)
    let result = client.get_orders(
        Some("EUR_USD"),  // Filter by instrument
        None,              // No ID filter
        Some("ALL"),       // All states
        Some(10),          // Limit to 10
        None,              // No pagination
    ).await;

    assert!(result.is_ok(), "Failed to get filtered orders: {:?}", result.err());

    let response = result.unwrap();
    println!("EUR_USD orders: {}", response.orders.len());
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_get_trades_filtered() {
    let client = create_test_client().expect("OANDA credentials required");

    // Get all trades (not just open)
    let result = client.get_trades(
        None,              // All instruments
        None,              // No ID filter
        Some("ALL"),       // All states (OPEN, CLOSED, CLOSE_WHEN_TRADEABLE)
        Some(10),          // Limit to 10
        None,              // No pagination
    ).await;

    assert!(result.is_ok(), "Failed to get filtered trades: {:?}", result.err());

    let response = result.unwrap();
    println!("Total trades (up to 10): {}", response.trades.len());

    for trade in &response.trades {
        println!(
            "  Trade {}: {} {} @ {} (state={:?})",
            trade.id, trade.current_units, trade.instrument, trade.price, trade.state
        );
    }
}

// ================================================================================================
// Risk Management Order Tests (Phase 1.5)
// These tests create real orders - run manually on practice account only
// ================================================================================================

/*
#[tokio::test]
#[ignore = "Requires OANDA practice credentials - CREATES REAL ORDERS"]
async fn test_create_market_if_touched_order() {
    use rust_decimal_macros::dec;

    let client = create_test_client().expect("OANDA credentials required");

    // Get current price first
    let prices = client.get_pricing(&["EUR_USD"]).await.expect("Failed to get pricing");
    let current_price = prices[0].bids[0].price.parse::<rust_decimal::Decimal>().unwrap();
    
    // Create MIT order below current price (for a buy)
    let trigger_price = current_price - dec!(0.0010);

    let result = client
        .create_market_if_touched_order(
            "EUR_USD",
            dec!(1),  // 1 unit buy
            trigger_price,
            None,     // Default TIF
            None,     // No GTD time
            None,     // No price bound
            None,     // No client extensions
        )
        .await;

    if result.is_err() {
        println!("MIT order error (may be expected): {:?}", result.err());
    } else {
        let response = result.unwrap();
        println!("MIT order response: {:?}", response);
        
        // Cancel the order to clean up
        if let Some(tx) = response.order_create_transaction {
            let _ = client.cancel_order(&tx.id).await;
            println!("Order {} cancelled", tx.id);
        }
    }
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials - CREATES REAL ORDERS"]
async fn test_trade_with_risk_management_orders() {
    use rust_decimal_macros::dec;

    let client = create_test_client().expect("OANDA credentials required");

    // Step 1: Create a market order to get an open trade
    let order_result = client
        .create_market_order("EUR_USD", dec!(1), None, None)
        .await;

    if order_result.is_err() {
        println!("Market order failed: {:?}", order_result.err());
        return;
    }

    let order_response = order_result.unwrap();
    println!("Market order created: {:?}", order_response.order_fill_transaction);

    // Get the trade ID from the fill transaction
    let trade_id = order_response
        .order_fill_transaction
        .and_then(|fill| Some(fill.order_id))
        .expect("No fill transaction");

    // Wait a moment for order to process
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Get open trades to find the trade
    let trades = client.get_open_trades().await.expect("Failed to get trades");
    println!("Open trades: {:?}", trades);

    if let Some(trade) = trades.first() {
        let trade_id = &trade.id;
        let open_price: rust_decimal::Decimal = trade.price.parse().unwrap();

        // Step 2: Add take profit order
        let tp_price = open_price + dec!(0.0050);  // 50 pips above
        let tp_result = client
            .create_take_profit_order(trade_id, tp_price, None, None, None)
            .await;
        println!("Take profit result: {:?}", tp_result);

        // Step 3: Add stop loss order
        let sl_price = open_price - dec!(0.0030);  // 30 pips below
        let sl_result = client
            .create_stop_loss_order(trade_id, Some(sl_price), None, None, None, None, None)
            .await;
        println!("Stop loss result: {:?}", sl_result);

        // Step 4: Modify the trade orders
        let new_tp_price = open_price + dec!(0.0060);  // Change to 60 pips
        let modify_result = client
            .modify_trade_orders(
                trade_id,
                Some(Some(new_tp_price)),  // New TP price
                None,                       // Keep SL unchanged
                None,                       // No TSL
            )
            .await;
        println!("Modify orders result: {:?}", modify_result);

        // Step 5: Close the trade to clean up
        let close_result = client.close_trade(trade_id, None).await;
        println!("Close trade result: {:?}", close_result);
    }
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials - CREATES REAL ORDERS"]
async fn test_create_trailing_stop_loss() {
    use rust_decimal_macros::dec;

    let client = create_test_client().expect("OANDA credentials required");

    // First create a trade
    let order_result = client
        .create_market_order("EUR_USD", dec!(1), None, None)
        .await;

    if order_result.is_err() {
        println!("Market order failed: {:?}", order_result.err());
        return;
    }

    // Get the trade
    let trades = client.get_open_trades().await.expect("Failed to get trades");
    
    if let Some(trade) = trades.first() {
        // Create trailing stop loss with 20 pip distance
        let tsl_result = client
            .create_trailing_stop_loss_order(
                &trade.id,
                dec!(0.0020),  // 20 pip trailing distance
                None,          // Default TIF
                None,          // No GTD time
                None,          // No client extensions
            )
            .await;

        match tsl_result {
            Ok(response) => println!("TSL created: {:?}", response),
            Err(e) => println!("TSL error (may need higher distance): {:?}", e),
        }

        // Close trade to clean up
        let _ = client.close_trade(&trade.id, None).await;
    }
}
*/

// ================================================================================================
// Transaction Streaming Tests (Phase 1.5.4)
// ================================================================================================

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_transaction_stream_connect() {
    use nautilus_oanda::websocket::{OANDATransactionStreamClient, TransactionStreamConfig};

    let (api_key, account_id) = get_test_credentials().expect("OANDA credentials required");

    let config = TransactionStreamConfig::new(
        OANDAEnvironment::Practice,
        &api_key,
        &account_id,
    );

    let mut client = OANDATransactionStreamClient::new(config);

    // Connect to transaction stream
    let result = client.connect().await;
    assert!(result.is_ok(), "Failed to connect to transaction stream: {:?}", result.err());

    assert!(client.is_running());
    println!("Transaction stream connected successfully");

    // Wait for a heartbeat (should arrive within 5-10 seconds)
    let timeout_result = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        client.next(),
    ).await;

    match timeout_result {
        Ok(Some(Ok(msg))) => {
            println!("Received message: {:?}", msg);
        }
        Ok(Some(Err(e))) => {
            println!("Stream error: {:?}", e);
        }
        Ok(None) => {
            println!("Stream ended");
        }
        Err(_) => {
            println!("Timeout waiting for message (expected in quiet account)");
        }
    }

    // Disconnect
    client.disconnect().await;
    assert!(!client.is_running());
    println!("Transaction stream disconnected");
}

#[tokio::test]
#[ignore = "Requires OANDA practice credentials"]
async fn test_transaction_stream_config_validation() {
    use nautilus_oanda::websocket::TransactionStreamConfig;

    // Valid config
    let config = TransactionStreamConfig::new(
        OANDAEnvironment::Practice,
        "test-key",
        "test-account",
    );
    assert!(config.validate().is_ok());

    // Invalid - missing API key
    let mut invalid_config = TransactionStreamConfig::default();
    invalid_config.account_id = "test".into();
    assert!(invalid_config.validate().is_err());
}
