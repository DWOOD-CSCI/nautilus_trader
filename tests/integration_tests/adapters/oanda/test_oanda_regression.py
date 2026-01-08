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
OANDA adapter regression tests.

These tests verify the OANDA adapter functionality against the live practice API.
Run with: pytest tests/integration_tests/adapters/oanda/test_oanda_regression.py -v

Requires environment variables:
  - OANDA_API_KEY or OANDA_TOKEN: Your OANDA API access token
  - OANDA_ACCOUNT_ID: Your OANDA account ID
"""

import json
import os

import pytest

from nautilus_trader.core.nautilus_pyo3.oanda import (
    OANDAEnvironment,
    OANDAHttpClient,
)


# Skip all tests if credentials not available
pytestmark = pytest.mark.skipif(
    not (os.environ.get("OANDA_API_KEY") or os.environ.get("OANDA_TOKEN"))
    or not os.environ.get("OANDA_ACCOUNT_ID"),
    reason="OANDA credentials not set (OANDA_API_KEY/OANDA_TOKEN and OANDA_ACCOUNT_ID required)",
)


@pytest.fixture
def oanda_client():
    """Create an OANDA HTTP client for testing."""
    api_key = os.environ.get("OANDA_API_KEY") or os.environ.get("OANDA_TOKEN")
    account_id = os.environ.get("OANDA_ACCOUNT_ID")
    
    return OANDAHttpClient(
        api_key=api_key,
        account_id=account_id,
        environment=OANDAEnvironment.Practice,
    )


class TestOANDAHttpClient:
    """Test suite for OANDA HTTP client."""
    
    @pytest.mark.asyncio
    async def test_get_account_summary(self, oanda_client):
        """Test fetching account summary."""
        result = await oanda_client.get_account_summary()
        
        assert result is not None
        assert len(result) > 0
        
        data = json.loads(result)
        # Rust adapter returns flattened account data
        assert "id" in data
        assert "balance" in data
        assert "currency" in data
    
    @pytest.mark.asyncio
    async def test_get_instruments(self, oanda_client):
        """Test fetching available instruments."""
        result = await oanda_client.get_instruments()
        
        assert result is not None
        assert len(result) > 0
        
        data = json.loads(result)
        # Rust adapter returns list of instruments directly
        assert isinstance(data, list)
        assert len(data) > 0
        
        # Check for common FX pairs
        instrument_names = [i["name"] for i in data]
        assert "EUR_USD" in instrument_names
        assert "GBP_USD" in instrument_names
    
    @pytest.mark.asyncio
    async def test_get_candles_with_count(self, oanda_client):
        """Test fetching candles with count parameter."""
        result = await oanda_client.get_candles(
            instrument="EUR_USD",
            granularity="M1",
            count=10,
        )
        
        assert result is not None
        data = json.loads(result)
        
        # Rust adapter returns list of candles directly
        assert isinstance(data, list)
        assert len(data) == 10
        
        # Verify candle structure
        candle = data[0]
        assert "time" in candle
        assert "volume" in candle
        assert "complete" in candle
        assert "mid" in candle or "bid" in candle or "ask" in candle
    
    @pytest.mark.asyncio
    async def test_get_candles_different_granularities(self, oanda_client):
        """Test fetching candles with different time granularities."""
        granularities = ["S5", "M1", "M5", "H1", "D"]
        
        for gran in granularities:
            result = await oanda_client.get_candles(
                instrument="EUR_USD",
                granularity=gran,
                count=5,
            )
            
            data = json.loads(result)
            assert isinstance(data, list)
            assert len(data) == 5
    
    @pytest.mark.asyncio
    async def test_get_pricing(self, oanda_client):
        """Test fetching current pricing for instruments."""
        result = await oanda_client.get_pricing(["EUR_USD", "GBP_USD"])
        
        assert result is not None
        data = json.loads(result)
        
        # Rust adapter returns list of prices directly
        assert isinstance(data, list)
        assert len(data) >= 1  # At least one price
        
        # Verify price structure
        price = data[0]
        assert "instrument" in price
        # Check for bid/ask data
        has_price_data = any(k in price for k in ["bids", "asks", "closeoutBid", "closeoutAsk"])
        assert has_price_data
    
    @pytest.mark.asyncio
    async def test_get_pending_orders(self, oanda_client):
        """Test fetching pending orders (may be empty)."""
        result = await oanda_client.get_pending_orders()
        
        assert result is not None
        data = json.loads(result)
        
        # Orders returned as dict with 'orders' key or as list
        if isinstance(data, dict):
            assert "orders" in data
            assert isinstance(data["orders"], list)
        else:
            assert isinstance(data, list)
    
    @pytest.mark.asyncio
    async def test_get_open_trades(self, oanda_client):
        """Test fetching open trades (may be empty)."""
        result = await oanda_client.get_open_trades()
        
        assert result is not None
        data = json.loads(result)
        
        # Trades returned as list (may be empty)
        assert isinstance(data, list)
    
    @pytest.mark.asyncio
    async def test_get_open_positions(self, oanda_client):
        """Test fetching open positions (may be empty)."""
        result = await oanda_client.get_open_positions()
        
        assert result is not None
        data = json.loads(result)
        
        # Positions returned as list (may be empty)
        assert isinstance(data, list)


class TestOANDAEnvironment:
    """Test suite for OANDA environment enum."""
    
    def test_environment_values(self):
        """Test that environment enum has expected values."""
        assert OANDAEnvironment.Practice is not None
        assert OANDAEnvironment.Live is not None
    
    def test_environment_from_str(self):
        """Test environment string parsing."""
        from nautilus_trader.core.nautilus_pyo3.oanda import oanda_environment_from_str
        
        # Test lowercase
        practice = oanda_environment_from_str("practice")
        assert practice == OANDAEnvironment.Practice
        
        live = oanda_environment_from_str("live")
        assert live == OANDAEnvironment.Live


class TestOANDADataIntegrity:
    """Test suite for OANDA data integrity checks."""
    
    @pytest.mark.asyncio
    async def test_candle_data_completeness(self, oanda_client):
        """Test that candle data has all required OHLC fields."""
        result = await oanda_client.get_candles(
            instrument="EUR_USD",
            granularity="M1",
            count=5,
        )
        
        data = json.loads(result)
        assert isinstance(data, list)
        
        for candle in data:
            assert "time" in candle
            assert "volume" in candle
            assert candle["complete"] in [True, False]
            
            if "mid" in candle and candle["mid"] is not None:
                mid = candle["mid"]
                assert "o" in mid  # Open
                assert "h" in mid  # High
                assert "l" in mid  # Low
                assert "c" in mid  # Close
                
                # Verify OHLC relationships
                o, h, l, c = float(mid["o"]), float(mid["h"]), float(mid["l"]), float(mid["c"])
                assert h >= l, f"High {h} should be >= Low {l}"
                assert h >= o, f"High {h} should be >= Open {o}"
                assert h >= c, f"High {h} should be >= Close {c}"
                assert l <= o, f"Low {l} should be <= Open {o}"
                assert l <= c, f"Low {l} should be <= Close {c}"
    
    @pytest.mark.asyncio
    async def test_instrument_data_structure(self, oanda_client):
        """Test that instrument data has required fields."""
        result = await oanda_client.get_instruments()
        data = json.loads(result)
        
        # Check EUR_USD specifically
        eur_usd = next(
            (i for i in data if i["name"] == "EUR_USD"),
            None
        )
        
        assert eur_usd is not None
        assert "name" in eur_usd
        assert "type" in eur_usd
        assert "displayName" in eur_usd
        assert "pipLocation" in eur_usd
        assert "displayPrecision" in eur_usd


class TestOANDABacktest:
    """Test suite for OANDA data backtest integration."""
    
    @pytest.mark.asyncio
    async def test_backtest_with_oanda_data(self, oanda_client):
        """Test that OANDA data can be used in a basic backtest."""
        from decimal import Decimal
        
        from nautilus_trader.backtest.engine import BacktestEngine, BacktestEngineConfig
        from nautilus_trader.backtest.models import FillModel
        from nautilus_trader.model.enums import AccountType, OmsType
        from nautilus_trader.config import LoggingConfig
        from nautilus_trader.examples.strategies.ema_cross import EMACross, EMACrossConfig
        from nautilus_trader.model.currencies import USD
        from nautilus_trader.model.data import Bar, BarType
        from nautilus_trader.model.identifiers import InstrumentId, TraderId, Venue
        from nautilus_trader.model.objects import Money, Price, Quantity
        from nautilus_trader.test_kit.providers import TestInstrumentProvider
        
        # Fetch a small sample of candle data
        result = await oanda_client.get_candles(
            instrument="EUR_USD",
            granularity="M5",
            count=100,
        )
        candles = json.loads(result)
        assert len(candles) >= 100, "Should fetch at least 100 candles"
        
        # Create backtest engine with minimal logging
        config = BacktestEngineConfig(
            trader_id=TraderId("BACKTESTER-OANDA-TEST"),
            logging=LoggingConfig(log_level="ERROR"),  # Suppress logs during test
        )
        engine = BacktestEngine(config=config)
        
        # Add venue with OANDA identifier
        OANDA = Venue("OANDA")
        engine.add_venue(
            venue=OANDA,
            oms_type=OmsType.NETTING,
            account_type=AccountType.MARGIN,
            base_currency=USD,
            starting_balances=[Money(10_000, USD)],
            fill_model=FillModel(),
        )
        
        # Use standard EUR/USD instrument
        EURUSD = TestInstrumentProvider.default_fx_ccy("EUR/USD", OANDA)
        engine.add_instrument(EURUSD)
        
        # Convert OANDA candles to NautilusTrader bars
        bar_type = BarType.from_str("EUR/USD.OANDA-5-MINUTE-MID-EXTERNAL")
        bars = []
        
        for candle in candles:
            if not candle.get("complete", False):
                continue
            
            mid = candle.get("mid")
            if not mid:
                continue
            
            # Parse timestamp
            ts_str = candle["time"]
            from datetime import datetime
            dt = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
            ts_ns = int(dt.timestamp() * 1_000_000_000)
            
            bar = Bar(
                bar_type=bar_type,
                open=Price.from_str(mid["o"]),
                high=Price.from_str(mid["h"]),
                low=Price.from_str(mid["l"]),
                close=Price.from_str(mid["c"]),
                volume=Quantity.from_int(int(candle.get("volume", 0))),
                ts_event=ts_ns,
                ts_init=ts_ns,
            )
            bars.append(bar)
        
        assert len(bars) >= 50, f"Should have at least 50 complete bars, got {len(bars)}"
        
        # Add bars to engine
        engine.add_data(bars)
        
        # Add EMA cross strategy with fast settings for test
        strategy = EMACross(
            config=EMACrossConfig(
                instrument_id=InstrumentId.from_str("EUR/USD.OANDA"),
                bar_type=bar_type,
                fast_ema_period=5,
                slow_ema_period=10,
                trade_size=Decimal("1000"),
            )
        )
        engine.add_strategy(strategy)
        
        # Run backtest
        engine.run()
        
        # Verify backtest completed
        assert engine.iteration > 0, "Engine should have processed iterations"
        
        # Get account state
        account = engine.cache.account_for_venue(OANDA)
        assert account is not None, "Account should exist after backtest"
        
        # Cleanup
        engine.reset()
        engine.dispose()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
