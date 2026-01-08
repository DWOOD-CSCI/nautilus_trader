#!/usr/bin/env python3
"""
OANDA Integration Test Script

Tests the OANDA HTTP client Python bindings against the practice API.

Usage:
    # Set environment variables first:
    export OANDA_API_KEY="your-practice-api-key"
    export OANDA_ACCOUNT_ID="101-001-xxxxx-001"

    # Run the script:
    python test_oanda_integration.py

    # Or run specific tests:
    python test_oanda_integration.py --test account
    python test_oanda_integration.py --test instruments
    python test_oanda_integration.py --test pricing
    python test_oanda_integration.py --test candles
"""

import asyncio
import json
import os
import sys
from datetime import datetime


def get_credentials():
    """Get OANDA credentials from environment variables."""
    api_key = os.environ.get("OANDA_API_KEY") or os.environ.get("OANDA_TOKEN")
    account_id = os.environ.get("OANDA_ACCOUNT_ID")

    if not api_key or not account_id:
        print("ERROR: OANDA credentials not found!")
        print()
        print("Please set environment variables:")
        print("  export OANDA_API_KEY='your-practice-api-key'")
        print("  export OANDA_ACCOUNT_ID='101-001-xxxxx-001'")
        print()
        print("You can get a practice account at: https://www.oanda.com/demo-account/")
        sys.exit(1)

    return api_key, account_id


async def test_account_summary(client):
    """Test getting account summary."""
    print("\n=== Account Summary ===")
    try:
        result = await client.get_account_summary()
        data = json.loads(result)
        print(f"  Account ID: {data.get('id')}")
        print(f"  Balance: {data.get('balance')} {data.get('currency')}")
        print(f"  NAV: {data.get('nav')}")
        print(f"  Unrealized P/L: {data.get('unrealizedPL')}")
        print(f"  Margin Used: {data.get('marginUsed')}")
        print(f"  Margin Available: {data.get('marginAvailable')}")
        print("  ✓ Account summary test passed")
        return True
    except Exception as e:
        print(f"  ✗ Account summary test failed: {e}")
        return False


async def test_instruments(client):
    """Test getting instruments."""
    print("\n=== Instruments ===")
    try:
        result = await client.get_instruments()
        instruments = json.loads(result)
        print(f"  Total instruments available: {len(instruments)}")

        # Find some common pairs
        forex_pairs = [i for i in instruments if i["type"] == "CURRENCY"]
        print(f"  Forex pairs: {len(forex_pairs)}")

        # Show first 5
        print("  Sample instruments:")
        for inst in instruments[:5]:
            print(f"    - {inst['name']}: {inst['displayName']}")

        print("  ✓ Instruments test passed")
        return True
    except Exception as e:
        print(f"  ✗ Instruments test failed: {e}")
        return False


async def test_pricing(client):
    """Test getting current prices."""
    print("\n=== Pricing ===")
    try:
        instruments = ["EUR_USD", "GBP_USD", "USD_JPY"]
        result = await client.get_pricing(instruments)
        prices = json.loads(result)

        for price in prices:
            instrument = price["instrument"]
            bids = price.get("bids", [])
            asks = price.get("asks", [])
            bid = bids[0]["price"] if bids else "N/A"
            ask = asks[0]["price"] if asks else "N/A"
            tradeable = price.get("tradeable", False)
            print(f"  {instrument}: Bid={bid}, Ask={ask}, Tradeable={tradeable}")

        print("  ✓ Pricing test passed")
        return True
    except Exception as e:
        print(f"  ✗ Pricing test failed: {e}")
        return False


async def test_candles(client):
    """Test getting historical candles."""
    print("\n=== Historical Candles ===")
    try:
        result = await client.get_candles(instrument="EUR_USD", granularity="H1", count=10)
        candles = json.loads(result)

        print(f"  Retrieved {len(candles)} candles")
        if candles:
            latest = candles[-1]
            mid = latest.get("mid", {})
            print("  Latest candle:")
            print(f"    Time: {latest.get('time')}")
            print(f"    Open: {mid.get('o')}")
            print(f"    High: {mid.get('h')}")
            print(f"    Low: {mid.get('l')}")
            print(f"    Close: {mid.get('c')}")
            print(f"    Complete: {latest.get('complete')}")

        print("  ✓ Candles test passed")
        return True
    except Exception as e:
        print(f"  ✗ Candles test failed: {e}")
        return False


