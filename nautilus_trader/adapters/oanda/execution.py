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

"""OANDA live execution client."""

from __future__ import annotations

import asyncio
import json
from decimal import Decimal
from typing import TYPE_CHECKING, Any

from nautilus_trader.adapters.oanda.config import OANDAExecClientConfig
from nautilus_trader.adapters.oanda.constants import OANDA_VENUE
from nautilus_trader.adapters.oanda.providers import OANDAInstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock, MessageBus
from nautilus_trader.common.enums import LogColor
from nautilus_trader.execution.messages import (
    CancelAllOrders,
    CancelOrder,
    ModifyOrder,
    SubmitOrder,
)
from nautilus_trader.execution.reports import OrderStatusReport, PositionStatusReport
from nautilus_trader.live.execution_client import LiveExecutionClient
from nautilus_trader.model.enums import AccountType, OmsType, OrderSide, OrderType
from nautilus_trader.model.events import (
    AccountState,
    OrderAccepted,
    OrderCanceled,
    OrderFilled,
    OrderRejected,
)
from nautilus_trader.model.identifiers import (
    AccountId,
    ClientId,
    ClientOrderId,
    InstrumentId,
    TradeId,
    VenueOrderId,
)
from nautilus_trader.model.objects import Currency, Money, Price, Quantity

if TYPE_CHECKING:
    pass


