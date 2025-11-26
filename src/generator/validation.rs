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

//! opcode validation and selection logic.
//!
//! this module implements the core validation logic that determines which opcodes
//! can be safely emitted given the current state of the pickle virtual machine.
//! it checks stack depth, type constraints, protocol version compatibility, and
//! other preconditions required for valid pickle generation.
//!
//! the validation system ensures that generated pickles are structurally valid
//! according to the pickle protocol specification, preventing invalid opcode
//! sequences that would cause unpickling to fail.

use super::source::{EntropySource, GenerationSource};
use super::Generator;
use crate::opcodes::{OpcodeKind, PICKLE_OPCODES};
use crate::stack::StackObject;

impl Generator {
    /// get all opcodes that are valid for the current protocol version and state.
    ///
    /// filters the complete set of opcodes available in the current protocol version
    /// to only those that can be safely emitted given the current stack and memo state.
    /// this is the primary entry point for opcode selection during generation.
    ///
    /// returns a vector of `OpcodeKind` values that pass the `can_emit()` validation.
    pub(super) fn get_valid_opcodes(&self) -> Vec<OpcodeKind> {
        let version = self.state.version as u8;
        let Some(all_opcodes) = PICKLE_OPCODES.get(&version) else {
            return vec![];
        };

        all_opcodes
            .iter()
            .filter(|&&op| self.can_emit(op))
            .copied()
            .collect()
    }

    /// select an opcode from a list using weighted random selection.
    ///
    /// currently implements uniform random selection from the provided opcodes.
    /// returns `OpcodeKind::None` as a safe fallback if the input list is empty.
    ///
    /// # Parameters
    /// - `opcodes`: list of valid opcodes to choose from
    /// - `source`: entropy source for random selection
    ///
    /// # Future Work
    /// TODO: implement weighted selection to favor interesting opcodes based on
    /// current state (e.g., favor value-producing opcodes when stack is empty).
    pub(super) fn weighted_choice(
        &self,
        opcodes: Vec<OpcodeKind>,
        source: &mut GenerationSource,
    ) -> OpcodeKind {
        if opcodes.is_empty() {
            // fallback to a safe opcode that's always valid
            return OpcodeKind::None;
        }

        // uniform random selection
        let idx = source.choose_index(opcodes.len());
        opcodes[idx]
    }

