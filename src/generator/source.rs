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

//! entropy source abstraction for pickle generation.
//!
//! this module provides a unified interface for generating random values from
//! different entropy sources, enabling the same generation logic to work in both
//! standalone CLI mode and coverage-guided fuzzing mode.
//!
//! # Architecture
//!
//! the `GenerationSource` enum wraps two different entropy sources:
//! - **`Rand`**: uses ChaCha8Rng for deterministic, seeded generation in CLI mode
//! - **`Arbitrary`**: consumes fuzzer-provided bytes for coverage-guided exploration
//!
//! the `EntropySource` trait provides a common interface with methods for generating
//! various primitive types (bool, integers, floats, bytes, strings). all methods
//! provide sensible fallback values when fuzzer bytes are exhausted.
//!
//! # Fuzzing Integration
//!
//! when used with a fuzzing engine (Atheris, libFuzzer), the fuzzer provides raw
//! bytes that are consumed by `Unstructured`. the fuzzer observes code coverage
//! and mutates the input bytes to explore different execution paths. this enables
//! structure-aware fuzzing where the fuzzer learns which byte patterns produce
//! interesting pickle opcodes and stack states.
//!
//! # Determinism
//!
//! both entropy sources are deterministic:
//! - `Rand` mode: same seed produces identical pickles
//! - `Arbitrary` mode: same input bytes produce identical pickles
//!
//! this is critical for reproducibility in testing and debugging.

use arbitrary::Unstructured;
use rand::{Rng, TryRngCore};
use rand_chacha::ChaCha8Rng;

/// source of entropy for pickle generation.
///
/// this enum abstracts over two different entropy sources:
/// - `Rand`: traditional PRNG for standalone/CLI usage
/// - `Arbitrary`: fuzzer-provided bytes for coverage-guided fuzzing
///
/// by abstracting the entropy source, we can share the same generation logic
/// between CLI mode (which uses `rand`) and fuzzing mode (which uses `arbitrary`),
/// while maintaining deterministic, structure-aware pickle generation.
pub enum GenerationSource<'a> {
    /// random number generator for standalone generation.
    ///
    /// used by the CLI and when generating pickles with a seed for reproducibility.
    /// provides unlimited entropy via a PRNG.
    Rand(&'a mut ChaCha8Rng),

    /// fuzzer-provided bytes for coverage-guided fuzzing.
    ///
    /// used when integrating with fuzzing engines like Atheris or libFuzzer.
    /// consumes bytes from the fuzzer's input, enabling coverage-guided exploration.
    /// when bytes are exhausted, `Unstructured` provides deterministic fallback values.
    Arbitrary(&'a mut Unstructured<'a>),
}

/// trait for abstracting entropy sources used in pickle generation.
///
/// this trait provides a unified interface for generating random values
/// from either a PRNG (`rand`) or fuzzer-provided bytes (`arbitrary`).
/// all methods provide fallback values when fuzzer bytes are exhausted.
pub trait EntropySource {
    /// choose a random index in range [0, max).
    ///
    /// returns 0 if max is 0 or fuzzer bytes exhausted.
    fn choose_index(&mut self, max: usize) -> usize;

    /// generate a random boolean value.
    fn gen_bool(&mut self) -> bool;

    /// generate a random u8 value.
    fn gen_u8(&mut self) -> u8;

    /// generate a random u16 value.
    fn gen_u16(&mut self) -> u16;

    /// generate a random u32 value.
    fn gen_u32(&mut self) -> u32;

    /// generate a random i32 value.
    fn gen_i32(&mut self) -> i32;

    /// generate a random i64 value.
    #[allow(dead_code)]
    fn gen_i64(&mut self) -> i64;

    /// generate a random f64 value.
    fn gen_f64(&mut self) -> f64;

    /// generate a random value in the given range [min, max).
    fn gen_range(&mut self, min: usize, max: usize) -> usize;