class OANDAExecutionClient(LiveExecutionClient):
    """
    Provides an execution client for the OANDA FX broker.

    This client interfaces with OANDA's REST API for order management and
    execution. It also connects to the transaction stream for real-time
    order and trade updates.

    Parameters
    ----------
    loop : asyncio.AbstractEventLoop
        The event loop for the client.
    client : OANDAHttpClient
        The OANDA HTTP client (from Rust adapter).
    transaction_client : OANDATransactionStreamClient
        The OANDA transaction stream client (from Rust adapter).
    msgbus : MessageBus
        The message bus for the client.
    cache : Cache
        The cache for the client.
    clock : LiveClock
        The clock for the client.
    instrument_provider : OANDAInstrumentProvider
        The instrument provider.
    config : OANDAExecClientConfig
        The configuration for the client.
    name : str, optional
        The custom client ID.

    """

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        client: Any,  # OANDAHttpClient from nautilus_pyo3.oanda
        transaction_client: Any,  # OANDATransactionStreamClient
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        instrument_provider: OANDAInstrumentProvider,
        config: OANDAExecClientConfig,
        name: str | None = None,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId(name or OANDA_VENUE.value),
            venue=OANDA_VENUE,
            oms_type=OmsType.NETTING,
            instrument_provider=instrument_provider,
            account_type=AccountType.MARGIN,  # OANDA is margin-based
            base_currency=None,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
        )

        self._instrument_provider: OANDAInstrumentProvider = instrument_provider

        # Configuration
        self._config = config
        self._account_id_str = config.account_id

        # Set account ID
        account_id = AccountId(f"{name or OANDA_VENUE.value}-{config.account_id or 'MAIN'}")
        self._set_account_id(account_id)

        # HTTP API client
        self._http_client = client

        # Transaction stream client
        self._transaction_client = transaction_client
        self._transaction_task: asyncio.Task | None = None

        # Order tracking
        self._pending_orders: dict[ClientOrderId, SubmitOrder] = {}
        self._venue_order_ids: dict[ClientOrderId, VenueOrderId] = {}

        self._log.info("OANDA execution client initialized", LogColor.BLUE)

    @property
    def instrument_provider(self) -> OANDAInstrumentProvider:
        """Return the instrument provider."""
        return self._instrument_provider

    async def _connect(self) -> None:
        """Connect to OANDA and initialize the execution client."""
        # Initialize instrument provider
        await self._instrument_provider.initialize()

        # Get initial account state
        await self._update_account_state()

        # Start transaction stream
        await self._start_transaction_stream()

        self._log.info("Connected to OANDA execution client", LogColor.BLUE)

    async def _disconnect(self) -> None:
        """Disconnect from OANDA."""
        # Stop transaction stream
        await self._stop_transaction_stream()

        self._log.info("Disconnected from OANDA execution client", LogColor.BLUE)

    async def _update_account_state(self) -> None:
        """Fetch and update account state from OANDA."""
        try:
            summary_json = await self._http_client.get_account_summary()
            summary = json.loads(summary_json)

            # Parse account balances
            balance = Decimal(summary.get("balance", "0"))
            unrealized_pnl = Decimal(summary.get("unrealizedPL", "0"))
            margin_used = Decimal(summary.get("marginUsed", "0"))
            margin_available = Decimal(summary.get("marginAvailable", "0"))

            # Get account currency (usually USD)
            currency_str = summary.get("currency", "USD")
            currency = Currency.from_str(currency_str)

            # Create account state event
            account_state = AccountState(
                account_id=self.account_id,
                account_type=AccountType.MARGIN,
                base_currency=currency,
                reported=True,
                balances=[
                    Money(balance, currency),
                ],
                margins=[
                    Money(margin_used, currency),
                ],
                info={
                    "unrealized_pnl": str(unrealized_pnl),
                    "margin_available": str(margin_available),
                },
                event_id=self._uuid_factory.generate(),
                ts_event=self._clock.timestamp_ns(),
                ts_init=self._clock.timestamp_ns(),
            )

            self._msgbus.publish(
                topic=f"data.accounts.{self.account_id}",
                msg=account_state,
            )

        except Exception as e:
            self._log.error(f"Error updating account state: {e}")

    async def _start_transaction_stream(self) -> None:
        """Start the transaction stream."""
        if self._transaction_task and not self._transaction_task.done():
            return

        self._log.info("Starting transaction stream", LogColor.BLUE)

        # Connect the transaction stream
        await self._transaction_client.connect()

        # Start processing transactions
        self._transaction_task = self.create_task(self._process_transactions())

    async def _stop_transaction_stream(self) -> None:
        """Stop the transaction stream."""
        if self._transaction_task:
            self._transaction_task.cancel()
            try:
                await self._transaction_task
            except asyncio.CancelledError:
                self._log.debug("Transaction task cancelled during stop")
            self._transaction_task = None

        await self._transaction_client.disconnect()
        self._log.info("Stopped transaction stream", LogColor.YELLOW)

    async def _process_transactions(self) -> None:
        """Process incoming transaction stream messages."""
        try:
            while True:
                message = await self._transaction_client.next()
                if message is None:
                    self._log.warning("Transaction stream disconnected")
                    break

                self._handle_transaction_message(message)

        except asyncio.CancelledError:
            self._log.debug("Transaction processing cancelled")
            raise
        except Exception as e:
            self._log.error(f"Error processing transactions: {e}")

    def _handle_transaction_message(self, message: str) -> None:
        """Handle a transaction message from OANDA."""
        try:
            data = json.loads(message)
            tx_type = data.get("type")

            if tx_type == "ORDER_FILL":
                self._handle_order_fill(data)
            elif tx_type == "ORDER_CANCEL":
                self._handle_order_cancel(data)
            elif tx_type in ("MARKET_ORDER", "LIMIT_ORDER", "STOP_ORDER"):
                self._handle_order_created(data)
            elif tx_type == "HEARTBEAT":
                # Heartbeat - ignore
                pass
            else:
                self._log.debug(f"Unhandled transaction type: {tx_type}")

        except Exception as e:
            self._log.error(f"Error handling transaction: {e}")

    def _handle_order_fill(self, data: dict) -> None:
        """Handle an ORDER_FILL transaction."""
        try:
            # Extract trade details
            trade_id = data.get("tradeOpened", {}).get("tradeID") or data.get("tradesClosed", [{}])[
                0
            ].get("tradeID")
            order_id = data.get("orderID")
            instrument_name = data.get("instrument")
            units = Decimal(data.get("units", "0"))
            price = Decimal(data.get("price", "0"))

            if not instrument_name:
                return

            instrument_id = InstrumentId.from_str(f"{instrument_name}.{OANDA_VENUE.value}")
            instrument = self._cache.instrument(instrument_id)

            if instrument is None:
                self._log.warning(f"Instrument not found for fill: {instrument_id}")
                return

            # Determine order side
            side = OrderSide.BUY if units > 0 else OrderSide.SELL

            # Create fill event
            fill = OrderFilled(
                trader_id=self._trader_id,
                strategy_id=self._strategy_id,
                instrument_id=instrument_id,
                client_order_id=ClientOrderId(str(order_id)),
                venue_order_id=VenueOrderId(str(order_id)),
                account_id=self.account_id,
                trade_id=TradeId(str(trade_id)),
                order_side=side,
                order_type=OrderType.MARKET,
                last_qty=Quantity(abs(units), precision=0),
                last_px=Price(price, precision=instrument.price_precision),
                currency=instrument.quote_currency,
                commission=Money(Decimal("0"), instrument.quote_currency),
                liquidity_side=None,
                event_id=self._uuid_factory.generate(),
                ts_event=self._clock.timestamp_ns(),
                ts_init=self._clock.timestamp_ns(),
            )

            self._msgbus.publish(
                topic=f"events.order.{instrument_id.venue}",
                msg=fill,
            )

        except Exception as e:
            self._log.error(f"Error handling order fill: {e}")

    def _handle_order_cancel(self, data: dict) -> None:
        """Handle an ORDER_CANCEL transaction."""
        try:
            order_id = data.get("orderID")

            if not order_id:
                return

            cancel = OrderCanceled(
                trader_id=self._trader_id,
                strategy_id=self._strategy_id,
                instrument_id=None,  # Not available in cancel message
                client_order_id=ClientOrderId(str(order_id)),
                venue_order_id=VenueOrderId(str(order_id)),
                account_id=self.account_id,
                event_id=self._uuid_factory.generate(),
                ts_event=self._clock.timestamp_ns(),
                ts_init=self._clock.timestamp_ns(),
            )

            self._msgbus.publish(
                topic=f"events.order.{OANDA_VENUE}",
                msg=cancel,
            )

        except Exception as e:
            self._log.error(f"Error handling order cancel: {e}")

    def _handle_order_created(self, data: dict) -> None:
        """Handle an order created transaction."""
        try:
            order_id = data.get("id")
            instrument_name = data.get("instrument")

            if not instrument_name or not order_id:
                return

            instrument_id = InstrumentId.from_str(f"{instrument_name}.{OANDA_VENUE.value}")

            accepted = OrderAccepted(
                trader_id=self._trader_id,
                strategy_id=self._strategy_id,
                instrument_id=instrument_id,
                client_order_id=ClientOrderId(str(order_id)),
                venue_order_id=VenueOrderId(str(order_id)),
                account_id=self.account_id,
                event_id=self._uuid_factory.generate(),
                ts_event=self._clock.timestamp_ns(),
                ts_init=self._clock.timestamp_ns(),
                reconciliation=False,
            )

            self._msgbus.publish(
                topic=f"events.order.{instrument_id.venue}",
                msg=accepted,
            )

        except Exception as e:
            self._log.error(f"Error handling order created: {e}")

    async def _submit_order(self, command: SubmitOrder) -> None:
        """Submit an order to OANDA."""
        order = command.order
        instrument_id = order.instrument_id
        oanda_symbol = instrument_id.symbol.value

        try:
            # Convert order side to units
            units = int(order.quantity)
            if order.side == OrderSide.SELL:
                units = -units

            # Submit via HTTP client
            if order.order_type == OrderType.MARKET:
                result_json = await self._http_client.create_market_order(
                    oanda_symbol,
                    str(units),
                )
            elif order.order_type == OrderType.LIMIT:
                result_json = await self._http_client.create_limit_order(
                    oanda_symbol,
                    str(units),
                    str(order.price),
                )
            else:
                self._log.error(f"Unsupported order type: {order.order_type}")
                return

            result = json.loads(result_json)

            # Check for errors
            if "errorMessage" in result:
                self._generate_order_rejected(order, result.get("errorMessage", "Unknown error"))
                return

            # Order was accepted
            self._log.info(f"Order submitted: {order.client_order_id}", LogColor.GREEN)

        except Exception as e:
            self._log.error(f"Error submitting order: {e}")
            self._generate_order_rejected(order, str(e))

    def _generate_order_rejected(self, order, reason: str) -> None:
        """Generate an OrderRejected event."""
        rejected = OrderRejected(
            trader_id=self._trader_id,
            strategy_id=order.strategy_id,
            instrument_id=order.instrument_id,
            client_order_id=order.client_order_id,
            account_id=self.account_id,
            reason=reason,
            event_id=self._uuid_factory.generate(),
            ts_event=self._clock.timestamp_ns(),
            ts_init=self._clock.timestamp_ns(),
        )

        self._msgbus.publish(
            topic=f"events.order.{order.instrument_id.venue}",
            msg=rejected,
        )

    async def _cancel_order(self, command: CancelOrder) -> None:
        """Cancel an order."""
        venue_order_id = command.venue_order_id

        try:
            result_json = await self._http_client.cancel_order(str(venue_order_id))
            result = json.loads(result_json)

            if "errorMessage" in result:
                self._log.error(f"Order cancel failed: {result.get('errorMessage')}")
                return

            self._log.info(f"Order cancelled: {venue_order_id}", LogColor.YELLOW)

        except Exception as e:
            self._log.error(f"Error cancelling order: {e}")

    async def _cancel_all_orders(self, command: CancelAllOrders) -> None:
        """Cancel all orders for an instrument."""
        self._log.warning("Cancel all orders not yet implemented for OANDA")

    async def _modify_order(self, command: ModifyOrder) -> None:
        """Modify an existing order."""
        self._log.warning("Modify order not yet implemented for OANDA")

    async def generate_order_status_report(
        self,
        instrument_id: InstrumentId,
        client_order_id: ClientOrderId | None = None,
        venue_order_id: VenueOrderId | None = None,
    ) -> OrderStatusReport | None:
        """Generate an order status report."""
        self._log.warning("Order status reports not yet implemented for OANDA")
        return None

    async def generate_order_status_reports(
        self,
        instrument_id: InstrumentId | None = None,
        start: int | None = None,
        end: int | None = None,
        open_only: bool = False,
    ) -> list[OrderStatusReport]:
        """Generate order status reports."""
        self._log.warning("Order status reports not yet implemented for OANDA")
        return []

    async def generate_fill_reports(
        self,
        instrument_id: InstrumentId | None = None,
        venue_order_id: VenueOrderId | None = None,
        start: int | None = None,
        end: int | None = None,
    ) -> list:
        """Generate fill reports."""
        self._log.warning("Fill reports not yet implemented for OANDA")
        return []

    async def generate_position_status_reports(
        self,
        instrument_id: InstrumentId | None = None,
        start: int | None = None,
        end: int | None = None,
    ) -> list[PositionStatusReport]:
        """Generate position status reports."""
        self._log.warning("Position reports not yet implemented for OANDA")
        return []
