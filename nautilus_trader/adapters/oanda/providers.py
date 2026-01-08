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

"""OANDA instrument provider."""

from __future__ import annotations

import json
from decimal import Decimal
from typing import TYPE_CHECKING, Any

from nautilus_trader.adapters.oanda.config import OANDAInstrumentProviderConfig
from nautilus_trader.adapters.oanda.constants import OANDA_VENUE
from nautilus_trader.common.providers import InstrumentProvider
from nautilus_trader.core.correctness import PyCondition
from nautilus_trader.model.identifiers import InstrumentId, Symbol
from nautilus_trader.model.instruments import CurrencyPair
from nautilus_trader.model.objects import Currency, Price, Quantity

if TYPE_CHECKING:
    pass


class OANDAInstrumentProvider(InstrumentProvider):
    """
    Provides Nautilus instrument definitions from OANDA.

    Parameters
    ----------
    client : nautilus_pyo3.oanda.OANDAHttpClient
        The OANDA HTTP client (from Rust adapter).
    config : OANDAInstrumentProviderConfig, optional
        The instrument provider configuration.

    """

    def __init__(
        self,
        client: Any,  # OANDAHttpClient from nautilus_pyo3.oanda
        config: OANDAInstrumentProviderConfig | None = None,
    ) -> None:
        super().__init__(config=config)
        self._client = client
        self._config: OANDAInstrumentProviderConfig = config or OANDAInstrumentProviderConfig()
        self._raw_instruments: list[dict] = []

    @property
    def raw_instruments(self) -> list[dict]:
        """
        Return the raw OANDA instrument definitions.

        Returns
        -------
        list[dict]

        """
        return self._raw_instruments

    def _parse_instrument(self, data: dict) -> CurrencyPair | None:
        """
        Parse an OANDA instrument response into a Nautilus CurrencyPair.

        Parameters
        ----------
        data : dict
            The raw OANDA instrument data.

        Returns
        -------
        CurrencyPair | None
            The parsed instrument, or None if parsing failed.

        """
        try:
            # OANDA instrument name is like "EUR_USD"
            name = data.get("name", "")
            if "_" not in name:
                self._log.warning(f"Skipping non-currency pair instrument: {name}")
                return None

            # Parse currencies
            parts = name.split("_")
            if len(parts) != 2:
                self._log.warning(f"Unexpected instrument format: {name}")
                return None

            base_currency_code = parts[0]
            quote_currency_code = parts[1]

            # Create symbol and instrument ID
            symbol = Symbol(name)
            instrument_id = InstrumentId(symbol=symbol, venue=OANDA_VENUE)

            # Parse precision from display precision
            display_precision = data.get("displayPrecision", 5)
            trade_units_precision = data.get("tradeUnitsPrecision", 0)

            # Get min/max trade sizes
            minimum_trade_size = data.get("minimumTradeSize", "1")
            maximum_trade_units = data.get("maximumTradeUnits", "100000000")

            # Price precision
            price_precision = display_precision
            size_precision = trade_units_precision

            # Price increment (pip location determines this)
            pip_location = data.get("pipLocation", -4)
            price_increment = Decimal(10) ** pip_location

            # Size increment
            size_increment = Decimal(10) ** (-size_precision) if size_precision > 0 else Decimal(1)

            # Create currency objects
            base_currency = Currency.from_str(base_currency_code)
            quote_currency = Currency.from_str(quote_currency_code)

            # Build the CurrencyPair instrument
            instrument = CurrencyPair(
                instrument_id=instrument_id,
                raw_symbol=symbol,
                base_currency=base_currency,
                quote_currency=quote_currency,
                price_precision=price_precision,
                size_precision=size_precision,
                price_increment=Price(price_increment, precision=price_precision),
                size_increment=Quantity(size_increment, precision=size_precision),
                lot_size=Quantity(Decimal(1), precision=0),
                max_quantity=Quantity(Decimal(maximum_trade_units), precision=0),
                min_quantity=Quantity(Decimal(minimum_trade_size), precision=size_precision),
                max_price=None,
                min_price=None,
                margin_init=Decimal(0),
                margin_maint=Decimal(0),
                maker_fee=Decimal(0),
                taker_fee=Decimal(0),
                ts_event=0,
                ts_init=0,
            )

            return instrument

        except Exception as e:
            self._log.error(f"Failed to parse instrument {data.get('name', 'unknown')}: {e}")
            return None

    async def load_all_async(self, filters: dict | None = None) -> None:
        """
        Load all OANDA instruments into the provider.

        Parameters
        ----------
        filters : dict, optional
            Not currently used for OANDA.

        """
        filters_str = "..." if not filters else f" with filters {filters}..."
        self._log.info(f"Loading all instruments{filters_str}")

        # Get instruments from OANDA API
        instruments_json = await self._client.get_instruments()
        instruments_data = json.loads(instruments_json)

        self._raw_instruments = instruments_data

        # Parse and add each instrument
        for raw_instrument in instruments_data:
            instrument = self._parse_instrument(raw_instrument)
            if instrument is not None:
                self.add(instrument=instrument)

        self._log.info(f"Loaded {self.count} instruments")

    async def load_ids_async(
        self,
        instrument_ids: list[InstrumentId],
        filters: dict | None = None,
    ) -> None:
        """
        Load specific instruments by ID.

        Parameters
        ----------
        instrument_ids : list[InstrumentId]
            The instrument IDs to load.
        filters : dict, optional
            Not currently used for OANDA.

        """
        if not instrument_ids:
            self._log.warning("No instrument IDs given for loading")
            return

        # Validate all instrument IDs
        for instrument_id in instrument_ids:
            PyCondition.equal(instrument_id.venue, OANDA_VENUE, "instrument_id.venue", "OANDA")

        # Get all instruments (OANDA doesn't support filtering by specific symbols in API)
        if not self._raw_instruments:
            instruments_json = await self._client.get_instruments()
            instruments_data = json.loads(instruments_json)
            self._raw_instruments = instruments_data

        # Create a set of requested symbol names for efficient lookup
        requested_symbols = {instrument_id.symbol.value for instrument_id in instrument_ids}

        # Parse and add only the requested instruments
        for raw_instrument in self._raw_instruments:
            name = raw_instrument.get("name", "")
            if name in requested_symbols:
                instrument = self._parse_instrument(raw_instrument)
                if instrument is not None:
                    self.add(instrument=instrument)

        self._log.info(f"Loaded {len(instrument_ids)} instruments")

    async def load_async(
        self,
        instrument_id: InstrumentId,
        filters: dict | None = None,
    ) -> None:
        """
        Load a specific instrument by ID.

        Parameters
        ----------
        instrument_id : InstrumentId
            The instrument ID to load.
        filters : dict, optional
            Not currently used for OANDA.

        """
        PyCondition.not_none(instrument_id, "instrument_id")
        await self.load_ids_async([instrument_id], filters)