    /// generate random bytes of the specified length.
    #[allow(dead_code)]
    fn gen_bytes(&mut self, len: usize) -> Vec<u8>;

    /// generate a random printable ASCII character.
    fn gen_ascii_char(&mut self) -> char;
}

/// printable ASCII characters for string generation (space through tilde).
///
/// includes lowercase, uppercase, digits, and common punctuation/symbols.
/// used by `gen_ascii_char()` to generate valid string content for pickle values.
const ASCII_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// implementation of `EntropySource` for `GenerationSource`.
///
/// each method dispatches to either the PRNG or the `Unstructured` fuzzer bytes,
/// with fallback values (typically 0 or false) when fuzzer bytes are exhausted.
/// this ensures generation never fails due to insufficient entropy.
impl<'a> EntropySource for GenerationSource<'a> {
    fn choose_index(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        match self {
            GenerationSource::Rand(rng) => rng.random_range(0..max),
            // fallback to 0 if fuzzer bytes exhausted
            GenerationSource::Arbitrary(u) => u.choose_index(max).unwrap_or(0),
        }
    }

    fn gen_bool(&mut self) -> bool {
        match self {
            GenerationSource::Rand(rng) => rng.random(),
            // fallback to false if fuzzer bytes exhausted
            GenerationSource::Arbitrary(u) => u.arbitrary().unwrap_or(false),
        }
    }

    fn gen_u8(&mut self) -> u8 {
        match self {
            GenerationSource::Rand(rng) => rng.random(),
            GenerationSource::Arbitrary(u) => u.arbitrary().unwrap_or(0),
        }
    }

    fn gen_u16(&mut self) -> u16 {
        match self {
            GenerationSource::Rand(rng) => rng.random(),
            GenerationSource::Arbitrary(u) => u.arbitrary().unwrap_or(0),
        }
    }

    fn gen_u32(&mut self) -> u32 {
        match self {
            GenerationSource::Rand(rng) => rng.random(),
            GenerationSource::Arbitrary(u) => u.arbitrary().unwrap_or(0),
        }
    }

    fn gen_i32(&mut self) -> i32 {
        match self {
            GenerationSource::Rand(rng) => rng.random(),
            GenerationSource::Arbitrary(u) => u.arbitrary().unwrap_or(0),
        }
    }

    fn gen_i64(&mut self) -> i64 {
        match self {
            GenerationSource::Rand(rng) => rng.random(),
            GenerationSource::Arbitrary(u) => u.arbitrary().unwrap_or(0),
        }
    }

    fn gen_f64(&mut self) -> f64 {
        match self {
            GenerationSource::Rand(rng) => rng.random(),
            GenerationSource::Arbitrary(u) => {
                // arbitrary crate doesn't have float64(), use arbitrary() instead
                u.arbitrary().unwrap_or(0.0)
            }
        }
    }

    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        match self {
            GenerationSource::Rand(rng) => rng.random_range(min..max),
            // convert exclusive range to inclusive for arbitrary, fallback to min
            GenerationSource::Arbitrary(u) => {
                u.int_in_range(min..=max.saturating_sub(1)).unwrap_or(min)
            }
        }
    }

    fn gen_bytes(&mut self, len: usize) -> Vec<u8> {
        match self {
            GenerationSource::Rand(rng) => {
                let mut bytes = vec![0u8; len];
                // fill with random bytes, ignore errors (keeps zeros on failure)
                rng.try_fill_bytes(&mut bytes).unwrap_or(());
                bytes
            }
            GenerationSource::Arbitrary(u) => {
                // try to get bytes from fuzzer input, fallback to zeros if exhausted
                u.bytes(len).unwrap_or(&vec![0u8; len]).to_vec()
            }
        }
    }

    fn gen_ascii_char(&mut self) -> char {
        // choose random index into ASCII_CHARS, convert byte to char
        let idx = self.choose_index(ASCII_CHARS.len());
        ASCII_CHARS[idx] as char
    }
}
