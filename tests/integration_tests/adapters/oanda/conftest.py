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
OANDA adapter test fixtures.
"""

import pytest

from nautilus_trader.model.identifiers import Venue


OANDA_VENUE = Venue("OANDA")


@pytest.fixture
def venue():
    """Return the OANDA venue."""
    return OANDA_VENUE


@pytest.fixture
def instrument():
    """Return a test instrument (None for basic API tests)."""
    return None


@pytest.fixture
def instrument_provider():
    """Return instrument provider (None for basic API tests)."""
    return None


@pytest.fixture
def data_client():
    """Return data client (None for basic API tests)."""
    return None


@pytest.fixture
def exec_client():
    """Return execution client (None for basic API tests)."""
    return None


@pytest.fixture
def account_state():
    """Return account state (None for basic API tests)."""
    return None
