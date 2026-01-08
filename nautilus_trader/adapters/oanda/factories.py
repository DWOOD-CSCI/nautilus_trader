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

"""OANDA adapter factory functions."""

from __future__ import annotations

import asyncio
import os
from functools import lru_cache
from typing import Any

from nautilus_trader.adapters.oanda.config import (
    OANDADataClientConfig,
    OANDAExecClientConfig,
    OANDAInstrumentProviderConfig,
)
from nautilus_trader.adapters.oanda.data import OANDADataClient
from nautilus_trader.adapters.oanda.execution import OANDAExecutionClient
from nautilus_trader.adapters.oanda.providers import OANDAInstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock, MessageBus
from nautilus_trader.live.factories import LiveDataClientFactory, LiveExecClientFactory


def _get_api_key(config_api_key: str | None) -> str:
    """Get API key from config or environment variable."""
    if config_api_key:
        return config_api_key
    api_key = os.environ.get("OANDA_API_KEY")
    if not api_key:
        raise ValueError(
            "OANDA API key not provided in config and OANDA_API_KEY environment variable not set"
        )
    return api_key


def _get_account_id(config_account_id: str | None) -> str:
    """Get account ID from config or environment variable."""
    if config_account_id:
        return config_account_id
    account_id = os.environ.get("OANDA_ACCOUNT_ID")
    if not account_id:
        raise ValueError(
            "OANDA account ID not provided in config and OANDA_ACCOUNT_ID environment variable not set"
        )
    return account_id


def get_oanda_http_client(
    api_key: str,
    account_id: str,
    is_live: bool = False,
    timeout_secs: int | None = None,
    max_retries: int | None = None,
) -> Any:
    """
    Create an OANDA HTTP client.

    Parameters
    ----------
    api_key : str
        The OANDA API access token.
    account_id : str
        The OANDA account ID.
    is_live : bool, default False
        If True, use the live environment. Otherwise use practice.
    timeout_secs : int, optional
        The HTTP request timeout in seconds.
    max_retries : int, optional
        The maximum number of retry attempts.

    Returns
    -------
    OANDAHttpClient
        The OANDA HTTP client (from Rust adapter).

    Raises
    ------
    ImportError
        If the nautilus_pyo3.oanda module is not available.

    """
    try:
        # Import the OANDA module from pyo3
        # This will be available once the Rust adapter is built and installed
        from nautilus_trader.core.nautilus_pyo3 import oanda

        environment = oanda.OANDAEnvironment.Live if is_live else oanda.OANDAEnvironment.Practice

        return oanda.OANDAHttpClient(
            api_key=api_key,
            account_id=account_id,
            environment=environment,
            timeout_secs=timeout_secs,
            max_retries=max_retries,
        )
    except ImportError as e:
        raise ImportError(
            "OANDA adapter requires the nautilus_pyo3.oanda module. "
            "Ensure the Rust OANDA adapter is built and installed."
        ) from e


def get_oanda_stream_client(
    api_key: str,
    account_id: str,
    is_live: bool = False,
) -> Any:
    """
    Create an OANDA price streaming client.

    Parameters
    ----------
    api_key : str
        The OANDA API access token.
    account_id : str
        The OANDA account ID.
    is_live : bool, default False
        If True, use the live environment. Otherwise use practice.

    Returns
    -------
    OANDAStreamClient
        The OANDA streaming client (from Rust adapter).

    """
    try:
        from nautilus_trader.core.nautilus_pyo3 import oanda

        environment = oanda.OANDAEnvironment.Live if is_live else oanda.OANDAEnvironment.Practice

        return oanda.OANDAStreamClient(
            api_key=api_key,
            account_id=account_id,
            environment=environment,
        )
    except ImportError as e:
        raise ImportError(
            "OANDA adapter requires the nautilus_pyo3.oanda module. "
            "Ensure the Rust OANDA adapter is built and installed."
        ) from e


def get_oanda_transaction_client(
    api_key: str,
    account_id: str,
    is_live: bool = False,
) -> Any:
    """
    Create an OANDA transaction streaming client.

    Parameters
    ----------
    api_key : str
        The OANDA API access token.
    account_id : str
        The OANDA account ID.
    is_live : bool, default False
        If True, use the live environment. Otherwise use practice.

    Returns
    -------
    OANDATransactionStreamClient
        The OANDA transaction stream client (from Rust adapter).

    """
    try:
        from nautilus_trader.core.nautilus_pyo3 import oanda

        environment = oanda.OANDAEnvironment.Live if is_live else oanda.OANDAEnvironment.Practice

        return oanda.OANDATransactionStreamClient(
            api_key=api_key,
            account_id=account_id,
            environment=environment,
        )
    except ImportError as e:
        raise ImportError(
            "OANDA adapter requires the nautilus_pyo3.oanda module. "
            "Ensure the Rust OANDA adapter is built and installed."
        ) from e


