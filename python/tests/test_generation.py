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

def test_generate_from_bytes():
    gen = pickle_fuzzer.Generator(protocol=3)
    fuzzer_input = b"test_fuzzer_input_bytes"

    data1 = gen.generate_from_bytes(fuzzer_input)
    gen.reset()
    data2 = gen.generate_from_bytes(fuzzer_input)

    assert data1 == data2  # same input bytes means same output

