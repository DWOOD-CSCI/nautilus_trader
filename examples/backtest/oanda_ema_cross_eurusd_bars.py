#!/usr/bin/env python3
# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------
"""
OANDA EMA Cross Backtest Example.

This example demonstrates:
1. Fetching historical candle data from OANDA API
2. Converting OANDA data to NautilusTrader Bar format
3. Running a backtest with the EMACross strategy

Requires environment variables:
  - OANDA_API_KEY or OANDA_TOKEN: Your OANDA API access token
  - OANDA_ACCOUNT_ID: Your OANDA account ID

Usage:
    cd nautilus_trader
    source .venv/bin/activate
    export $(grep -v '^#' ../.env | sed 's/ *= */=/' | xargs)
    python examples/backtest/oanda_ema_cross_eurusd_bars.py
"""

import asyncio
import json
import os
from datetime import datetime, timedelta
from decimal import Decimal

import pandas as pd

from nautilus_trader.backtest.config import BacktestEngineConfig
from nautilus_trader.backtest.engine import BacktestEngine
from nautilus_trader.core.nautilus_pyo3.oanda import OANDAEnvironment, OANDAHttpClient
from nautilus_trader.examples.strategies.ema_cross import EMACross, EMACrossConfig
from nautilus_trader.model.currencies import USD
from nautilus_trader.model.data import Bar, BarSpecification, BarType
from nautilus_trader.model.enums import AccountType, AggregationSource, BarAggregation, OmsType, PriceType
from nautilus_trader.model.identifiers import InstrumentId, Symbol, TraderId, Venue
from nautilus_trader.model.instruments import CurrencyPair
from nautilus_trader.model.objects import Money, Price, Quantity
from nautilus_trader.test_kit.providers import TestInstrumentProvider


# OANDA venue
OANDA = Venue("OANDA")


def create_oanda_instrument(symbol: str = "EUR/USD") -> CurrencyPair:
    """Create an OANDA FX instrument."""
    return TestInstrumentProvider.default_fx_ccy(symbol, OANDA)


async def fetch_oanda_candles(
    instrument: str = "EUR_USD",
    granularity: str = "M5",  # 5-minute bars
    count: int = 500,
) -> list[dict]:
    """Fetch candle data from OANDA API."""
    api_key = os.environ.get("OANDA_API_KEY") or os.environ.get("OANDA_TOKEN")
    account_id = os.environ.get("OANDA_ACCOUNT_ID")
    
    if not api_key or not account_id:
        raise ValueError(
            "OANDA credentials not set. "
            "Set OANDA_API_KEY/OANDA_TOKEN and OANDA_ACCOUNT_ID environment variables."
        )
    
    client = OANDAHttpClient(
        api_key=api_key,
        account_id=account_id,
        environment=OANDAEnvironment.Practice,
    )
    
    print(f"Fetching {count} {granularity} candles for {instrument}...")
    result = await client.get_candles(
        instrument=instrument,
        granularity=granularity,
        count=count,
    )
    
    candles = json.loads(result)
    print(f"Received {len(candles)} candles")
    return candles


def convert_oanda_candles_to_bars(
    candles: list[dict],
    instrument: CurrencyPair,
    bar_spec: BarSpecification,
) -> list[Bar]:
    """Convert OANDA candle data to NautilusTrader Bar objects."""
    bars = []
    bar_type = BarType(instrument.id, bar_spec, AggregationSource.EXTERNAL)
    
    for candle in candles:
        if not candle.get("complete", False):
            continue  # Skip incomplete candles
            
        # Use mid prices
        mid = candle.get("mid")
        if not mid:
            continue
            
        timestamp = pd.Timestamp(candle["time"])
        ts_event = timestamp.value  # nanoseconds
        ts_init = ts_event
        
        bar = Bar(
            bar_type=bar_type,
            open=Price.from_str(mid["o"]),
            high=Price.from_str(mid["h"]),
            low=Price.from_str(mid["l"]),
            close=Price.from_str(mid["c"]),
            volume=Quantity.from_int(int(candle.get("volume", 0))),
            ts_event=ts_event,
            ts_init=ts_init,
        )
        bars.append(bar)
    
    return bars


async def main():
    """Run OANDA EMA Cross backtest."""
    print("=" * 60)
    print("OANDA EMA Cross Backtest")
    print("=" * 60)
    
    # Fetch data from OANDA
    oanda_candles = await fetch_oanda_candles(
        instrument="EUR_USD",
        granularity="M5",  # 5-minute bars
        count=500,
    )
    
    if not oanda_candles:
        print("No candles received from OANDA. Check credentials.")
        return
    
    # Create instrument
    EURUSD_OANDA = create_oanda_instrument("EUR/USD")
    
    # Create bar specification matching our data
    bar_spec = BarSpecification(
        step=5,
        aggregation=BarAggregation.MINUTE,
        price_type=PriceType.MID,
    )
    
    # Convert OANDA candles to NautilusTrader bars
    bars = convert_oanda_candles_to_bars(oanda_candles, EURUSD_OANDA, bar_spec)
    print(f"Converted {len(bars)} bars for backtest")
    
    if len(bars) < 50:
        print("Not enough bars for backtest (need at least 50)")
        return
    
    # Configure backtest engine
    config = BacktestEngineConfig(
        trader_id=TraderId("BACKTESTER-OANDA-001"),
    )
    engine = BacktestEngine(config=config)
    
    # Add OANDA venue
    engine.add_venue(
        venue=OANDA,
        oms_type=OmsType.HEDGING,
        account_type=AccountType.MARGIN,
        base_currency=USD,
        starting_balances=[Money(100_000, USD)],
    )
    
    # Add instrument
    engine.add_instrument(EURUSD_OANDA)
    
    # Add bar data
    engine.add_data(bars)
    
    # Configure EMA Cross strategy
    bar_type = BarType(
        EURUSD_OANDA.id,
        bar_spec,
        AggregationSource.EXTERNAL,
    )
    
    strategy_config = EMACrossConfig(
        instrument_id=EURUSD_OANDA.id,
        bar_type=bar_type,
        fast_ema_period=10,
        slow_ema_period=20,
        trade_size=Decimal(10_000),  # Mini lot
    )
    strategy = EMACross(config=strategy_config)
    engine.add_strategy(strategy=strategy)
    
    print("\n" + "=" * 60)
    print("Running backtest...")
    print("=" * 60)
    
    # Run the backtest
    engine.run()
    
    # Generate and print reports
    print("\n" + "=" * 60)
    print("BACKTEST RESULTS")
    print("=" * 60)
    
    with pd.option_context(
        "display.max_rows", 50,
        "display.max_columns", None,
        "display.width", 300,
    ):
        print("\n--- Account Report ---")
        print(engine.trader.generate_account_report(OANDA))
        
        print("\n--- Order Fills Report ---")
        print(engine.trader.generate_order_fills_report())
        
        print("\n--- Positions Report ---")
        print(engine.trader.generate_positions_report())
    
    # Clean up
    engine.reset()
    engine.dispose()
    
    print("\n✅ Backtest complete!")


if __name__ == "__main__":
    asyncio.run(main())
