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

import sys

import atheris
from pickle_fuzzer.fuzzer import PickleMutator

with atheris.instrument_imports():
    import pickletools

MAX_INPUT_BYTES = 4096
MAX_PICKLE_BYTES = 256
MUTATORS = [PickleMutator(protocol=proto) for proto in range(6)]


@atheris.instrument_func
def TestOneInput(data: bytes) -> None:
    if not data:
        return
    if len(data) > MAX_INPUT_BYTES:
        data = data[:MAX_INPUT_BYTES]
    proto = data[0] % len(MUTATORS)
    pickle_bytes = MUTATORS[proto].mutate(data[1:], max_size=MAX_PICKLE_BYTES)
    try:
        for _ in pickletools.genops(pickle_bytes):
            pass
    except Exception:
        pass


def main() -> None:
    atheris.Setup(sys.argv, TestOneInput)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
