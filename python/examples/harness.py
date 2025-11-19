#!/usr/bin/env python3
# Copyright 2025 Cisco Systems, Inc. and its affiliates
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# SPDX-License-Identifier: Apache-2.0

"""Example Atheris harness for fuzzing Python's pickle module."""

import atheris
from pickle_whip.fuzzer import PickleMutator

with atheris.instrument_imports():
    import sys
    import pickle
    # import your fuzz target


def TestOneInput(data: bytes) -> None:
    """Test Python's pickle.loads() with generated data."""
    if not data:
        return
    proto = data[0] % 6
    mutator = PickleMutator(protocol=proto)
    pickle_bytes = mutator.mutate(data[1:], max_size=10000)

    try:
        # call your fuzz target with pickle_bytes
        # e.g. target.parse(pickle_bytes)
    except Exception:
        ...


if __name__ == "__main__":
    atheris.Setup(sys.argv, TestOneInput)
    atheris.Fuzz()
