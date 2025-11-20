// SPDX-License-Identifier: Apache-2.0
//
// Copyright 2025 Cisco Systems, Inc. and its affiliates
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! fuzz target that validates generated pickles with Python's pickletools module.
//!
//! this target exercises the generator with fuzzer-controlled configuration:
//! - protocol version (0-5)
//! - opcode range (min/max opcodes to generate)
//! - mutation rate (0.0-1.0)
//! - mutator selection (via bit flags)
//! - arbitrary data seed for generation
//!
//! the generated pickles are validated using Python's `pickletools.genops()` (same
//! validation logic as scripts/validate-pickles.py) to ensure they are structurally
//! valid and can be parsed by the reference implementation. this catches bugs in:
//! - opcode emission logic
//! - stack simulation
//! - protocol version handling
//! - mutator implementations
//!
//! # Input Format
//!
//! The fuzzer input is structured as follows:
//! - Byte 0: Protocol version selector (modulo 6 → 0-5)
//! - Bytes 1-2: Minimum opcode count (little-endian u16)
//! - Bytes 3-4: Maximum opcode count (little-endian u16)
//! - Byte 5: Mutation rate (0-255 → 0.0-1.0)
//! - Byte 6: Mutator flags (bit field)
//!   - Bit 0 (0x01): BitFlipMutator
//!   - Bit 1 (0x02): BoundaryMutator
//!   - Bit 2 (0x04): OffByOneMutator
//!   - Bit 3 (0x08): StringLengthMutator
//!   - Bit 4 (0x10): CharacterMutator
//!   - Bit 5 (0x20): MemoIndexMutator (unsafe mode if bit 6 set)
//! - Bytes 7+: Arbitrary data seed for generation
//!
//! # Validation
//!
//! Generated pickles must:
//! 1. Be non-empty
//! 2. End with STOP opcode (0x2e / '.')
//! 3. Successfully parse with `pickletools.genops()` (via Python subprocess)
#![no_main]

use libfuzzer_sys::fuzz_target;
use pickle_whip::{Generator, Version};
use pickle_whip::mutators::{
    BitFlipMutator, BoundaryMutator, OffByOneMutator,
    StringLengthMutator, CharacterMutator, MemoIndexMutator,
};
use std::process::{Command, Stdio};
use std::io::Write;

/// Validate pickle using Python's pickletools.genops() (same as validate-pickles.py)
fn validate_with_python(pickle_bytes: &[u8]) -> bool {
    let mut child = match Command::new("python3")
        .arg("-c")
        .arg("import sys, pickletools; list(pickletools.genops(sys.stdin.buffer.read()))")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return true, // Skip validation if Python unavailable
    };
    
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(pickle_bytes);
    }
    
    child.wait().map(|status| status.success()).unwrap_or(false)
}

fuzz_target!(|data: &[u8]| {
    // need at least 7 bytes for configuration + some arbitrary data
    if data.len() < 7 {
        return;
    }

    // byte 0: protocol version (0-5)
    let protocol = (data[0] % 6) as usize;
    let version = Version::try_from(protocol).unwrap();
    
    let gen = Generator::new(version);

    // bytes 1-4: opcode range (min/max)
    let min_opcodes = u16::from_le_bytes([data[1], data[2]]) as usize;
    let max_opcodes = u16::from_le_bytes([data[3], data[4]]) as usize;
    
    // ensure valid range and cap at 1000 opcodes to prevent stack overflow
    let mut gen = if min_opcodes > max_opcodes  {
        if min_opcodes > 1000 {
            return;
        }
        gen.with_opcode_range(max_opcodes, min_opcodes)
    } else {
        if max_opcodes > 1000 {
            return;
        }
        gen.with_opcode_range(min_opcodes, max_opcodes)
    };

    // byte 5: mutation rate configuration
    let config = data[5];
    let mutation_rate = (config as f64) / 255.0; // map 0-255 to 0.0-1.0
    gen = gen.with_mutation_rate(mutation_rate);

    // byte 6: mutator selection via bit flags
    let mutator_flags = data[6];
    if mutator_flags & 0x01 != 0 {
        gen = gen.with_mutator(Box::new(BitFlipMutator));
    }
    if mutator_flags & 0x02 != 0 {
        gen = gen.with_mutator(Box::new(BoundaryMutator));
    }
    if mutator_flags & 0x04 != 0 {
        gen = gen.with_mutator(Box::new(OffByOneMutator));
    }
    if mutator_flags & 0x08 != 0 {
        gen = gen.with_mutator(Box::new(StringLengthMutator));
    }
    if mutator_flags & 0x10 != 0 {
        gen = gen.with_mutator(Box::new(CharacterMutator));
    }
    if mutator_flags & 0x20 != 0 {
        let unsafe_mode = mutator_flags & 0x20 % 2 == 1;
        gen = gen.with_mutator(Box::new(MemoIndexMutator::new(unsafe_mode)));
    }

    // bytes 7+: arbitrary data seed for generation
    if let Ok(pickle) = gen.generate_from_arbitrary(&data[7..]) {
        // basic structural validation
        assert!(!pickle.is_empty(), "generated pickle must not be empty");
        assert_eq!(pickle[pickle.len() - 1], b'.', "pickle must end with STOP opcode");
        
        // validate with Python's pickletools
        assert!(
            validate_with_python(&pickle),
            "generated pickle failed Python validation"
        );
    }
});
