# OANDA Adapter for NautilusTrader

Integration adapter for OANDA forex broker with full REST API v3 and streaming support.

## Features

- **REST API v3 Client**: Full HTTP client for account, instruments, pricing, and order operations
- **Streaming Client**: Real-time price feeds via HTTP streaming API
- **Python Bindings**: Full async Python support via PyO3
- **Order Types**: Market, limit, and stop orders with client extensions
- **Position Management**: Track and close positions by instrument
- **Error Handling**: Comprehensive error types with automatic retry for transient failures

## Installation

The adapter is included as part of the NautilusTrader project. Build with:

```bash
cargo build -p nautilus-oanda

# With Python bindings
cargo build -p nautilus-oanda --features python

# Run tests
cargo test -p nautilus-oanda --features python
```

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `OANDA_API_KEY` | Your OANDA API access token | Yes |
| `OANDA_ACCOUNT_ID` | Your OANDA account ID | Yes |
| `OANDA_LIVE` | Set to `true` for live trading | No (default: false) |

### Rust Usage

```rust
use nautilus_oanda::{
    common::{credential::OANDACredential, enums::OANDAEnvironment},
    http::client::OANDAHttpClient,
};

// Create client
let credential = OANDACredential::new(api_key, account_id.clone());
let client = OANDAHttpClient::new(
    credential,
    account_id,
    OANDAEnvironment::Practice,  // or OANDAEnvironment::Live
    Some(30),  // timeout in seconds
    Some(3),   // max retries
)?;

// Get account summary
let summary = client.get_account_summary().await?;

// Get instrument pricing
let prices = client.get_pricing(&["EUR_USD", "GBP_USD"]).await?;

// Create a market order
let order = client.create_market_order("EUR_USD", dec!(10000), None, None).await?;
```

### Python Usage

```python
import asyncio
from nautilus_trader.core.nautilus_pyo3.oanda import OANDAHttpClient, OANDAEnvironment

async def main():
    # Create client
    client = OANDAHttpClient(
        api_key="your-api-key",
        account_id="your-account-id",
        environment=OANDAEnvironment.Practice,
        timeout_secs=30,
        max_retries=3,
    )
    
    # Get account summary (returns JSON string)
    summary = await client.get_account_summary()
    print(summary)
    
    # Get instruments
    instruments = await client.get_instruments()
    
    # Get pricing
    prices = await client.get_pricing(["EUR_USD", "GBP_USD"])
    
    # Get candles
    candles = await client.get_candles(
        instrument="EUR_USD",
        granularity="H1",
        count=100,
    )
    
    # Create market order (units as string)
    order = await client.create_market_order("EUR_USD", "10000")
    
    # Create limit order
    limit_order = await client.create_limit_order("EUR_USD", "10000", "1.0850")
    
    # Create stop order
    stop_order = await client.create_stop_order("EUR_USD", "-10000", "1.0750")
    
    # Close position
    await client.close_position("EUR_USD", long_units="ALL")
    
asyncio.run(main())
```

## API Endpoints Supported

### Account
- `get_account()` - Full account details
- `get_account_summary()` - Account summary (balance, margin, etc.)

### Instruments
- `get_instruments()` - All tradeable instruments
- `get_instruments_by_name(instruments)` - Specific instruments
- `get_candles(instrument, granularity, ...)` - Historical OHLC data

### Pricing
- `get_pricing(instruments)` - Current bid/ask prices
- Streaming prices via `OANDAStreamClient`

### Orders
- `create_market_order(instrument, units, ...)` - Execute at market
- `create_limit_order(instrument, units, price, ...)` - Limit order
- `create_stop_order(instrument, units, price, ...)` - Stop order
- `cancel_order(order_id)` - Cancel pending order

### Positions
- `get_open_positions()` - Current open positions
- `get_positions()` - All positions including closed
- `close_position(instrument, long_units, short_units)` - Close position

### Trades
- `get_open_trades()` - Open trades
- `close_trade(trade_id, units)` - Close specific trade

## Streaming

```rust
use nautilus_oanda::websocket::{OANDAStreamClient, StreamConfig, StreamMessage};

// Configure streaming
let config = StreamConfig::new(
    OANDAEnvironment::Practice,
    "api-key",
    "account-id",
    vec!["EUR_USD".into(), "GBP_USD".into()],
);

// Create client and connect
let mut client = OANDAStreamClient::new(config);
let mut receiver = client.connect().await?;

// Process messages
while let Some(result) = receiver.recv().await {
    match result {
        Ok(StreamMessage::Price(price)) => {
            println!("Price: {} bid={} ask={}", price.instrument, price.bid, price.ask);
        }
        Ok(StreamMessage::Heartbeat(ts)) => {
            println!("Heartbeat: {}", ts);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Error Handling

The adapter uses typed errors for different failure modes:

```rust
pub enum OANDAHttpError {
    Request(String),           // HTTP request failed
    Deserialization(String),   // Failed to parse response
    Api { error_type, message }, // OANDA API error
    Authentication(String),    // Auth failure
    RateLimited,               // Rate limit exceeded (retryable)
    InvalidParameters(String), // Bad request parameters
    OrderRejected(String),     // Order was rejected
    InsufficientFunds(String), // Margin error
    Connection(String),        // Connection error (retryable)
    Timeout,                   // Request timeout (retryable)
}
```

Automatic retry is performed for `RateLimited`, `Connection`, and `Timeout` errors with exponential backoff.

## Testing

Run the test suite:

```bash
# All tests
cargo test -p nautilus-oanda --features python

# Specific test module
cargo test -p nautilus-oanda --features python http::client

# With output
cargo test -p nautilus-oanda --features python -- --nocapture
```

Current test coverage: 50 tests covering:
- HTTP client (URL building, parameter encoding, retry config)
- Error types and conversions
- Streaming client configuration
- Credential management
- Environment URLs

## API Documentation

- [OANDA REST API v3](https://developer.oanda.com/rest-live-v20/introduction/)
- [OANDA Streaming API](https://developer.oanda.com/rest-live-v20/pricing-ep/)
- [OANDA Order API](https://developer.oanda.com/rest-live-v20/order-ep/)

## License

Licensed under the GNU Lesser General Public License Version 3.0.
