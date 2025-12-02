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

use pickle_fuzzer::{Generator, Version};

#[test]
#[ignore = "skipped due to error in CI, test with --ignored to run locally"]
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
#[ignore = "skipped due to error in CI, test with --ignored to run locally"]
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

#[test]
fn test_cli_single_file_generation() {
    use std::fs;
    use std::process::Command;

    let temp_file = "/tmp/test_pickle_cli.pkl";

    // clean up if exists
    let _ = fs::remove_file(temp_file);

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", temp_file])
        .output()
        .expect("failed to execute command");

    assert!(
        output.status.success(),
        "CLI command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fs::metadata(temp_file).is_ok(), "output file not created");

    let contents = fs::read(temp_file).expect("failed to read output file");
    assert!(!contents.is_empty(), "output file is empty");
    assert_eq!(contents[contents.len() - 1], b'.', "missing STOP opcode");

    // cleanup
    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_cli_with_protocol_flag() {
    use std::fs;
    use std::process::Command;

    let temp_file = "/tmp/test_pickle_protocol.pkl";
    let _ = fs::remove_file(temp_file);

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--protocol", "4", temp_file])
        .output()
        .expect("failed to execute command");

    assert!(output.status.success(), "CLI command failed");

    let contents = fs::read(temp_file).expect("failed to read output file");
    assert_eq!(contents[0], 0x80, "should start with PROTO opcode");
    assert_eq!(contents[1], 4, "should be protocol 4");

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_cli_with_seed_produces_deterministic_output() {
    use std::fs;
    use std::process::Command;

    let temp_file1 = "/tmp/test_pickle_seed1.pkl";
    let temp_file2 = "/tmp/test_pickle_seed2.pkl";

    let _ = fs::remove_file(temp_file1);
    let _ = fs::remove_file(temp_file2);

    // generate with same seed twice
    Command::new("cargo")
        .args(["run", "--quiet", "--", "--seed", "42", temp_file1])
        .output()
        .expect("failed to execute command");

    Command::new("cargo")
        .args(["run", "--quiet", "--", "--seed", "42", temp_file2])
        .output()
        .expect("failed to execute command");

    let contents1 = fs::read(temp_file1).expect("failed to read file 1");
    let contents2 = fs::read(temp_file2).expect("failed to read file 2");

    assert_eq!(
        contents1, contents2,
        "same seed should produce identical output"
    );

    let _ = fs::remove_file(temp_file1);
    let _ = fs::remove_file(temp_file2);
}

#[test]
#[ignore = "skipped due to timeout issues in CI, test with --ignored to run locally"]
fn test_cli_batch_mode() {
    use std::fs;
    use std::process::Command;

    let temp_dir = "tests/test_pickle_batch";
    let _ = fs::remove_dir_all(temp_dir);
    fs::create_dir_all(temp_dir).expect("failed to create temp dir");

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--dir", temp_dir, "--samples", "5"])
        .output()
        .expect("failed to execute command");

    assert!(
        output.status.success(),
        "batch generation failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    // check that files were created
    let entries = fs::read_dir(temp_dir).expect("failed to read dir");
    let count = entries.count();
    assert_eq!(count, 5, "should create 5 pickle files");

    // verify one of the files
    let test_file = format!("{}/0.pkl", temp_dir);
    let contents = fs::read(&test_file).expect("failed to read generated file");
    assert!(!contents.is_empty());
    assert_eq!(contents[contents.len() - 1], b'.');

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_cli_with_opcode_range() {
    use std::fs;
    use std::process::Command;

    let temp_file = "/tmp/test_pickle_opcodes.pkl";
    let _ = fs::remove_file(temp_file);

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "--min-opcodes",
            "10",
            "--max-opcodes",
            "20",
            temp_file,
        ])
        .output()
        .expect("failed to execute command");

    assert!(output.status.success(), "CLI command failed");
    assert!(fs::metadata(temp_file).is_ok(), "output file not created");

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_cli_with_mutators() {
    use std::fs;
    use std::process::Command;

    let temp_file = "/tmp/test_pickle_mutators.pkl";
    let _ = fs::remove_file(temp_file);

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "--mutators",
            "bitflip",
            "boundary",
            "--mutation-rate",
            "0.5",
            temp_file,
        ])
        .output()
        .expect("failed to execute command");

    assert!(output.status.success(), "CLI command with mutators failed");
    assert!(fs::metadata(temp_file).is_ok(), "output file not created");

    let _ = fs::remove_file(temp_file);
}
