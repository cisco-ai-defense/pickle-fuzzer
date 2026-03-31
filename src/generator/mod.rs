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

//! pickle generator module.
//!
//! this module contains the core `Generator` struct and its implementation,
//! organized into focused submodules for maintainability.
//!
//! # Module Organization
//!
//! - `source`: entropy source abstraction (rand vs arbitrary)
//! - `core`: main generation loop and PROTO/FRAME handling
//! - `emission`: opcode emission methods (emit_int, emit_string, etc.)
//! - `validation`: opcode validation (can_emit, get_valid_opcodes)
//! - `stack_ops`: stack simulation (process_stack_ops, cleanup_for_stop)
//! - `utils`: helper methods (peek, push, pop, has_mark, is_*_at)
//! - `mutation`: mutation support (mutate_*, create_snapshot)

mod core;
mod emission;
mod mutation;
mod source;
mod stack_ops;
mod utils;
mod validation;

pub use source::{EntropySource, GenerationSource};

// ---8<--- module declarations above; Generator definition and imports below ---8<---
use arbitrary::Unstructured;
use color_eyre::Result;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::mutators::Mutator;
use super::protocol::Version;
use super::state::State;

const MAX_OPCODE_RANGE_BOUND: usize = 50_000;

fn normalize_opcode_range(min: usize, max: usize) -> (usize, usize) {
    let min = min.min(MAX_OPCODE_RANGE_BOUND);
    let max = max.min(MAX_OPCODE_RANGE_BOUND);

    if min <= max {
        (min, max)
    } else {
        (max, min)
    }
}

/// stateful pickle generator that produces valid pickle bytecode.
///
/// the generator maintains a simulated pickle virtual machine (PVM) stack and memo
/// table to ensure that only valid opcode sequences are emitted. it uses structure-aware
/// generation to produce complex but valid pickles across all protocol versions.
///
/// # Structure-Aware Generation
///
/// unlike naive byte-level fuzzing, this generator:
/// - simulates the pickle VM stack to ensure valid opcode sequences
/// - tracks memoization to enable GET/PUT operations
/// - validates opcodes against protocol version constraints
/// - ensures STOP opcode has exactly one item on stack
///
/// # Entropy Sources
///
/// supports two entropy sources for generation decisions:
/// - **rand**: traditional PRNG for CLI/standalone use
/// - **arbitrary**: fuzzer-provided bytes for coverage-guided fuzzing
///
/// # Examples
///
/// ```no_run
/// use pickle_fuzzer::{Generator, Version};
///
/// // basic generation with random seed
/// let mut gen = Generator::new(Version::V4);
/// let pickle_bytes = gen.generate().unwrap();
///
/// // deterministic generation with seed
/// let mut gen = Generator::new(Version::V3)
///     .with_seed(42)
///     .with_opcode_range(10, 50);
/// let pickle = gen.generate().unwrap();
///
/// // fuzzer-driven generation
/// let fuzzer_input = b"fuzzer_provided_bytes";
/// let mut gen = Generator::new(Version::V3);
/// let pickle = gen.generate_from_arbitrary(fuzzer_input).unwrap();
/// ```
#[derive(Debug)]
pub struct Generator {
    /// generator state (stack, memo, protocol version, etc.)
    pub state: State,

    /// generated output bytecode
    pub output: Vec<u8>,

    /// optional seed for the PRNG (if None, uses OS entropy)
    pub seed: Option<u64>,

    /// maximum pickle size for generated output
    pub bufsize: Option<usize>,

    /// minimum number of opcodes to generate
    pub min_opcodes: usize,

    /// maximum number of opcodes to generate
    pub max_opcodes: usize,

    /// active mutators for argument mutation
    pub mutators: Vec<Box<dyn Mutator>>,

    /// mutation rate (0.0-1.0)
    pub mutation_rate: f64,

    /// allow unsafe mutations that may violate pickle validity
    pub unsafe_mutations: bool,

    /// allow EXT* opcodes (requires configured extension registry)
    pub allow_ext_opcodes: bool,

    /// allow NEXT_BUFFER/READONLY_BUFFER opcodes (requires out-of-band buffers)
    pub allow_buffer_opcodes: bool,

    /// allow PERSID/BINPERSID opcodes (requires persistent_load support)
    pub allow_persistent_id_opcodes: bool,
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            state: State::default(),
            output: Vec::new(),
            seed: None,
            bufsize: None,
            min_opcodes: 60,
            max_opcodes: 300,
            mutators: Vec::new(),
            mutation_rate: 0.1,
            unsafe_mutations: false,
            allow_ext_opcodes: false,
            allow_buffer_opcodes: false,
            allow_persistent_id_opcodes: false,
        }
    }
}

