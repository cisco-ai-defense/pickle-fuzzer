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
from pickle_fuzzer import Generator

with atheris.instrument_imports():
    import pickletools

MAX_INPUT_BYTES = 4096
PROTOCOLS = 6


def validate_pickle(pickle_bytes: bytes) -> None:
    last_opcode = None
    last_pos = None

    for last_opcode, _arg, last_pos in pickletools.genops(pickle_bytes):
        pass

    if last_opcode is None or last_pos is None:
        raise ValueError("generated pickle did not contain any opcodes")

    if last_opcode.name != "STOP":
        raise ValueError(f"generated pickle terminated with {last_opcode.name} instead of STOP")

    if last_pos != len(pickle_bytes) - 1:
        trailing = len(pickle_bytes) - last_pos - 1
        raise ValueError(f"generated pickle has {trailing} trailing byte(s) after STOP")


@atheris.instrument_func
def TestOneInput(data: bytes) -> None:
    if not data:
        return
    if len(data) > MAX_INPUT_BYTES:
        data = data[:MAX_INPUT_BYTES]
    proto = data[0] % PROTOCOLS
    pickle_bytes = Generator(protocol=proto).generate_from_bytes(data[1:])
    validate_pickle(pickle_bytes)


def main() -> None:
    atheris.Setup(sys.argv, TestOneInput)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
