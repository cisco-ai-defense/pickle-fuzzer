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

use cisco_ai_defense_pickle_fuzzer::{Generator, Version};

#[test]
fn test_generate_all_protocol_versions() {
    for version_num in 0..=5 {
        let version = Version::try_from(version_num).unwrap();
        let mut gen = Generator::new(version);
        let result = gen.generate();
        assert!(
            result.is_ok(),
            "Failed to generate for version {}",
            version_num
        );
        let pickle = result.unwrap();
        assert!(
            !pickle.is_empty(),
            "Empty pickle for version {}",
            version_num
        );
        // All pickles should end with STOP opcode (0x2e or '.')
        assert_eq!(
            pickle[pickle.len() - 1],
            b'.',
            "Missing STOP opcode for version {}",
            version_num
        );
    }
}

#[test]
fn test_deterministic_generation_with_seed() {
    let seed = 42;
    let mut gen1 = Generator::new(Version::V3).with_seed(seed);
    let mut gen2 = Generator::new(Version::V3).with_seed(seed);

    let pickle1 = gen1.generate().unwrap();
    let pickle2 = gen2.generate().unwrap();

    assert_eq!(
        pickle1, pickle2,
        "Same seed should produce identical pickles"
    );
}

#[test]
fn test_different_seeds_produce_different_pickles() {
    let mut gen1 = Generator::new(Version::V3).with_seed(1);
    let mut gen2 = Generator::new(Version::V3).with_seed(2);

    let pickle1 = gen1.generate().unwrap();
    let pickle2 = gen2.generate().unwrap();

    assert_ne!(
        pickle1, pickle2,
        "Different seeds should produce different pickles"
    );
}

#[test]
fn test_opcode_range_configuration() {
    let mut gen = Generator::new(Version::V3)
        .with_min_opcodes(10)
        .with_max_opcodes(20);

    let result = gen.generate();
    assert!(result.is_ok());
}

#[test]
fn test_reset_clears_state() {
    let mut gen = Generator::new(Version::V3);
    gen.generate().unwrap();

    let output_before = gen.output.len();
    assert!(output_before > 0);

    gen.reset();
    assert_eq!(gen.output.len(), 0, "Reset should clear output");
}

#[test]
fn test_builder_pattern() {
    let mut gen = Generator::new(Version::V4)
        .with_seed(123)
        .with_buffer_size(4096)
        .with_opcode_range(50, 100);

    let result = gen.generate();
    assert!(result.is_ok());
}

#[test]
fn test_protocol_v0_generates_ascii() {
    let mut gen = Generator::new(Version::V0).with_seed(999);
    let pickle = gen.generate().unwrap();

    // Protocol 0 should not have PROTO opcode
    assert_ne!(
        pickle[0], 0x80,
        "Protocol 0 should not start with PROTO opcode"
    );
}

#[test]
fn test_protocol_v2_and_above_have_proto() {
    for version_num in 2..=5 {
        let version = Version::try_from(version_num).unwrap();
        let mut gen = Generator::new(version).with_seed(123);
        let pickle = gen.generate().unwrap();

        // Protocol 2+ should start with PROTO opcode (0x80)
        assert_eq!(
            pickle[0], 0x80,
            "Protocol {} should start with PROTO opcode",
            version_num
        );
        assert_eq!(
            pickle[1], version_num as u8,
            "Protocol byte should match version"
        );
    }
}
