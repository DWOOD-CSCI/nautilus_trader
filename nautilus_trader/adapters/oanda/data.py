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

"""OANDA live market data client."""

from __future__ import annotations

import asyncio
import json
from decimal import Decimal
from typing import TYPE_CHECKING, Any

from nautilus_trader.adapters.oanda.config import OANDADataClientConfig
from nautilus_trader.adapters.oanda.constants import OANDA_VENUE
from nautilus_trader.adapters.oanda.providers import OANDAInstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock, MessageBus
from nautilus_trader.common.enums import LogColor
from nautilus_trader.data.messages import SubscribeQuoteTicks, UnsubscribeQuoteTicks
from nautilus_trader.live.data_client import LiveMarketDataClient
from nautilus_trader.model.data import QuoteTick
from nautilus_trader.model.identifiers import ClientId, InstrumentId
from nautilus_trader.model.objects import Price, Quantity

if TYPE_CHECKING:
    pass


class OANDADataClient(LiveMarketDataClient):
    """
    Provides a data client for the OANDA FX broker.

    Parameters
    ----------
    loop : asyncio.AbstractEventLoop
        The event loop for the client.
    client : OANDAHttpClient
        The OANDA HTTP client (from Rust adapter).
    stream_client : OANDAStreamClient
        The OANDA streaming client (from Rust adapter).
    msgbus : MessageBus
        The message bus for the client.
    cache : Cache
        The cache for the client.
    clock : LiveClock
        The clock for the client.
    instrument_provider : OANDAInstrumentProvider
        The instrument provider.
    config : OANDADataClientConfig
        The configuration for the client.
    name : str, optional
        The custom client ID.

    """

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        client: Any,  # OANDAHttpClient from nautilus_pyo3.oanda
        stream_client: Any,  # OANDAStreamClient from nautilus_pyo3.oanda
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        instrument_provider: OANDAInstrumentProvider,
        config: OANDADataClientConfig,
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or OANDA_VENUE.value),
            venue=OANDA_VENUE,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=instrument_provider,
        )

        self._instrument_provider: OANDAInstrumentProvider = instrument_provider

        # Configuration
        self._config = config

        # HTTP API client
        self._http_client = client

        # Streaming client
        self._stream_client = stream_client
        self._stream_task: asyncio.Task | None = None

        # Track subscribed instruments
        self._subscribed_instruments: set[str] = set()

        self._log.info("OANDA data client initialized", LogColor.BLUE)

    @property
    def instrument_provider(self) -> OANDAInstrumentProvider:
        """Return the instrument provider."""
        return self._instrument_provider

    async def _connect(self) -> None:
        """Connect to OANDA and initialize the data client."""
        # Initialize instrument provider
        await self._instrument_provider.initialize()
        self._send_all_instruments_to_data_engine()

        self._log.info("Connected to OANDA data client", LogColor.BLUE)

    async def _disconnect(self) -> None:
        """Disconnect from OANDA."""
        # Stop streaming task
        if self._stream_task and not self._stream_task.done():
            self._stream_task.cancel()
            try:
                await self._stream_task
            except asyncio.CancelledError:
                self._log.debug("Stream task cancelled during disconnect")
            self._stream_task = None

        # Disconnect stream client
        if self._stream_client:
            await self._stream_client.disconnect()

        self._log.info("Disconnected from OANDA data client", LogColor.BLUE)

    def _send_all_instruments_to_data_engine(self) -> None:
        """Send all loaded instruments to the data engine."""
        for instrument in self._instrument_provider.get_all().values():
            self._handle_data(instrument)

    async def _subscribe_quote_ticks(self, command: SubscribeQuoteTicks) -> None:
        """Subscribe to quote tick updates for an instrument."""
        instrument_id = command.instrument_id
        oanda_symbol = instrument_id.symbol.value  # e.g., "EUR_USD"

        if oanda_symbol in self._subscribed_instruments:
            self._log.warning(f"Already subscribed to {oanda_symbol}")
            return

        self._subscribed_instruments.add(oanda_symbol)

        # If this is the first subscription, start the streaming task
        if len(self._subscribed_instruments) == 1:
            await self._start_streaming()
        else:
            # Update the stream with new instruments
            await self._restart_streaming()

        self._log.info(f"Subscribed to quote ticks for {oanda_symbol}", LogColor.GREEN)

    async def _unsubscribe_quote_ticks(self, command: UnsubscribeQuoteTicks) -> None:
        """Unsubscribe from quote tick updates for an instrument."""
        instrument_id = command.instrument_id
        oanda_symbol = instrument_id.symbol.value

        if oanda_symbol not in self._subscribed_instruments:
            self._log.warning(f"Not subscribed to {oanda_symbol}")
            return

        self._subscribed_instruments.discard(oanda_symbol)

        if not self._subscribed_instruments:
            # No more subscriptions, stop streaming
            await self._stop_streaming()
        else:
            # Restart stream with remaining instruments
            await self._restart_streaming()

        self._log.info(f"Unsubscribed from quote ticks for {oanda_symbol}", LogColor.YELLOW)

    async def _start_streaming(self) -> None:
        """Start the price streaming task."""
        if self._stream_task and not self._stream_task.done():
            return

        instruments = list(self._subscribed_instruments)
        self._log.info(f"Starting price stream for: {instruments}", LogColor.BLUE)

        # Connect the stream client with instruments
        await self._stream_client.connect(instruments)

        # Start processing stream messages
        self._stream_task = self.create_task(self._process_stream())

    async def _stop_streaming(self) -> None:
        """Stop the price streaming task."""
        if self._stream_task:
            self._stream_task.cancel()
            try:
                await self._stream_task
            except asyncio.CancelledError:
                self._log.debug("Stream task cancelled during stop")
            self._stream_task = None

        await self._stream_client.disconnect()
        self._log.info("Stopped price stream", LogColor.YELLOW)

    async def _restart_streaming(self) -> None:
        """Restart streaming with updated instrument list."""
        await self._stop_streaming()
        await self._start_streaming()

    async def _process_stream(self) -> None:
        """Process incoming price stream messages."""
        try:
            while True:
                message = await self._stream_client.next()
                if message is None:
                    # Stream disconnected
                    self._log.warning("Price stream disconnected")
                    break

                self._handle_stream_message(message)

        except asyncio.CancelledError:
            self._log.debug("Stream processing cancelled")
            raise
        except Exception as e:
            self._log.error(f"Error processing stream: {e}")

    def _handle_stream_message(self, message: str) -> None:
        """Handle a streaming price message from OANDA."""
        try:
            data = json.loads(message)
            msg_type = data.get("type")

            if msg_type == "PRICE":
                self._handle_price_message(data)
            elif msg_type == "HEARTBEAT":
                # Heartbeat - ignore silently
                pass
            else:
                self._log.debug(f"Unknown message type: {msg_type}")

        except Exception as e:
            self._log.error(f"Error handling stream message: {e}")

    def _handle_price_message(self, data: dict) -> None:
        """Convert OANDA price message to QuoteTick and publish."""
        try:
            instrument_name = data.get("instrument")
            if not instrument_name:
                return

            # Get instrument for precision info
            instrument_id = InstrumentId.from_str(f"{instrument_name}.{OANDA_VENUE.value}")
            instrument = self._cache.instrument(instrument_id)

            if instrument is None:
                self._log.warning(f"Instrument not found in cache: {instrument_id}")
                return

            # Parse bid/ask prices
            bids = data.get("bids", [])
            asks = data.get("asks", [])

            if not bids or not asks:
                return

            # Use top of book
            bid_price = Decimal(bids[0].get("price", "0"))
            ask_price = Decimal(asks[0].get("price", "0"))
            bid_size = Decimal(bids[0].get("liquidity", "1"))
            ask_size = Decimal(asks[0].get("liquidity", "1"))

            # Create timestamp
            ts_event = self._clock.timestamp_ns()
            ts_init = ts_event

            # Create QuoteTick
            quote_tick = QuoteTick(
                instrument_id=instrument_id,
                bid_price=Price(bid_price, precision=instrument.price_precision),
                ask_price=Price(ask_price, precision=instrument.price_precision),
                bid_size=Quantity(bid_size, precision=0),
                ask_size=Quantity(ask_size, precision=0),
                ts_event=ts_event,
                ts_init=ts_init,
            )

            self._handle_data(quote_tick)

        except Exception as e:
            self._log.error(f"Error parsing price message: {e}")

    # Required subscription methods - not all supported by OANDA

    async def _subscribe_order_book_deltas(self, command) -> None:
        self._log.warning("Order book subscriptions not supported by OANDA data client")

    async def _subscribe_order_book_snapshots(self, command) -> None:
        self._log.warning("Order book subscriptions not supported by OANDA data client")

    async def _subscribe_trade_ticks(self, command) -> None:
        self._log.warning("Trade tick subscriptions not supported by OANDA data client")

    async def _subscribe_bars(self, command) -> None:
        self._log.warning("Bar subscriptions not yet implemented for OANDA data client")

    async def _unsubscribe_order_book_deltas(self, command) -> None:
        # Order book not supported by OANDA - no-op
        pass

    async def _unsubscribe_order_book_snapshots(self, command) -> None:
        # Order book not supported by OANDA - no-op
        pass

    async def _unsubscribe_trade_ticks(self, command) -> None:
        # Trade ticks not supported by OANDA - no-op
        pass

    async def _unsubscribe_bars(self, command) -> None:
        # Bar subscriptions not yet implemented - no-op
        pass

    async def _request_quote_ticks(self, command) -> None:
        self._log.warning("Historical quote tick requests not yet implemented for OANDA")

    async def _request_trade_ticks(self, command) -> None:
        self._log.warning("Historical trade tick requests not yet implemented for OANDA")

    async def _request_bars(self, command) -> None:
        self._log.warning("Historical bar requests not yet implemented for OANDA")
