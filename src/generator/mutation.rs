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

//! mutation application during pickle generation.
//!
//! this module implements the mutation system that applies controlled variations
//! to generated pickle values. mutations are applied during generation to inject
//! interesting edge cases, boundary values, and potential bug triggers into the
//! pickle bytecode.
//!
//! # Mutation Strategy
//!
//! mutations are applied at two levels:
//!
//! 1. **value-level mutations**: applied to individual values (ints, floats, strings,
//!    bytes, memo indices) as they are generated. only one mutator is applied per
//!    value, chosen based on the mutation rate.
//!
//! 2. **post-emission mutations**: applied after an opcode is emitted, allowing
//!    mutators to modify the raw bytecode in the output buffer. this enables
//!    low-level mutations like bit flips and byte corruption.
//!
//! # Mutation Rate
//!
//! the `mutation_rate` parameter (0.0 to 1.0) controls how frequently mutations
//! are applied. each mutator checks this rate independently, so multiple mutators
//! can be active but only one mutation is applied per value.
//!
//! # Safety
//!
//! some mutations can produce invalid pickles (marked with `is_unsafe()` in the
//! mutator trait). these are useful for finding parser bugs but may cause
//! unpickling to fail.

use super::source::GenerationSource;
use super::Generator;
use crate::mutators::EmissionSnapshot;

impl Generator {
    /// apply mutations to an integer value.
    ///
    /// iterates through all registered mutators and applies the first mutation
    /// that triggers (based on mutation rate). returns the original value if no
    /// mutators are registered or none trigger.
    ///
    /// # Parameters
    /// - `value`: the original integer value
    /// - `source`: entropy source for random mutation decisions
    ///
    /// # Returns
    /// the mutated value, or the original if no mutation applied.
    pub(super) fn mutate_int(&self, value: i32, source: &mut GenerationSource) -> i32 {
        if self.mutators.is_empty() {
            return value;
        }

        let mut result = value;
        for mutator in &self.mutators {
            if let Some(mutated) = mutator.mutate_int(result, source, self.mutation_rate) {
                result = mutated;
                break; // Apply only one mutation
            }
        }
        result
    }

    /// apply mutations to a long integer value.
    ///
    /// similar to `mutate_int()` but for 64-bit integers. applies the first
    /// mutation that triggers from the registered mutators.
    ///
    /// # Parameters
    /// - `value`: the original long integer value
    /// - `source`: entropy source for random mutation decisions
    ///
    /// # Returns
    /// the mutated value, or the original if no mutation applied.
    pub(super) fn _mutate_long(&self, value: i64, source: &mut GenerationSource) -> i64 {
        // unused right now, keeping around for completeness/future use
        if self.mutators.is_empty() {
            return value;
        }

        let mut result = value;
        for mutator in &self.mutators {
            if let Some(mutated) = mutator.mutate_long(result, source, self.mutation_rate) {
                result = mutated;
                break;
            }
        }
        result
    }
    /// apply mutations to a float value.
    ///
    /// applies mutations to floating-point values, potentially injecting special
    /// values like NaN, infinity, or boundary values. applies the first mutation
    /// that triggers from the registered mutators.
    ///
    /// # Parameters
    /// - `value`: the original float value
    /// - `source`: entropy source for random mutation decisions
    ///
    /// # Returns
    /// the mutated value, or the original if no mutation applied.
    pub(super) fn mutate_float(&self, value: f64, source: &mut GenerationSource) -> f64 {
        if self.mutators.is_empty() {
            return value;
        }

        let mut result = value;
        for mutator in &self.mutators {
            if let Some(mutated) = mutator.mutate_float(result, source, self.mutation_rate) {
                result = mutated;
                break;
            }
        }
        result
    }

    /// apply mutations to a string value.
    ///
    /// applies mutations to string values, potentially modifying length, inserting
    /// special characters, or corrupting content. applies the first mutation that
    /// triggers from the registered mutators.
    ///
    /// # Parameters
    /// - `value`: the original string value
    /// - `source`: entropy source for random mutation decisions
    ///
    /// # Returns
    /// the mutated string, or the original if no mutation applied.
    pub(super) fn mutate_string(&self, value: String, source: &mut GenerationSource) -> String {
        if self.mutators.is_empty() {
            return value;
        }

        let mut result = value;
        for mutator in &self.mutators {
            if let Some(mutated) = mutator.mutate_string(result.clone(), source, self.mutation_rate)
            {
                result = mutated;
                break;
            }
        }
        result
    }

