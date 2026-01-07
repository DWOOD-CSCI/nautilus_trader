# OANDA Adapter for NautilusTrader

Integration adapter for OANDA forex broker.

## Features

- REST API v3 client for account, instruments, pricing, and order operations
- Streaming API client for real-time price feeds
- Support for forex and CFD instruments
- Full order lifecycle management (market, limit, stop orders)
- Position and account state tracking

## API Documentation

- [OANDA REST API v3](https://developer.oanda.com/rest-live-v20/introduction/)
- [OANDA Streaming API](https://developer.oanda.com/rest-live-v20/pricing-ep/)

## Configuration

Set environment variables:
- `OANDA_API_KEY` - Your OANDA API token
- `OANDA_ACCOUNT_ID` - Your OANDA account ID

For demo accounts, use the practice API endpoint (default).
For live accounts, set `OANDA_LIVE=true`.

## Python Usage

```python
from nautilus_trader.adapters.oanda import OANDADataClientConfig, OANDAExecClientConfig

data_config = OANDADataClientConfig(
    api_key="your-api-key",
    account_id="your-account-id",
    is_demo=True,
)

exec_config = OANDAExecClientConfig(
    api_key="your-api-key",
    account_id="your-account-id",
    is_demo=True,
)
```