impl Generator {
    /// create a new generator with the specified protocol version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pickle_fuzzer::{Generator, Version};
    ///
    /// let gen = Generator::new(Version::V3);
    /// ```
    pub fn new(version: Version) -> Self {
        Self {
            state: State::new(version),
            ..Default::default()
        }
    }

    /// reset the generator state for generating a new pickle.
    ///
    /// clears the stack, memo, output buffer, and resets flags.
    /// generation methods already reset automatically before each run, so this
    /// is only needed when clearing state manually between operations.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pickle_fuzzer::{Generator, Version};
    ///
    /// let mut gen = Generator::new(Version::V3);
    /// let pickle1 = gen.generate().unwrap();
    /// gen.reset();
    /// let pickle2 = gen.generate().unwrap();
    /// ```
    pub fn reset(&mut self) {
        self.state.reset();
        self.output.clear();
    }

    /// set a seed for the PRNG (for reproducible generation).
    ///
    /// when a seed is provided, generation becomes deterministic.
    /// same seed + same configuration = same pickle output.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// set a maximum pickle size for generated output.
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.bufsize = Some(size);
        self
    }

    /// set the minimum number of opcodes to generate.
    pub fn with_min_opcodes(mut self, min: usize) -> Self {
        self.min_opcodes = min.min(MAX_OPCODE_RANGE_BOUND);
        if self.max_opcodes < self.min_opcodes {
            self.max_opcodes = self.min_opcodes;
        }
        self
    }

    /// set the maximum number of opcodes to generate.
    pub fn with_max_opcodes(mut self, max: usize) -> Self {
        self.max_opcodes = max.min(MAX_OPCODE_RANGE_BOUND);
        if self.min_opcodes > self.max_opcodes {
            self.min_opcodes = self.max_opcodes;
        }
        self
    }

    /// set both min and max opcodes at once.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pickle_fuzzer::{Generator, Version};
    ///
    /// let gen = Generator::new(Version::V3)
    ///     .with_opcode_range(10, 50);
    /// ```
    pub fn with_opcode_range(mut self, min: usize, max: usize) -> Self {
        let (min, max) = normalize_opcode_range(min, max);
        self.min_opcodes = min;
        self.max_opcodes = max;
        self
    }

    /// update the opcode range in place.
    pub fn set_opcode_range(&mut self, min: usize, max: usize) {
        let (min, max) = normalize_opcode_range(min, max);
        self.min_opcodes = min;
        self.max_opcodes = max;
        self.reset();
    }

    /// add mutators to the generator.
    ///
    /// mutators inject controlled variations during generation for fuzzing.
    pub fn with_mutators(mut self, mutators: Vec<Box<dyn Mutator>>) -> Self {
        self.mutators = mutators;
        self
    }

    /// add a mutator to the generator.
    ///
    /// mutators inject controlled variations during generation for fuzzing.
    pub fn with_mutator(mut self, mutator: Box<dyn Mutator>) -> Self {
        self.mutators.push(mutator);
        self
    }

    /// set the mutation rate (0.0-1.0).
    ///
    /// controls how frequently mutations are applied during generation.
    /// rate is automatically clamped to valid range.
    pub fn with_mutation_rate(mut self, rate: f64) -> Self {
        self.mutation_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// enable unsafe mutations that may violate pickle validity.
    ///
    /// when enabled, allows mutations that can produce invalid pickles.
    /// useful for robustness testing of pickle parsers.
    pub fn with_unsafe_mutations(mut self, unsafe_mutations: bool) -> Self {
        self.unsafe_mutations = unsafe_mutations;
        self
    }

    /// allow EXT* opcodes during generation.
    ///
    /// EXT opcodes require a configured extension registry. enable this only
    /// if you have properly configured the extension registry in your unpickler.
    /// without proper configuration, pickles with EXT opcodes will fail to unpickle.
    pub fn with_ext_opcodes(mut self, allow: bool) -> Self {
        self.allow_ext_opcodes = allow;
        self
    }

    /// allow NEXT_BUFFER/READONLY_BUFFER opcodes during generation.
    ///
    /// buffer opcodes require out-of-band buffer support. enable this only
    /// if you have properly configured buffer callbacks in your unpickler.
    /// without proper configuration, pickles with buffer opcodes will fail to unpickle.
    pub fn with_buffer_opcodes(mut self, allow: bool) -> Self {
        self.allow_buffer_opcodes = allow;
        self
    }

    /// allow PERSID/BINPERSID opcodes during generation.
    ///
    /// persistent-id opcodes require a persistent_load callback in the
    /// unpickler. enable this only if your consumer is configured to resolve
    /// persistent IDs.
    pub fn with_persistent_id_opcodes(mut self, allow: bool) -> Self {
        self.allow_persistent_id_opcodes = allow;
        self
    }

    fn minimum_pickle_size(&self) -> usize {
        let proto_size = if self.state.version >= Version::V2 {
            2
        } else {
            0
        };

        proto_size + 2
    }

    fn generate_with_bufsize<F>(&mut self, max_size: usize, mut attempt: F) -> Result<Vec<u8>>
    where
        F: FnMut(&mut Self, Option<usize>, Option<bool>) -> Result<Vec<u8>>,
    {
        let minimum_size = self.minimum_pickle_size();
        if max_size < minimum_size {
            self.reset();
            return Err(color_eyre::eyre::eyre!(
                "buffer size {} is too small for protocol {} (minimum valid pickle size is {})",
                max_size,
                self.state.version as u8,
                minimum_size
            ));
        }

        let (_, max_budget) = self.normalized_opcode_range();
        let mut last_error = None;

        let mut frame_modes = vec![None];
        if self.state.version >= Version::V4 {
            frame_modes.push(Some(false));
        }

        for force_frame in frame_modes {
            self.reset();
            match attempt(self, None, force_frame) {
                Ok(bytes) if bytes.len() <= max_size => return Ok(bytes),
                Ok(_) => {}
                Err(error) => last_error = Some(error),
            }

            for target_total in (0..=max_budget).rev() {
                self.reset();
                match attempt(self, Some(target_total), force_frame) {
                    Ok(bytes) if bytes.len() <= max_size => return Ok(bytes),
                    Ok(_) => {}
                    Err(error) => last_error = Some(error),
                }
            }
        }

        self.reset();
        match last_error {
            Some(error) => Err(color_eyre::eyre::eyre!(
                "failed to generate pickle within {} bytes: {}",
                max_size,
                error
            )),
            None => Err(color_eyre::eyre::eyre!(
                "failed to generate pickle within {} bytes",
                max_size
            )),
        }
    }

    /// generate a random, but valid pickle opcode stream using PRNG.
    ///
    /// uses `rand` for entropy source. suitable for CLI and standalone use.
    ///
    /// Returns the generated pickle bytecode. The pickle will be valid according
    /// to the protocol version specified when the generator was created.
    pub fn generate(&mut self) -> Result<Vec<u8>> {
        self.reset();

        if let Some(max_size) = self.bufsize {
            let seed = self.seed;
            return self.generate_with_bufsize(
                max_size,
                move |generator, target_total, force_frame| {
                    let mut rng = if let Some(seed) = seed {
                        ChaCha8Rng::seed_from_u64(seed)
                    } else {
                        ChaCha8Rng::from_os_rng()
                    };
                    let mut source = GenerationSource::Rand(&mut rng);
                    generator.generate_internal(&mut source, target_total, force_frame)
                },
            );
        }

        let mut rng = if let Some(seed) = self.seed {
            ChaCha8Rng::seed_from_u64(seed)
        } else {
            ChaCha8Rng::from_os_rng()
        };

        let mut source = GenerationSource::Rand(&mut rng);

        self.generate_internal(&mut source, None, None)
    }

    /// generate a pickle opcode stream from fuzzer-provided bytes.
    ///
    /// uses `arbitrary` crate to consume fuzzer bytes for generation decisions.
    /// maintains all structure-aware validation while using fuzzer input.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pickle_fuzzer::{Generator, Version};
    ///
    /// let fuzzer_input = b"fuzzer_bytes_here";
    /// let mut gen = Generator::new(Version::V3);
    /// let pickle = gen.generate_from_arbitrary(fuzzer_input).unwrap();
    /// ```
    pub fn generate_from_arbitrary(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.reset();

        if let Some(max_size) = self.bufsize {
            return self.generate_with_bufsize(max_size, |generator, target_total, force_frame| {
                let mut u = Unstructured::new(data);
                let mut source = GenerationSource::Arbitrary(&mut u);
                generator.generate_internal(&mut source, target_total, force_frame)
            });
        }

        let mut u = Unstructured::new(data);
        let mut source = GenerationSource::Arbitrary(&mut u);
        self.generate_internal(&mut source, None, None)
    }

    pub(crate) fn normalized_opcode_range(&self) -> (usize, usize) {
        normalize_opcode_range(self.min_opcodes, self.max_opcodes)
    }
}