    /// apply mutations to a bytes value.
    ///
    /// applies mutations to byte sequences, potentially modifying length, flipping
    /// bits, or corrupting content. applies the first mutation that triggers from
    /// the registered mutators.
    ///
    /// # Parameters
    /// - `value`: the original byte vector
    /// - `source`: entropy source for random mutation decisions
    ///
    /// # Returns
    /// the mutated bytes, or the original if no mutation applied.
    pub(super) fn mutate_bytes(&self, value: Vec<u8>, source: &mut GenerationSource) -> Vec<u8> {
        if self.mutators.is_empty() {
            return value;
        }

        let mut result = value;
        for mutator in &self.mutators {
            if let Some(mutated) = mutator.mutate_bytes(result.clone(), source, self.mutation_rate)
            {
                result = mutated;
                break;
            }
        }
        result
    }

    /// apply mutations to a memo index.
    ///
    /// applies mutations to memoization indices, potentially creating invalid
    /// references or off-by-one errors. applies the first mutation that triggers
    /// from the registered mutators.
    ///
    /// # Parameters
    /// - `index`: the original memo index
    /// - `source`: entropy source for random mutation decisions
    ///
    /// # Returns
    /// the mutated index, or the original if no mutation applied.
    pub(super) fn mutate_memo_index(&self, index: usize, source: &mut GenerationSource) -> usize {
        if self.mutators.is_empty() {
            return index;
        }

        let mut result = index;
        for mutator in &self.mutators {
            if let Some(mutated) = mutator.mutate_memo_index(result, source, self.mutation_rate) {
                result = mutated;
                break;
            }
        }
        result
    }

    /// create a snapshot of current generator state before emitting an opcode.
    ///
    /// captures the current stack depth, output buffer length, and memo size.
    /// this snapshot is used by `post_process_emission()` to calculate what
    /// changed during opcode emission, enabling post-emission mutations.
    ///
    /// # Returns
    /// an `EmissionSnapshot` with current state, empty deltas to be filled later.
    pub(super) fn create_snapshot(&self) -> EmissionSnapshot {
        EmissionSnapshot {
            stack_depth: self.state.stack.len(),
            output_len: self.output.len(),
            memo_size: self.state.memo.len(),
            stack_delta: Vec::new(),
            output_delta: Vec::new(),
            memo_delta: Vec::new(),
        }
    }

    /// apply post-processing mutations after an opcode emission.
    ///
    /// calculates the deltas (changes) since the snapshot was created and allows
    /// mutators to modify the emitted bytecode directly. this enables low-level
    /// mutations like bit flips, byte corruption, and opcode argument tampering.
    ///
    /// the deltas capture:
    /// - **stack_delta**: new items pushed to the stack (if stack grew)
    /// - **output_delta**: new bytes written to the output buffer
    /// - **memo_delta**: new indices added to the memo table
    ///
    /// each mutator can inspect these deltas and modify the output buffer based
    /// on the mutation rate.
    ///
    /// # Parameters
    /// - `snapshot`: the pre-emission snapshot to compare against
    /// - `source`: entropy source for random mutation decisions
    pub(super) fn post_process_emission(
        &mut self,
        mut snapshot: EmissionSnapshot,
        source: &mut GenerationSource,
    ) {
        if self.mutators.is_empty() {
            return;
        }

        // Calculate deltas
        // Stack can shrink (items popped), so only capture new items if stack grew
        if self.state.stack.len() > snapshot.stack_depth {
            snapshot.stack_delta = self.state.stack.inner[snapshot.stack_depth..].to_vec();
        }

        // Output always grows (or stays same)
        if self.output.len() >= snapshot.output_len {
            snapshot.output_delta = self.output[snapshot.output_len..].to_vec();
        }

        // Memo delta: find new indices (memo only grows)
        for idx in snapshot.memo_size..self.state.memo.len() {
            snapshot.memo_delta.push(idx);
        }

        // Let each mutator post-process
        for mutator in &self.mutators {
            mutator.post_process(&snapshot, &mut self.output, source, self.mutation_rate);
        }
    }
}
