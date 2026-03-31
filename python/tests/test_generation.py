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

import pickle_fuzzer


def test_basic_generation():
    gen = pickle_fuzzer.Generator(protocol=3)
    data = gen.generate()
    assert len(data) > 0
    assert data[-1] == ord(".")


def test_deterministic_generation():
    gen1 = pickle_fuzzer.Generator(protocol=3, seed=42)
    gen2 = pickle_fuzzer.Generator(protocol=3, seed=42)

    data1 = gen1.generate()
    data2 = gen2.generate()

    assert data1 == data2


def test_set_opcode_range_preserves_seeded_determinism():
    gen1 = pickle_fuzzer.Generator(protocol=3, seed=42)
    gen2 = pickle_fuzzer.Generator(protocol=3, seed=42)

    gen1.set_opcode_range(50, 10)
    gen2.set_opcode_range(50, 10)

    assert gen1.generate() == gen2.generate()


def test_generate_from_bytes():
    gen = pickle_fuzzer.Generator(protocol=3)
    fuzzer_input = b"test_fuzzer_input_bytes"

    data1 = gen.generate_from_bytes(fuzzer_input)
    data2 = gen.generate_from_bytes(fuzzer_input)

    assert data1 == data2  # same input bytes means same output


def test_generate_from_bytes_respects_max_size():
    gen = pickle_fuzzer.Generator(protocol=4, seed=123)
    data = gen.generate_from_bytes(b"test_fuzzer_input_bytes", max_size=32)

    assert len(data) <= 32
    assert data[-1] == ord(".")
