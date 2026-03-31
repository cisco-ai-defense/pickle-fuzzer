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
import py_compile
from pathlib import Path

import pickle_fuzzer.fuzzer as fuzz_module
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
        def reset(self):
            return None

        def generate_from_bytes(self, *args, **kwargs):
            raise RuntimeError("boom")

    monkeypatch.setattr(mutator, "generator", FailingGenerator())

    data = mutator.mutate(b"fuzzer_input_bytes", max_size=32)

    assert len(data) <= 32
    assert_whole_pickle_consumed(data)


def test_pickle_mutator_reuse_matches_fresh_instance():
    reused_mutator = PickleMutator(protocol=4, seed=123)
    fresh_mutator = PickleMutator(protocol=4, seed=123)

    first = reused_mutator.mutate(b"alpha", max_size=256)
    second = reused_mutator.mutate(b"beta", max_size=256)
    fresh_second = fresh_mutator.mutate(b"beta", max_size=256)

    assert second == fresh_second
    assert second != first
    assert_whole_pickle_consumed(second)


def test_fuzz_pickle_parser_swallows_parser_exceptions_without_logging(
    monkeypatch, capsys
):
    captured = {}

    monkeypatch.setattr(fuzz_module.atheris, "instrument_func", lambda func: func)
    monkeypatch.setattr(
        fuzz_module.atheris,
        "Setup",
        lambda _argv, func: captured.setdefault("callback", func),
    )
    monkeypatch.setattr(fuzz_module.atheris, "Fuzz", lambda: None)

    def failing_parser(_data: bytes) -> None:
        raise ValueError("boom")

    fuzz_module.fuzz_pickle_parser(failing_parser, protocol=3, use_structure_aware=False)
    captured["callback"](b"input")

    output = capsys.readouterr()
    assert output.out == ""
    assert output.err == ""


def test_example_harness_compiles():
    harness_path = Path(__file__).resolve().parents[1] / "examples" / "harness.py"
    py_compile.compile(str(harness_path), doraise=True)
