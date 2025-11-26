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
//
// SPDX-License-Identifier: Apache-2.0

//! Mutation strategies for pickle generation.
//!
//! Mutators allow injecting controlled variations during pickle generation
//! to create more diverse test cases for fuzzing and validation.

use crate::generator::GenerationSource;
use clap::ValueEnum;

use crate::stack::StackObjectRef;

mod bitflip;
mod boundary;
mod character;
mod memoindex;
mod offbyone;
mod stringlen;
mod typeconfusion;

pub use bitflip::BitFlipMutator;
pub use boundary::BoundaryMutator;
pub use character::CharacterMutator;
pub use memoindex::MemoIndexMutator;
pub use offbyone::OffByOneMutator;
pub use stringlen::StringLengthMutator;
pub use typeconfusion::TypeConfusionMutator;

/// Snapshot of generator state before an opcode emission.
///
/// This allows mutators to see what changed after an opcode was emitted
/// and perform post-processing mutations on the bytecode.
#[derive(Debug, Clone)]
pub struct EmissionSnapshot {
    /// Stack depth before emission
    pub stack_depth: usize,

    /// Output buffer length before emission
    pub output_len: usize,

    /// Memo size before emission
    pub memo_size: usize,

    /// Stack items added by this emission (references to new items)
    pub stack_delta: Vec<StackObjectRef>,

    /// Bytecode added by this emission
    pub output_delta: Vec<u8>,

    /// Memo indices added by this emission
    pub memo_delta: Vec<usize>,
}

/// Available mutator types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MutatorKind {
    /// Enable all mutators
    All,
    /// Apply bit flips to integer arguments
    Bitflip,
    /// Use boundary values (0, -1, MAX, MIN)
    Boundary,
    /// Apply off-by-one mutations (±1)
    Offbyone,
    /// Mutate string/bytes lengths
    Stringlen,
    /// Mutate individual characters/bytes
    Character,
    /// Mutate memo indices
    Memoindex,
    /// Type confusion: inject non-string values before opcodes expecting strings
    Typeconfusion,
}

impl MutatorKind {
    /// returns all individual mutator kinds (excluding the "all" meta-option).
    ///
    /// if `unsafe_mutations` is false, excludes MemoIndex mutator since it can
    /// generate invalid memo references even in safe mode.
    pub fn all_mutators(unsafe_mutations: bool) -> Vec<MutatorKind> {
        let mut mutators = vec![
            MutatorKind::Bitflip,
            MutatorKind::Boundary,
            MutatorKind::Offbyone,
            MutatorKind::Stringlen,
            MutatorKind::Character,
            MutatorKind::Typeconfusion,
        ];

        // only include MemoIndex when explicitly using unsafe mutations
        if unsafe_mutations {
            mutators.push(MutatorKind::Memoindex);
        }

        mutators
    }

    /// Create a boxed mutator instance from this kind.
    pub fn create(&self, unsafe_mode: bool) -> Box<dyn Mutator> {
        match self {
            MutatorKind::All => {
                panic!("MutatorKind::All should be expanded before calling create()")
            }
            MutatorKind::Bitflip => Box::new(BitFlipMutator),
            MutatorKind::Boundary => Box::new(BoundaryMutator),
            MutatorKind::Offbyone => Box::new(OffByOneMutator),
            MutatorKind::Stringlen => Box::new(StringLengthMutator),
            MutatorKind::Character => Box::new(CharacterMutator),
            MutatorKind::Memoindex => Box::new(MemoIndexMutator::new(unsafe_mode)),
            MutatorKind::Typeconfusion => Box::new(TypeConfusionMutator::new(unsafe_mode)),
        }
    }
}

/// Trait for implementing mutation strategies.
///
/// Mutators can modify opcode arguments during generation to create
/// variations in the output. Each mutator implements a specific
/// mutation strategy (e.g., bit flips, boundary values, etc.).
pub trait Mutator: Send + Sync + std::fmt::Debug {
    /// Returns the name of this mutator.
    fn name(&self) -> &str;

    /// Attempts to mutate an integer argument.
    ///
    /// Returns `Some(mutated_value)` if mutation was applied,
    /// or `None` if this mutator doesn't apply to integers.
    fn mutate_int(&self, _value: i32, _source: &mut GenerationSource, _rate: f64) -> Option<i32> {
        None
    }

    /// Attempts to mutate a long integer argument.
    fn mutate_long(&self, _value: i64, _source: &mut GenerationSource, _rate: f64) -> Option<i64> {
        None
    }

    /// Attempts to mutate a float argument.
    fn mutate_float(&self, _value: f64, _source: &mut GenerationSource, _rate: f64) -> Option<f64> {
        None
    }

    /// Attempts to mutate a string argument.
    fn mutate_string(
        &self,
        _value: String,
        _source: &mut GenerationSource,
        _rate: f64,
    ) -> Option<String> {
        None
    }

    /// Attempts to mutate a bytes argument.
    fn mutate_bytes(
        &self,
        _value: Vec<u8>,
        _source: &mut GenerationSource,
        _rate: f64,
    ) -> Option<Vec<u8>> {
        None
    }

    /// Attempts to mutate a memo index.
    fn mutate_memo_index(
        &self,
        _index: usize,
        _source: &mut GenerationSource,
        _rate: f64,
    ) -> Option<usize> {
        None
    }

    /// Returns true if this mutator can produce unsafe/invalid mutations.
    fn is_unsafe(&self) -> bool {
        false
    }

    /// Post-process the most recently emitted opcode.
    ///
    /// This method is called after each opcode emission with a snapshot of the
    /// state before emission and the current output buffer. Mutators can inspect
    /// what was emitted and rewrite the bytecode to inject mutations.
    ///
    /// # Arguments
    /// * `snapshot` - State before the opcode was emitted
    /// * `output` - Mutable reference to the output buffer (can be modified)
    /// * `rng` - Random number generator
    /// * `rate` - Mutation rate (0.0-1.0)
    ///
    /// # Returns
    /// `true` if the mutator modified the output, `false` otherwise
    fn post_process(
        &self,
        _snapshot: &EmissionSnapshot,
        _output: &mut Vec<u8>,
        _source: &mut GenerationSource,
        _rate: f64,
    ) -> bool {
        false // Default: no post-processing
    }
}
