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
"""Type stubs for the OANDA adapter."""

from enum import Enum

class OANDAEnvironment(Enum):
    """OANDA trading environment."""

    Practice = ...
    Live = ...

class OANDAHttpClient:
    """OANDA HTTP REST API client."""

    def __init__(
        self,
        api_key: str,
        account_id: str,
        environment: OANDAEnvironment = OANDAEnvironment.Practice,
        timeout_secs: int | None = None,
        max_retries: int | None = None,
    ) -> None: ...
    @property
    def account_id(self) -> str: ...
    async def get_account_summary(self) -> str: ...
    async def get_instruments(self) -> str: ...
    async def get_candles(
        self,
        instrument: str,
        granularity: str,
        count: int | None = None,
        from_time: str | None = None,
        to_time: str | None = None,
        price: str | None = None,
    ) -> str: ...
    async def get_pricing(self, instruments: list[str]) -> str: ...
    async def create_market_order(
        self,
        instrument: str,
        units: str,
        time_in_force: str | None = None,
        take_profit_price: str | None = None,
        stop_loss_price: str | None = None,
    ) -> str: ...
    async def create_limit_order(
        self,
        instrument: str,
        units: str,
        price: str,
        time_in_force: str | None = None,
        take_profit_price: str | None = None,
        stop_loss_price: str | None = None,
    ) -> str: ...
    async def cancel_order(self, order_id: str) -> str: ...
    async def get_orders(self, state: str | None = None) -> str: ...
    async def get_order(self, order_id: str) -> str: ...
    async def get_trades(self, state: str | None = None) -> str: ...
    async def get_trade(self, trade_id: str) -> str: ...

class OANDAStreamClient:
    """OANDA price streaming client."""

    def __init__(
        self,
        api_key: str,
        account_id: str,
        environment: OANDAEnvironment = OANDAEnvironment.Practice,
    ) -> None: ...
    async def connect(self, instruments: list[str]) -> None: ...
    async def disconnect(self) -> None: ...
    async def next(self) -> str | None: ...
    def is_connected(self) -> bool: ...

class OANDATransactionStreamClient:
    """OANDA transaction streaming client."""

    def __init__(
        self,
        api_key: str,
        account_id: str,
        environment: OANDAEnvironment = OANDAEnvironment.Practice,
    ) -> None: ...
    async def connect(self) -> None: ...
    async def disconnect(self) -> None: ...
    async def next(self) -> str | None: ...
    def is_connected(self) -> bool: ...

class OANDADataClientConfig:
    """Configuration for OANDA data client."""

    def __init__(
        self,
        account_id: str,
        is_live: bool = False,
    ) -> None: ...

class OANDAExecClientConfig:
    """Configuration for OANDA execution client."""

    def __init__(
        self,
        account_id: str,
        is_live: bool = False,
    ) -> None: ...

def oanda_environment_from_str(value: str) -> OANDAEnvironment: ...
def oanda_order_type_to_str(order_type: str) -> str: ...
def oanda_time_in_force_to_str(tif: str) -> str: ...
