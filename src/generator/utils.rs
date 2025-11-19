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

//! utility methods for stack and memo operations.
//!
//! this module provides helper methods for the `Generator` that simplify
//! common operations on the pickle virtual machine stack and memo table.
//! these methods are used throughout the generation process to query and
//! manipulate the simulated PVM state.

use super::Generator;
use crate::stack::{StackObject, StackObjectRef};

impl Generator {
    /// peek at the top of the stack without removing it.
    ///
    /// returns a reference to the top stack object, or `None` if the stack is empty.
    /// this is a non-destructive operation that doesn't modify the stack.
    pub(super) fn peek(&self) -> Option<&StackObjectRef> {
        self.state.stack.peek()
    }

    /// push a value onto the stack.
    ///
    /// adds a new object to the top of the simulated pickle virtual machine stack.
    /// this is used after emitting opcodes that produce stack values.
    pub(super) fn push(&mut self, value: StackObject) {
        self.state.stack.push(value);
    }

    /// pop a value from the stack.
    ///
    /// removes and returns the top stack object, or `None` if the stack is empty.
    /// this is used after emitting opcodes that consume stack values.
    pub(super) fn pop(&mut self) -> Option<StackObjectRef> {
        self.state.stack.pop()
    }

    /// get a value from the memo table.
    ///
    /// retrieves a previously memoized object by its index. returns `None` if
    /// the index doesn't exist in the memo table.
    pub(super) fn get(&self, index: usize) -> Option<&StackObjectRef> {
        self.state.memo.get(&index)
    }

    /// put a value into the memo table.
    ///
    /// stores an object in the memo table at the specified index. this allows
    /// the object to be referenced later by GET/BINGET opcodes.
    pub(super) fn put(&mut self, index: usize, value: StackObject) {
        self.state.memo.insert(index, StackObjectRef::new(value));
    }

    /// check if the stack contains any MARK objects.
    ///
    /// returns `true` if at least one MARK is present on the stack. MARKs are
    /// used to delimit groups of items for operations like building lists or tuples.
    pub(super) fn has_mark(&self) -> bool {
        self.state
            .stack
            .inner
            .iter()
            .any(|obj| matches!(*obj.borrow(), StackObject::Mark))
    }

    /// peek at a stack object at a specific depth from the top.
    ///
    /// depth 0 is the top of the stack, depth 1 is one below the top, etc.
    /// returns `None` if the depth exceeds the stack size.
    pub(super) fn peek_at(&self, depth: usize) -> Option<&StackObjectRef> {
        let len = self.state.stack.len();
        if depth < len {
            self.state.stack.inner.get(len - 1 - depth)
        } else {
            None
        }
    }

    /// check if the object at a given depth is a list.
    ///
    /// returns `true` if the object at the specified depth from the top is a
    /// `StackObject::List`, `false` otherwise or if the depth is invalid.
    pub(super) fn is_list_at(&self, depth: usize) -> bool {
        if let Some(obj_ref) = self.peek_at(depth) {
            matches!(*obj_ref.borrow(), StackObject::List(_))
        } else {
            false
        }
    }

    /// check if the object at a given depth is a dict.
    ///
    /// returns `true` if the object at the specified depth from the top is a
    /// `StackObject::Dict`, `false` otherwise or if the depth is invalid.
    pub(super) fn is_dict_at(&self, depth: usize) -> bool {
        if let Some(obj_ref) = self.peek_at(depth) {
            matches!(*obj_ref.borrow(), StackObject::Dict(_))
        } else {
            false
        }
    }

    /// check if the object at a given depth is callable.
    ///
    /// returns `true` if the object at the specified depth is either a
    /// `StackObject::Callable` or `StackObject::Global`, `false` otherwise.
    pub(super) fn is_callable_at(&self, depth: usize) -> bool {
        if let Some(obj_ref) = self.peek_at(depth) {
            matches!(
                *obj_ref.borrow(),
                StackObject::Callable(_) | StackObject::Global { .. }
            )
        } else {
            false
        }
    }

    /// check if the object at a given depth is a tuple.
    ///
    /// returns `true` if the object at the specified depth from the top is a
    /// `StackObject::Tuple`, `false` otherwise or if the depth is invalid.
    pub(super) fn is_tuple_at(&self, depth: usize) -> bool {
        if let Some(obj_ref) = self.peek_at(depth) {
            matches!(*obj_ref.borrow(), StackObject::Tuple(_))
        } else {
            false
        }
    }

    /// check if the object at a given depth is an instance.
    ///
    /// returns `true` if the object at the specified depth from the top is a
    /// `StackObject::Instance`, `false` otherwise or if the depth is invalid.
    pub(super) fn is_instance_at(&self, depth: usize) -> bool {
        if let Some(obj_ref) = self.peek_at(depth) {
            matches!(*obj_ref.borrow(), StackObject::Instance(_))
        } else {
            false
        }
    }