    /// check if a specific opcode can be safely emitted in the current state.
    ///
    /// validates all preconditions for the given opcode, including:
    /// - stack depth requirements
    /// - type constraints on stack objects
    /// - protocol version compatibility
    /// - memo table state
    /// - MARK presence and positioning
    ///
    /// this is the core validation logic that ensures generated pickles are
    /// structurally valid according to the pickle protocol specification.
    ///
    /// # Returns
    /// `true` if the opcode can be safely emitted, `false` otherwise.
    pub(super) fn can_emit(&self, opcode: OpcodeKind) -> bool {
        use OpcodeKind::*;

        match opcode {
            // stack manipulation - need items on stack
            Pop => self.state.stack.len() >= 1,
            Dup => {
                // DUP requires at least 1 item, and TOS can't be a MARK
                // duplicating MARKs creates invalid pickle state that causes
                // various MARK-reliant opcodes to crash
                if self.state.stack.len() < 1 {
                    return false;
                }

                if let Some(top) = self.peek() {
                    !matches!(*top.borrow(), StackObject::Mark)
                } else {
                    false
                }
            }

            // list operations
            Append => self.state.stack.len() >= 2 && self.is_list_at(1),
            Appends => {
                self.has_mark()
                    && self.is_list_at_mark()
                    && self.count_items_to_mark().is_some_and(|count| count > 0)
            }

            // dict operations
            SetItem => self.state.stack.len() >= 3 && self.is_dict_at(2),
            SetItems => {
                // need: MARK, dict below mark, and even number of items (key-value pairs)
                self.has_mark()
                    && self.is_dict_at_mark()
                    && self
                        .count_items_to_mark()
                        .is_some_and(|count| count > 0 && count % 2 == 0)
            }

            // set operations - ADDITEMS needs a set below the MARK and items between MARK and TOS
            AddItems => {
                self.has_mark()
                    && self.is_set_at_mark()
                    && self.count_items_to_mark().is_some_and(|count| count > 0)
            }

            // MARK-consuming operations
            Tuple | List | FrozenSet => self.has_mark(),
            Dict => {
                // dict requires MARK and even number of items for key-value pairs)
                self.has_mark()
                    && self
                        .count_items_to_mark()
                        .is_some_and(|count| count > 0 && count % 2 == 0)
            }
            PopMark => self.has_mark(),

            // tuple shortcuts (no MARK needed)
            Tuple1 => self.state.stack.len() >= 1,
            Tuple2 => self.state.stack.len() >= 2,
            Tuple3 => self.state.stack.len() >= 3,

            // object construction - need proper types on stack
            // REDUCE: stack layout is [... callable args] where args is TOS
            // pops args, then callable -> creates instance
            // args must be a tuple (not None, int, or other non-iterable)
            Reduce => self.state.stack.len() >= 2 && self.is_callable_at(1) && self.is_tuple_at(0),
            // NEWOBJ: stack layout is [... class args] where args (tuple) is TOS
            NewObj => self.state.stack.len() >= 2 && self.is_callable_at(1) && self.is_tuple_at(0),
            // NEWOBJ_EX: stack layout is [... class args kwargs] where kwargs is TOS
            NewObjEx => {
                self.state.stack.len() >= 3
                    && self.is_callable_at(2)
                    && self.is_tuple_at(1)
                    && self.is_dict_at(0)
            }
            // BUILD: stack layout is [... instance state] where state is TOS
            // constrain state to typical types (tuple/dict) to avoid invalid shapes
            Build => {
                self.state.stack.len() >= 2
                    && self.is_instance_at(1)
                    && (self.is_tuple_at(0) || self.is_dict_at(0))
            }
            Inst => self.has_mark() && self.count_items_to_mark().is_some_and(|count| count > 0),
            Obj => self.has_mark() && self.is_callable_above_mark(),

            // memo operations - GET requires existing memo entry
            Get | BinGet | LongBinGet => !self.state.memo.is_empty(),

            // PUT operations - need something to memoize (and not MARK)
            Put | BinPut | LongBinPut | Memoize => {
                self.state.stack.len() >= 1
                    && self
                        .peek()
                        .is_some_and(|obj| !matches!(*obj.borrow(), StackObject::Mark))
            }

            // STACK_GLOBAL needs 2 strings on stack (module at depth 1, name at depth 0)
            // in unsafe mode, allow any 2 values (type confusion will replace them)
            StackGlobal => {
                if self.unsafe_mutations {
                    self.state.stack.len() >= 2
                } else {
                    self.state.stack.len() >= 2 && self.is_string_at(0) && self.is_string_at(1)
                }
            }

            // BinPersID needs 1 item on stack
            BinPersID => self.state.stack.len() >= 1,

            // proto should only be emitted once at the start
            Proto => !self.state.proto_emitted,

            // stop should never be emitted during generation - only at the very end
            Stop => false,

            // value-producing opcodes - always valid
            None | NewTrue | NewFalse | Int | Long | Long1 | Long4 | BinInt | BinInt1 | BinInt2
            | Float | BinFloat | String | BinString | ShortBinString | Unicode
            | ShortBinUnicode | BinUnicode | BinUnicode8 | ShortBinBytes | BinBytes | BinBytes8
            | ByteArray8 | EmptyList | EmptyDict | EmptyTuple | EmptySet | Global | PersID
            | Mark => true,

            // EXT* opcodes require a configured extension registry
            // allow only if explicitly enabled via with_ext_opcodes()
            Ext1 | Ext2 | Ext4 => self.allow_ext_opcodes,
            
            // NextBuffer/ReadOnlyBuffer require out-of-band buffer support
            // allow only if explicitly enabled via with_buffer_opcodes()
            NextBuffer | ReadOnlyBuffer => self.allow_buffer_opcodes,

            // frame: handled specially after generation is complete
            Frame => false, // don't emit during generation, will be inserted at the end if needed
        }
    }
}
