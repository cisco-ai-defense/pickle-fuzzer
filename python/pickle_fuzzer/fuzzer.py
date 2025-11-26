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

"""Atheris integration utilities for pickle-fuzzer fuzzing."""
import sys
from typing import Optional, Callable

import atheris

from . import Generator


class PickleMutator:
    """structure-aware mutator for pickle fuzzing with atheris.
    
    this class provides a bridge between atheris fuzzing and pickle-fuzzer's
    structure-aware generation. it can be used as a custom mutator in atheris
    to generate valid pickle bytecode from fuzzer input.
    
    example:
        ```python
        import atheris
        from pickle_fuzzer.fuzzer import PickleMutator
        
        mutator = PickleMutator(protocol=4)
        
        @atheris.instrument_func
        def fuzz_target(data):
            pickle_bytes = mutator.mutate(data, max_size=10000)
            # test your pickle parser with pickle_bytes
            ...
        
        atheris.Setup(sys.argv, fuzz_target)
        atheris.Fuzz()
        ```
    """
    
    def __init__(self, protocol: int = 3, seed: Optional[int] = None):
        """initialize the mutator with a specific pickle protocol.
        
        args:
            protocol: pickle protocol version (0-5), default 3
            seed: optional seed for deterministic generation
        """
        self.protocol = protocol
        self.generator = Generator(protocol=protocol, seed=seed)
    
    def mutate(self, data: bytes, max_size: int) -> bytes:
        """mutate pickle data using structure-aware mutations.
        
        uses the fuzzer-provided data as a seed for generating new pickle
        bytecode. the generated pickle will be valid according to the
        specified protocol version.
        
        args:
            data: fuzzer-provided input data
            max_size: maximum size of generated pickle
            
        returns:
            generated pickle bytecode
        """
        try:
            # use fuzzer bytes to generate new pickle
            result = self.generator.generate_from_bytes(data)
            if len(result) <= max_size:
                return result
            # if too large, truncate to max_size (may be invalid)
            return result[:max_size]
        except Exception:
            # if generation fails, return original data
            return data[:max_size] if len(data) > max_size else data
    
    def reset(self):
        """reset the generator state."""
        self.generator.reset()

def fuzz_pickle_parser(
    parser_func: Callable[[bytes], None],
    protocol: int = 3,
    use_structure_aware: bool = True,
) -> None:
    """
    Atheris harness for fuzzing pickle parsers.

    Args:
        parser_func: Function that takes pickle bytes and parses them
        protocol: Pickle protocol version (0-5)
        use_structure_aware: Use structure-aware generation
    """
    generator = Generator(protocol=protocol)

    @atheris.instrument_func
    def test_one_input(data: bytes) -> None:
        try:
            if use_structure_aware:
                # Generate structured pickle from fuzzer bytes
                pickle_bytes = generator.generate_from_bytes(data)
            else:
                # Use raw fuzzer bytes
                pickle_bytes = data

            # Test the parser
            parser_func(pickle_bytes)
        except Exception:
            print(f'Failed to parse pickle: {data}')
            pass  # Expected - we're looking for crashes/hangs

    atheris.Setup(sys.argv, test_one_input)
    atheris.Fuzz()


__all__ = ["PickleMutator", "Generator"]
