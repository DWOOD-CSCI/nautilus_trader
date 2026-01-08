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

"""OANDA adapter configuration classes."""

from nautilus_trader.config import (
    InstrumentProviderConfig,
    LiveDataClientConfig,
    LiveExecClientConfig,
)


class OANDAInstrumentProviderConfig(InstrumentProviderConfig, frozen=True):
    """
    Configuration for the OANDA instrument provider.

    Parameters
    ----------
    api_key : str, optional
        The OANDA API access token.
        If not provided, uses OANDA_API_KEY environment variable.
    account_id : str, optional
        The OANDA account ID.
        If not provided, uses OANDA_ACCOUNT_ID environment variable.
    is_live : bool, default False
        If True, use the live environment. Otherwise use practice.
    load_all_instruments : bool, default True
        If True, load all available instruments on initialization.
    instruments : list[str], optional
        Specific instruments to load (e.g., ["EUR_USD", "GBP_USD"]).
    """

    api_key: str | None = None
    account_id: str | None = None
    is_live: bool = False
    load_all_instruments: bool = True
    instruments: list[str] | None = None


class OANDADataClientConfig(LiveDataClientConfig, frozen=True):
    """
    Configuration for the OANDA live data client.

    Parameters
    ----------
    api_key : str, optional
        The OANDA API access token.
        If not provided, uses OANDA_API_KEY environment variable.
    account_id : str, optional
        The OANDA account ID.
        If not provided, uses OANDA_ACCOUNT_ID environment variable.
    is_live : bool, default False
        If True, use the live environment. Otherwise use practice.
    instrument_provider : OANDAInstrumentProviderConfig, optional
        Configuration for the instrument provider.
    """

    api_key: str | None = None
    account_id: str | None = None
    is_live: bool = False
    instrument_provider: OANDAInstrumentProviderConfig | None = None


class OANDAExecClientConfig(LiveExecClientConfig, frozen=True):
    """
    Configuration for the OANDA live execution client.

    Parameters
    ----------
    api_key : str, optional
        The OANDA API access token.
        If not provided, uses OANDA_API_KEY environment variable.
    account_id : str, optional
        The OANDA account ID.
        If not provided, uses OANDA_ACCOUNT_ID environment variable.
    is_live : bool, default False
        If True, use the live environment. Otherwise use practice.
    instrument_provider : OANDAInstrumentProviderConfig, optional
        Configuration for the instrument provider.
    """

    api_key: str | None = None
    account_id: str | None = None
    is_live: bool = False
    instrument_provider: OANDAInstrumentProviderConfig | None = None
