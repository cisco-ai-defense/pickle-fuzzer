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

import pickletools

from pickle_fuzzer.fuzzer import PickleMutator


def assert_whole_pickle_consumed(data: bytes) -> None:
    stop_position = None
    for _, _, position in pickletools.genops(data):
        stop_position = position + 1
    assert stop_position is not None
    assert stop_position == len(data)


def test_pickle_mutator_respects_max_size():
    mutator = PickleMutator(protocol=4, seed=123)

    data = mutator.mutate(b"fuzzer_input_bytes", max_size=48)

    assert len(data) <= 48
    assert_whole_pickle_consumed(data)


def test_pickle_mutator_fallback_stays_valid(monkeypatch):
    mutator = PickleMutator(protocol=4, seed=123)

    class FailingGenerator:
        def generate_from_bytes(self, *args, **kwargs):
            raise RuntimeError("boom")

    monkeypatch.setattr(mutator, "generator", FailingGenerator())

    data = mutator.mutate(b"fuzzer_input_bytes", max_size=32)

    assert len(data) <= 32
    assert_whole_pickle_consumed(data)