async def test_positions(client):
    """Test getting positions."""
    print("\n=== Positions ===")
    try:
        result = await client.get_open_positions()
        positions = json.loads(result)

        print(f"  Open positions: {len(positions)}")
        for pos in positions:
            instrument = pos["instrument"]
            long_units = pos.get("long", {}).get("units", "0")
            short_units = pos.get("short", {}).get("units", "0")
            print(f"    {instrument}: Long={long_units}, Short={short_units}")

        print("  ✓ Positions test passed")
        return True
    except Exception as e:
        print(f"  ✗ Positions test failed: {e}")
        return False


async def test_trades(client):
    """Test getting open trades."""
    print("\n=== Open Trades ===")
    try:
        result = await client.get_open_trades()
        trades = json.loads(result)

        print(f"  Open trades: {len(trades)}")
        for trade in trades[:5]:  # Show first 5
            trade_id = trade["id"]
            instrument = trade["instrument"]
            units = trade["currentUnits"]
            price = trade["price"]
            pl = trade.get("unrealizedPL", "N/A")
            print(f"    Trade {trade_id}: {units} {instrument} @ {price} (P/L: {pl})")

        print("  ✓ Trades test passed")
        return True
    except Exception as e:
        print(f"  ✗ Trades test failed: {e}")
        return False


async def run_all_tests():
    """Run all integration tests."""
    # Import the OANDA client
    try:
        from nautilus_trader.core.nautilus_pyo3.oanda import OANDAEnvironment, OANDAHttpClient
    except ImportError:
        print("ERROR: Could not import OANDA module!")
        print()
        print("Make sure you've built with Python bindings:")
        print("  cd nautilus_trader")
        print("  cargo build -p nautilus-oanda --features python")
        print()
        print("Or build the full package:")
        print("  pip install -e .[dev]")
        sys.exit(1)

    # Get credentials
    api_key, account_id = get_credentials()

    print("=" * 60)
    print("OANDA Integration Tests")
    print("=" * 60)
    print(f"Account: {account_id}")
    print("Environment: Practice")
    print(f"Time: {datetime.now().isoformat()}")

    # Create client
    try:
        client = OANDAHttpClient(
            api_key=api_key,
            account_id=account_id,
            environment=OANDAEnvironment.Practice,
            timeout_secs=30,
            max_retries=3,
        )
        print(f"Client created: {client}")
    except Exception as e:
        print(f"ERROR: Failed to create client: {e}")
        sys.exit(1)

    # Run tests
    results = []
    results.append(("Account Summary", await test_account_summary(client)))
    results.append(("Instruments", await test_instruments(client)))
    results.append(("Pricing", await test_pricing(client)))
    results.append(("Candles", await test_candles(client)))
    results.append(("Positions", await test_positions(client)))
    results.append(("Trades", await test_trades(client)))

    # Summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    passed = sum(1 for _, r in results if r)
    failed = len(results) - passed

    for name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        print(f"  {name}: {status}")

    print()
    print(f"Total: {passed} passed, {failed} failed")

    return failed == 0


async def run_specific_test(test_name):
    """Run a specific test."""
    from nautilus_trader.core.nautilus_pyo3.oanda import OANDAEnvironment, OANDAHttpClient

    api_key, account_id = get_credentials()

    client = OANDAHttpClient(
        api_key=api_key,
        account_id=account_id,
        environment=OANDAEnvironment.Practice,
        timeout_secs=30,
        max_retries=3,
    )

    tests = {
        "account": test_account_summary,
        "instruments": test_instruments,
        "pricing": test_pricing,
        "candles": test_candles,
        "positions": test_positions,
        "trades": test_trades,
    }

    if test_name not in tests:
        print(f"Unknown test: {test_name}")
        print(f"Available tests: {', '.join(tests.keys())}")
        sys.exit(1)

    await tests[test_name](client)


def main():
    import argparse

    parser = argparse.ArgumentParser(description="OANDA Integration Tests")
    parser.add_argument(
        "--test",
        "-t",
        help="Run specific test (account, instruments, pricing, candles, positions, trades)",
    )
    args = parser.parse_args()

    if args.test:
        asyncio.run(run_specific_test(args.test))
    else:
        success = asyncio.run(run_all_tests())
        sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
