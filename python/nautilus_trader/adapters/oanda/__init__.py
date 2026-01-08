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
OANDA forex broker adapter for NautilusTrader.

This adapter provides integration with OANDA, a major forex and CFD broker,
supporting both demo (practice) and live trading environments.

Features:
- REST API v3 client for account and trading operations
- Streaming API client for real-time price feeds
- Transaction streaming for order/trade updates
- Support for forex pairs and CFD instruments
- Full order lifecycle management
- Position and account state tracking

Example
-------
>>> from nautilus_trader.adapters.oanda import OANDAHttpClient
>>> from nautilus_trader.adapters.oanda import OANDAEnvironment
>>>
>>> # Create HTTP client
>>> client = OANDAHttpClient(
...     api_key="your-api-key",
...     account_id="your-account-id",
...     environment=OANDAEnvironment.Practice,
... )
>>>
>>> # Get instruments
>>> instruments = await client.get_instruments()
"""

from nautilus_trader._libnautilus.oanda import *  # noqa: F403 (undefined-local-with-import-star)