@lru_cache(1)
def get_cached_oanda_http_client(
    api_key: str,
    account_id: str,
    is_live: bool = False,
    timeout_secs: int | None = None,
    max_retries: int | None = None,
) -> Any:
    """
    Cache and return an OANDA HTTP client.

    If a cached client with matching parameters already exists, the cached client will be returned.

    Parameters
    ----------
    api_key : str
        The OANDA API access token.
    account_id : str
        The OANDA account ID.
    is_live : bool, default False
        If True, use the live environment. Otherwise use practice.
    timeout_secs : int, optional
        The HTTP request timeout in seconds.
    max_retries : int, optional
        The maximum number of retry attempts.

    Returns
    -------
    OANDAHttpClient
        The cached OANDA HTTP client.

    """
    return get_oanda_http_client(
        api_key=api_key,
        account_id=account_id,
        is_live=is_live,
        timeout_secs=timeout_secs,
        max_retries=max_retries,
    )


@lru_cache(1)
def get_cached_oanda_instrument_provider(
    client: Any,
    config: OANDAInstrumentProviderConfig | None = None,
) -> OANDAInstrumentProvider:
    """
    Cache and return an OANDA instrument provider.

    If a cached provider already exists, then that provider will be returned.

    Parameters
    ----------
    client : OANDAHttpClient
        The OANDA HTTP client.
    config : OANDAInstrumentProviderConfig, optional
        The instrument provider configuration.

    Returns
    -------
    OANDAInstrumentProvider

    """
    return OANDAInstrumentProvider(
        client=client,
        config=config,
    )


class OANDALiveDataClientFactory(LiveDataClientFactory):
    """
    Provides an OANDA live data client factory.
    """

    @staticmethod
    def create(  # type: ignore
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: OANDADataClientConfig,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> OANDADataClient:
        """
        Create a new OANDA data client.

        Parameters
        ----------
        loop : asyncio.AbstractEventLoop
            The event loop for the client.
        name : str
            The custom client ID.
        config : OANDADataClientConfig
            The client configuration.
        msgbus : MessageBus
            The message bus for the client.
        cache : Cache
            The cache for the client.
        clock : LiveClock
            The clock for the client.

        Returns
        -------
        OANDADataClient

        """
        api_key = _get_api_key(config.api_key)
        account_id = _get_account_id(config.account_id)

        client = get_cached_oanda_http_client(
            api_key=api_key,
            account_id=account_id,
            is_live=config.is_live,
        )

        stream_client = get_oanda_stream_client(
            api_key=api_key,
            account_id=account_id,
            is_live=config.is_live,
        )

        provider = get_cached_oanda_instrument_provider(
            client=client,
            config=config.instrument_provider,
        )

        return OANDADataClient(
            loop=loop,
            client=client,
            stream_client=stream_client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=provider,
            config=config,
            name=name,
        )


class OANDALiveExecClientFactory(LiveExecClientFactory):
    """
    Provides an OANDA live execution client factory.
    """

    @staticmethod
    def create(  # type: ignore
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: OANDAExecClientConfig,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> OANDAExecutionClient:
        """
        Create a new OANDA execution client.

        Parameters
        ----------
        loop : asyncio.AbstractEventLoop
            The event loop for the client.
        name : str
            The custom client ID.
        config : OANDAExecClientConfig
            The client configuration.
        msgbus : MessageBus
            The message bus for the client.
        cache : Cache
            The cache for the client.
        clock : LiveClock
            The clock for the client.

        Returns
        -------
        OANDAExecutionClient

        """
        api_key = _get_api_key(config.api_key)
        account_id = _get_account_id(config.account_id)

        client = get_cached_oanda_http_client(
            api_key=api_key,
            account_id=account_id,
            is_live=config.is_live,
        )

        transaction_client = get_oanda_transaction_client(
            api_key=api_key,
            account_id=account_id,
            is_live=config.is_live,
        )

        provider = get_cached_oanda_instrument_provider(
            client=client,
            config=config.instrument_provider,
        )

        return OANDAExecutionClient(
            loop=loop,
            client=client,
            transaction_client=transaction_client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=provider,
            config=config,
            name=name,
        )
