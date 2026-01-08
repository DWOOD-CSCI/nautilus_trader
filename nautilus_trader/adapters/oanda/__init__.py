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
OANDA adapter for NautilusTrader.

This adapter provides connectivity to OANDA for forex trading through the v20 REST API.

Features:
- Real-time price streaming
- Order execution (market, limit, stop, take profit, stop loss)
- Position management
- Account information
- Historical candle data

Example
-------
>>> from nautilus_trader.adapters.oanda import OANDALiveDataClientFactory
>>> from nautilus_trader.adapters.oanda import OANDALiveExecClientFactory
>>> from nautilus_trader.adapters.oanda.config import OANDADataClientConfig
>>> from nautilus_trader.adapters.oanda.config import OANDAExecClientConfig
>>>
>>> # Configure data client
>>> data_config = OANDADataClientConfig(
...     api_key="your-api-key",
...     account_id="your-account-id",
... )
>>>
>>> # Configure execution client
>>> exec_config = OANDAExecClientConfig(
...     api_key="your-api-key",
...     account_id="your-account-id",
... )
"""

from nautilus_trader.adapters.oanda.config import (
    OANDADataClientConfig,
    OANDAExecClientConfig,
    OANDAInstrumentProviderConfig,
)
from nautilus_trader.adapters.oanda.constants import OANDA_VENUE
from nautilus_trader.adapters.oanda.data import OANDADataClient
from nautilus_trader.adapters.oanda.execution import OANDAExecutionClient
from nautilus_trader.adapters.oanda.factories import (
    OANDALiveDataClientFactory,
    OANDALiveExecClientFactory,
)
from nautilus_trader.adapters.oanda.providers import OANDAInstrumentProvider

__all__ = [
    "OANDA_VENUE",
    "OANDADataClient",
    "OANDADataClientConfig",
    "OANDAExecClientConfig",
    "OANDAExecutionClient",
    "OANDAInstrumentProvider",
    "OANDAInstrumentProviderConfig",
    "OANDALiveDataClientFactory",
    "OANDALiveExecClientFactory",
]