    /// check if there's a list immediately below the topmost MARK.
    ///
    /// searches from the top of the stack for the first MARK, then checks if the
    /// object immediately below it is a list. returns `false` if no MARK is found,
    /// if the MARK is at the bottom, or if the object below isn't a list.
    pub(super) fn is_list_at_mark(&self) -> bool {
        // check if there's a list below the topmost MARK
        for (idx, obj_ref) in self.state.stack.inner.iter().enumerate().rev() {
            if matches!(*obj_ref.borrow(), StackObject::Mark) {
                // found the mark, check if item below it is a list
                if idx > 0 {
                    if let Some(below_mark) = self.state.stack.inner.get(idx - 1) {
                        return matches!(*below_mark.borrow(), StackObject::List(_));
                    }
                }
                return false;
            }
        }
        false
    }

    /// check if there's a dict immediately below the topmost MARK.
    ///
    /// searches from the top of the stack for the first MARK, then checks if the
    /// object immediately below it is a dict. returns `false` if no MARK is found,
    /// if the MARK is at the bottom, or if the object below isn't a dict.
    pub(super) fn is_dict_at_mark(&self) -> bool {
        // check if there's a dict below the topmost MARK
        for (idx, obj_ref) in self.state.stack.inner.iter().enumerate().rev() {
            if matches!(*obj_ref.borrow(), StackObject::Mark) {
                // found the mark, check if item below it is a dict
                if idx > 0 {
                    if let Some(below_mark) = self.state.stack.inner.get(idx - 1) {
                        return matches!(*below_mark.borrow(), StackObject::Dict(_));
                    }
                }
                return false;
            }
        }
        false
    }

    /// count the number of items from the top of the stack to the topmost MARK.
    ///
    /// returns the count of items above (but not including) the first MARK found
    /// when searching from the top. returns `None` if no MARK is found on the stack.
    /// this is used to determine how many items will be consumed by MARK-based opcodes.
    pub(super) fn count_items_to_mark(&self) -> Option<usize> {
        // count items from TOS back to (but not including) the topmost MARK
        for (count, obj_ref) in self.state.stack.inner.iter().rev().enumerate() {
            if matches!(*obj_ref.borrow(), StackObject::Mark) {
                return Some(count);
            }
        }
        None // no MARK found
    }

    /// check if the object at a given depth is a string.
    ///
    /// returns `true` if the object at the specified depth from the top is a
    /// `StackObject::String`, `false` otherwise or if the depth is invalid.
    pub(super) fn is_string_at(&self, depth: usize) -> bool {
        if let Some(obj_ref) = self.peek_at(depth) {
            matches!(*obj_ref.borrow(), StackObject::String(_))
        } else {
            false
        }
    }

    /// check if there's a set immediately below the topmost MARK.
    ///
    /// searches from the top of the stack for the first MARK, then checks if the
    /// object immediately below it is a set. returns `false` if no MARK is found,
    /// if the MARK is at the bottom, or if the object below isn't a set.
    pub(super) fn is_set_at_mark(&self) -> bool {
        // check if there's a set below the topmost MARK
        for (idx, obj_ref) in self.state.stack.inner.iter().enumerate().rev() {
            if matches!(*obj_ref.borrow(), StackObject::Mark) {
                // found the mark, check if item below it is a set
                if idx > 0 {
                    if let Some(below_mark) = self.state.stack.inner.get(idx - 1) {
                        return matches!(*below_mark.borrow(), StackObject::Set(_));
                    }
                }
                return false;
            }
        }
        false
    }

    /// check if there's a callable immediately above the topmost MARK.
    ///
    /// searches from the top of the stack for the first MARK, then checks if the
    /// object immediately above it (closer to TOS) is callable. returns `false` if
    /// no MARK is found, if the MARK is at the top, or if the object above isn't callable.
    /// this is used to validate BUILD/INST opcode preconditions.
    pub(super) fn is_callable_above_mark(&self) -> bool {
        // check if there's a callable immediately above the topmost MARK
        for (idx, obj_ref) in self.state.stack.inner.iter().enumerate().rev() {
            if matches!(*obj_ref.borrow(), StackObject::Mark) {
                let above_idx = idx + 1;
                if above_idx < self.state.stack.inner.len() {
                    if let Some(above_mark) = self.state.stack.inner.get(above_idx) {
                        return matches!(
                            *above_mark.borrow(),
                            StackObject::Callable(_) | StackObject::Global { .. }
                        );
                    }
                }
                return false;
            }
        }
        false
    }
}
