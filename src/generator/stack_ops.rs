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

//! stack operation processing for pickle opcodes.
//!
//! this module implements the core logic for simulating the pickle virtual machine
//! stack as opcodes are emitted. it handles all stack effects for every opcode,
//! including:
//!
//! - stack manipulation (push, pop, dup, mark)
//! - container construction (list, tuple, dict, set, frozenset)
//! - container modification (append, setitem, additems)
//! - object instantiation (reduce, build, inst, obj, newobj)
//! - memoization (put, get, memoize)
//! - value parsing (int, float, string, bytes)
//!
//! the simulation ensures that the generator maintains an accurate representation
//! of the PVM state, which is critical for validating whether subsequent opcodes
//! can be safely emitted.

use super::Generator;
use crate::opcodes::OpcodeKind;
use crate::protocol::Version;
use crate::stack::{InstanceObject, StackObject, StackObjectRef};
use std::collections::{HashMap, HashSet};

impl Generator {
    /// clean up the stack to prepare for the STOP opcode.
    ///
    /// the STOP opcode requires exactly one item on the stack. this method
    /// intelligently combines or removes stack items until only one remains,
    /// using the following strategies:
    ///
    /// - if stack is empty: push a None value
    /// - if stack has one MARK: pop it and push None
    /// - if stack has multiple items with MARK: combine items above MARK into tuple
    /// - if stack has multiple items without MARK: combine top items into tuples
    ///
    /// this method is called at the end of generation to ensure the pickle is
    /// valid before emitting the final STOP opcode.
    pub(super) fn cleanup_for_stop(&mut self) {
        use OpcodeKind::*;

        // remove any MARKs by using TUPLE
        // TUPLE pops items until it finds a MARK, pops the MARK, and pushes a tuple
        // note: DUP no longer duplicates MARKs, so each MARK corresponds to a real
        // MARK byte in the pickle
        while self.has_mark() {
            self.emit_opcode(Tuple);
        }

        // at this point, stack has no MARKs, just regular items
        // keep combining until we have exactly 1 item
        // use TUPLE2/TUPLE3 which don't require MARKs
        let mut safety_counter = 0;
        while self.state.stack.len() > 1 && safety_counter < 10000 {
            safety_counter += 1;
            
            let stack_len = self.state.stack.len();
            if stack_len >= 3 {
                self.emit_opcode(Tuple3);
            } else if stack_len == 2 {
                self.emit_opcode(Tuple2);
            } else if stack_len == 1 {
                // exactly 1 item, we're done
                break;
            } else {
                // stack is empty - shouldn't happen but break to be safe
                break;
            }
        }

        // handle edge case: stack is empty
        if self.state.stack.len() == 0 {
            self.emit_opcode(None);
        }

        // final check
        if let Some(top) = self.peek() {
            if matches!(*top.borrow(), StackObject::Mark) {
                // should never happen after our cleanup, but handle it anyway
                self.pop();
                if self.state.stack.len() == 0 {
                    self.emit_opcode(None);
                }
            }
        }
    }

    /// process the stack effects of an emitted opcode.
    ///
    /// this is the core method that simulates the pickle virtual machine's stack
    /// operations. after an opcode is emitted to the output buffer, this method
    /// updates the simulated stack to reflect the opcode's effects.
    ///
    /// the simulation handles:
    /// - **stack manipulation**: Pop, Dup, Mark, PopMark
    /// - **containers**: List, Tuple, Dict, Set, FrozenSet and their empty variants
    /// - **container modification**: Append, Appends, SetItem, SetItems, AddItems
    /// - **values**: Int, Long, Float, String, Bytes, Bool, None
    /// - **objects**: Global, StackGlobal, Reduce, Build, Inst, Obj, NewObj, NewObjEx
    /// - **memoization**: Put, BinPut, LongBinPut, Get, BinGet, LongBinGet, Memoize
    /// - **persistence**: PersID, BinPersID
    /// - **extensions**: Ext1, Ext2, Ext4
    ///
    /// # Parameters
    /// - `opcode`: the opcode kind that was emitted
    /// - `arg_bytes`: the raw argument bytes for opcodes that take arguments
    ///
    /// # Implementation Notes
    /// - uses `Rc<RefCell<>>` for shared ownership of stack objects
    /// - handles circular references safely via pointer-based equality
    /// - parses argument bytes according to each opcode's format
    /// - maintains type information for validation of subsequent opcodes
    pub(super) fn process_stack_ops(&mut self, opcode: OpcodeKind, arg_bytes: Option<&[u8]>) {
        use OpcodeKind::*;
        
        match opcode {
            Pop => {
                self.pop();
            }
            Dup => {
                if let Some(top) = self.peek() {
                    // IMPORTANT: don't duplicate a MARK!
                    // duplicating MARKs creates invalid pickle state that causes
                    // TUPLE to fail (it tries to pop until MARK, but if stack is
                    // all MARKs, it crashes with "list index out of range")
                    if !matches!(*top.borrow(), StackObject::Mark) {
                        self.state.stack.inner.push(top.clone());
                    }
                }
            }
            Mark => {
                self.push(StackObject::Mark);
            }
            PopMark => {
                while let Some(item) = self.pop() {
                    if matches!(*item.borrow(), StackObject::Mark) {
                        break;
                    }
                }
            }
            EmptyList => self.push(StackObject::List(Vec::new())),
            Append => {
                if self.state.stack.len() < 2 {
                    return;
                }
                if let Some(item) = self.pop() {
                    if let Some(cell) = self.peek() {
                        // check if it's a list first without holding the borrow
                        let is_list = matches!(*cell.borrow(), StackObject::List(_));
                        if is_list {
                            // now mutably borrow to append
                            if let StackObject::List(ref mut list) = *cell.borrow_mut() {
                                list.push(item);
                            }
                        }
                    }
                }
            }
            Appends => {
                let mut items_to_append = Vec::new();
                while let Some(item) = self.pop() {
                    if matches!(*item.borrow(), StackObject::Mark) {
                        break;
                    }
                    items_to_append.push(item);
                }
                items_to_append.reverse();

                if let Some(list_obj) = self.peek() {
                    if let StackObject::List(ref mut list) = *list_obj.borrow_mut() {
                        list.extend(items_to_append);
                    }
                }
            }
            List => {
                let mut accumulated = Vec::new();
                while let Some(item) = self.pop() {
                    match *item.borrow() {
                        StackObject::Mark => break,
                        _ => accumulated.push(item.clone()),
                    }
                }
                accumulated.reverse();
                self.push(StackObject::List(accumulated));
            }
            EmptyTuple => {
                self.push(StackObject::Tuple(Vec::new()));
            }
            Tuple => {
                let mut accumulated = Vec::new();
                while let Some(item) = self.pop() {
                    match *item.borrow() {
                        StackObject::Mark => break,
                        _ => accumulated.push(item.clone()),
                    }
                }
                accumulated.reverse();
                self.push(StackObject::Tuple(accumulated));
            }
            Tuple1 => {
                if let Some(item) = self.pop() {
                    self.push(StackObject::Tuple(vec![item]));
                }
            }
            Tuple2 => {
                if self.state.stack.len() >= 2 {
                    // unwraps are guarded with the length check, fine to leave as-is
                    let second = self.pop().unwrap();
                    let first = self.pop().unwrap();
                    self.push(StackObject::Tuple(vec![first, second]));
                }
            }
            Tuple3 => {
                if self.state.stack.len() >= 3 {
                    let third = self.pop().unwrap();
                    let second = self.pop().unwrap();
                    let first = self.pop().unwrap();
                    self.push(StackObject::Tuple(vec![first, second, third]));
                }
            }
            EmptyDict => {
                self.push(StackObject::Dict(HashMap::new()));
            }
            Dict => {
                // allow interior mutability in hash keys - our Hash/Eq implementations
                // are value-based and we never mutate objects used as dict keys
                #[allow(clippy::mutable_key_type)]
                let mut accumulated = HashMap::new();

                while let Some(value) = self.pop() {
                    match *value.borrow() {
                        StackObject::Mark => break,
                        _ => {
                            if let Some(key) = self.pop() {
                                accumulated.insert(key, value.clone());
                            }
                        }
                    }
                }
                self.push(StackObject::Dict(accumulated));
            }
            SetItem => {
                if self.state.stack.len() < 3 {
                    return;
                }

                if let Some(value) = self.pop() {
                    if let Some(key) = self.pop() {
                        if let Some(cell) = self.peek() {
                            // Check if it's a dict first without holding the borrow
                            let is_dict = matches!(*cell.borrow(), StackObject::Dict(_));
                            if is_dict {
                                // now mutably borrow to insert
                                if let StackObject::Dict(ref mut dict) = *cell.borrow_mut() {
                                    dict.insert(key, value);
                                }
                            }
                        }
                    }
                }
            }
            SetItems => {
                let mut accumulated = Vec::new();
                while let Some(value) = self.pop() {
                    let is_mark = matches!(*value.borrow(), StackObject::Mark);
                    if is_mark {
                        break;
                    }
                    if let Some(key) = self.pop() {
                        accumulated.push((key, value));
                    }
                }
                if let Some(cell) = self.peek() {
                    // check if it's a dict first without holding the borrow
                    let is_dict = matches!(*cell.borrow(), StackObject::Dict(_));
                    if is_dict {
                        // now mutably borrow to insert all items
                        if let StackObject::Dict(ref mut dict) = *cell.borrow_mut() {
                            for (key, value) in accumulated {
                                dict.insert(key, value);
                            }
                        }
                    }
                }
            }
            EmptySet => {
                self.push(StackObject::Set(HashSet::new()));
            }
            AddItems => {
                let mut accumulated = Vec::new();
                while let Some(item) = self.pop() {
                    match *item.borrow() {
                        StackObject::Mark => break,
                        _ => accumulated.push(item.clone()),
                    }
                }
                if let Some(cell) = self.peek() {
                    accumulated.reverse();
                    // check if it's a set first without holding the borrow
                    let is_set = matches!(*cell.borrow(), StackObject::Set(_));
                    if is_set {
                        // now mutably borrow to insert all items
                        if let StackObject::Set(ref mut set) = *cell.borrow_mut() {
                            for item in accumulated {
                                set.insert(item);
                            }
                        }
                    }
                }
            }
            FrozenSet => {
                // allow interior mutability in hash keys - our Hash/Eq implementations
                // are value-based and we never mutate objects used as set members
                #[allow(clippy::mutable_key_type)]
                let mut accumulated = HashSet::new();

                while let Some(item) = self.pop() {
                    match *item.borrow() {
                        StackObject::Mark => break,
                        _ => {
                            accumulated.insert(item.clone());
                        }
                    }
                }

                self.push(StackObject::FrozenSet(accumulated));
            }
            Int => {
                // always push, even if parsing fails
                let value = if let Some(arg_bytes) = arg_bytes {
                    if let Ok(value_str) = std::str::from_utf8(arg_bytes) {
                        value_str.trim().parse::<i64>().unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };
                // in protocol 0-1, INT opcode with 00/01 represents booleans
                // protocol 2+ has dedicated NEWTRUE/NEWFALSE opcodes
                if matches!(self.state.version, Version::V0 | Version::V1)
                    && (value == 0 || value == 1)
                {
                    self.push(StackObject::Bool(value == 1));
                } else {
                    self.push(StackObject::Int(value));
                }
            }
            BinInt => {
                if let Some(arg_bytes) = arg_bytes {
                    if arg_bytes.len() < 4 {
                        return; // malformed opcode, need 4 bytes
                    }
                    let val = i32::from_le_bytes([
                        arg_bytes[0],
                        arg_bytes[1],
                        arg_bytes[2],
                        arg_bytes[3],
                    ]);
                    self.push(StackObject::Int(val as i64));
                }
            }
            BinInt1 => {
                if let Some(arg_bytes) = arg_bytes {
                    if arg_bytes.is_empty() {
                        return; // malformed opcode, need 1 byte
                    }
                    let val = arg_bytes[0];
                    self.push(StackObject::Int(val as i64));
                }
            }
            BinInt2 => {
                if let Some(arg_bytes) = arg_bytes {
                    if arg_bytes.len() < 2 {
                        return; // malformed opcode, need 2 bytes
                    }
                    let val = u16::from_le_bytes([arg_bytes[0], arg_bytes[1]]);
                    self.push(StackObject::Int(val as i64));
                }
            }
            Long => {
                // always push, even if parsing fails
                let value = if let Some(arg_bytes) = arg_bytes {
                    if let Ok(value_str) = std::str::from_utf8(arg_bytes) {
                        // strip trailing 'L\n'
                        let value_str = value_str.trim_end_matches('\n').trim_end_matches('L');
                        value_str.trim().parse::<i64>().unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };
                self.push(StackObject::Int(value));
            }
            Long1 => {
                if let Some(arg_bytes) = arg_bytes {
                    if arg_bytes.is_empty() {
                        return; // malformed opcode, no size byte
                    }
                    let size = arg_bytes[0] as usize;
                    if arg_bytes.len() > size {
                        let int_bytes = &arg_bytes[1..1 + size];
                        // interpret as little-endian integer without static size
                        // limit to i64 size (8 bytes) to prevent overflow
                        let mut value: i64 = 0;
                        for (i, &b) in int_bytes.iter().enumerate().take(8) {
                            // safe: i < 8, so i*8 < 64, shift is always valid
                            value |= (b as i64) << (i * 8);
                        }
                        self.push(StackObject::Int(value));
                    }
                }
            }
            Long4 => {
                if let Some(arg_bytes) = arg_bytes {
                    if arg_bytes.len() >= 4 {
                        let size = u32::from_le_bytes([
                            arg_bytes[0],
                            arg_bytes[1],
                            arg_bytes[2],
                            arg_bytes[3],
                        ]) as usize;
                        if arg_bytes.len() >= 4 + size {
                            let int_bytes = &arg_bytes[4..4 + size];
                            // interpret as little-endian integer without static size
                            // limit to i64 size (8 bytes) to prevent overflow
                            let mut value: i64 = 0;
                            for (i, &b) in int_bytes.iter().enumerate().take(8) {
                                // safe: i < 8, so i*8 < 64, shift is always valid
                                value |= (b as i64) << (i * 8);
                            }
                            self.push(StackObject::Int(value));
                        }
                    }
                }
            }
            String | ShortBinUnicode | Unicode | BinUnicode | BinUnicode8 => {
                // always push a string, even if arg_bytes is None
                let value = if let Some(arg_bytes) = arg_bytes {
                    std::string::String::from_utf8_lossy(arg_bytes).into_owned()
                } else {
                    std::string::String::new()
                };
                self.push(StackObject::String(value));
            }
            BinString | ShortBinString | BinBytes | ShortBinBytes | BinBytes8 => {
                // always push bytes, even if arg_bytes is None
                let bytes = if let Some(arg_bytes) = arg_bytes {
                    arg_bytes.to_vec()
                } else {
                    Vec::new()
                };
                self.push(StackObject::Bytes(bytes));
            }
            ByteArray8 => {
                // always push bytearray, even if arg_bytes is None
                let bytes = if let Some(arg_bytes) = arg_bytes {
                    arg_bytes.to_vec()
                } else {
                    Vec::new()
                };
                self.push(StackObject::ByteArray(bytes));
            }
            None => {
                self.push(StackObject::None);
            }
            NewTrue => {
                self.push(StackObject::Bool(true));
            }
            NewFalse => {
                self.push(StackObject::Bool(false));
            }
            Float => {
                // always push, even if parsing fails
                let value = if let Some(arg_bytes) = arg_bytes {
                    if let Ok(value_str) = std::str::from_utf8(arg_bytes) {
                        value_str.trim().parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                self.push(StackObject::Float(value));
            }
            BinFloat => {
                if let Some(arg_bytes) = arg_bytes {
                    if arg_bytes.len() < 8 {
                        return; // malformed opcode, need 8 bytes
                    }
                    let val = f64::from_be_bytes([
                        arg_bytes[0],
                        arg_bytes[1],
                        arg_bytes[2],
                        arg_bytes[3],
                        arg_bytes[4],
                        arg_bytes[5],
                        arg_bytes[6],
                        arg_bytes[7],
                    ]);
                    self.push(StackObject::Float(val));
                }
            }
            Global => {
                if let Some(arg_bytes) = arg_bytes {
                    // arg_bytes contains "module\nclass\n" - two newline-terminated strings
                    let full_string = std::string::String::from_utf8_lossy(arg_bytes);
                    let parts: Vec<&str> = full_string.split('\n').collect();

                    if parts.len() >= 2 {
                        let module = parts[0].to_string();
                        let class = parts[1].to_string();

                        let global = StackObjectRef::new(StackObject::Global {
                            module,
                            name: class,
                        });
                        // wrap global in Callable since it can be invoked by REDUCE
                        self.push(StackObject::Callable(global));
                    }
                }
            }
            StackGlobal => {
                if let Some(attr) = self.pop() {
                    if let Some(module) = self.pop() {
                        if let (StackObject::String(module_str), StackObject::String(name_str)) =
                            (&*module.borrow(), &*attr.borrow())
                        {
                            let global = StackObjectRef::new(StackObject::Global {
                                module: module_str.clone(),
                                name: name_str.clone(),
                            });
                            // wrap global in Callable since it can be invoked by REDUCE
                            self.push(StackObject::Callable(global));
                        }
                    }
                }
            }
            Reduce => {
                if self.state.stack.len() < 2 {
                    return;
                }

                // pops args (usually tuple) and callable from stack
                if let (Some(args), Some(callable)) = (self.pop(), self.pop()) {
                    // unwrap Callable to get the actual callable object (usually Global)
                    let inner_callable = if let StackObject::Callable(inner) = &*callable.borrow() {
                        inner.clone()
                    } else {
                        // if not wrapped in Callable, use as-is
                        callable.clone()
                    };

                    self.push(StackObject::Instance(InstanceObject {
                        callable: inner_callable,
                        args,
                    }));
                }
            }
            Build => {
                if self.state.stack.len() < 2 {
                    return;
                }

                // pops state and instance, updates instance's args
                if let (Some(state), Some(instance_ref)) = (self.pop(), self.pop()) {
                    if let StackObject::Instance(ref mut inst) = *instance_ref.borrow_mut() {
                        inst.args = state;
                    }
                    self.push(instance_ref.borrow().clone());
                }
            }
            Inst => {
                if let Some(arg_bytes) = arg_bytes {
                    // arg_bytes contains "module\nclass\n" - two newline-terminated strings
                    let full_string = std::string::String::from_utf8_lossy(arg_bytes);
                    let parts: Vec<&str> = full_string.split('\n').collect();

                    if parts.len() >= 2 {
                        let module = parts[0].to_string();
                        let class = parts[1].to_string();

                        let global = StackObjectRef::new(StackObject::Global {
                            module: module.clone(),
                            name: class.clone(),
                        });

                        let mut accumulated = Vec::new();

                        while let Some(item) = self.pop() {
                            match *item.borrow() {
                                StackObject::Mark => break,
                                _ => accumulated.push(item.clone()),
                            }
                        }

                        accumulated.reverse();

                        let args = StackObjectRef::new(StackObject::Tuple(accumulated));

                        self.push(StackObject::Instance(InstanceObject {
                            callable: global,
                            args,
                        }));
                    }
                }
            }
            Obj => {
                // build a class instance (protocol 1)
                // pops items from TOS back to MARK: first item after MARK is class, rest are args
                // stack before: MARK class arg1 arg2 ... (arg2 on TOS)
                let mut accumulated = Vec::new();

                while let Some(item) = self.pop() {
                    match *item.borrow() {
                        StackObject::Mark => break,
                        _ => accumulated.push(item.clone()),
                    }
                }

                // accumulated is now [arg2, arg1, class] - class is LAST (closest to mark)
                if !accumulated.is_empty() {
                    accumulated.reverse(); // now [class, arg1, arg2]
                    let class_obj = accumulated.remove(0); // extract class
                    let args = StackObjectRef::new(StackObject::Tuple(accumulated)); // rest are args

                    self.push(StackObject::Instance(InstanceObject {
                        callable: class_obj,
                        args,
                    }));
                }
            }
            NewObj => {
                // build an object instance (protocol 2)
                // pops args tuple and class, calls cls.__new__(cls, *args)
                if let (Some(args), Some(callable)) = (self.pop(), self.pop()) {
                    // unwrap Callable to get the actual callable object (usually Global)
                    let inner_callable = if let StackObject::Callable(inner) = &*callable.borrow() {
                        inner.clone()
                    } else {
                        // if not wrapped in Callable, use as-is (backwards compatibility)
                        callable.clone()
                    };

                    self.push(StackObject::Instance(InstanceObject {
                        callable: inner_callable.clone(),
                        args: args.clone(),
                    }));
                }
            }
            NewObjEx => {
                // build an object instance (protocol 4)
                // pops kwargs dict, args tuple, and class, calls cls.__new__(cls, *args, **kwargs)
                if let (Some(_kwargs), Some(args), Some(callable)) =
                    (self.pop(), self.pop(), self.pop())
                {
                    // unwrap Callable to get the actual callable object (usually Global)
                    let inner_callable = if let StackObject::Callable(inner) = &*callable.borrow() {
                        inner.clone()
                    } else {
                        // if not wrapped in Callable, use as-is (backwards compatibility)
                        callable.clone()
                    };

                    self.push(StackObject::Instance(InstanceObject {
                        callable: inner_callable,
                        args,
                    }));
                }
            }
            PersID => {
                // push an object identified by a persistent id
                // the argument is a newline-terminated string which is the persistent id
                if let Some(arg_bytes) = arg_bytes {
                    let persistent_id = std::string::String::from_utf8_lossy(arg_bytes);
                    // store as a string for now - persistent ids are application-specific
                    self.push(StackObject::String(persistent_id.into_owned()));
                }
            }
            BinPersID => {
                // push an object identified by a persistent id
                // the persistent id is popped from the stack
                if let Some(_pid) = self.pop() {
                    // in a real implementation, this would call persistent_load()
                    // We must push a safe placeholder that won't cause "X is not iterable" errors
                    // if later used by STACK_GLOBAL or other opcodes expecting strings.
                    // Push a string representation of the persistent ID.
                    self.push(StackObject::String("persistent_object".to_string()));
                }
            }
            Get => {
                if let Some(arg_bytes) = arg_bytes {
                    if let Ok(index_str) = std::str::from_utf8(arg_bytes) {
                        if let Ok(index) = index_str.trim().parse() {
                            if let Some(obj) = self.get(index) {
                                let cloned = obj.borrow().clone();
                                self.push(cloned);
                            }
                        }
                    }
                }
            }
            BinGet => {
                if let Some(arg_bytes) = arg_bytes {
                    let index = arg_bytes[0] as usize;
                    if let Some(obj) = self.get(index) {
                        let cloned = obj.borrow().clone();
                        self.push(cloned);
                    }
                }
            }
            LongBinGet => {
                if let Some(arg_bytes) = arg_bytes {
                    let index = u32::from_le_bytes([
                        arg_bytes[0],
                        arg_bytes[1],
                        arg_bytes[2],
                        arg_bytes[3],
                    ]) as usize;
                    if let Some(obj) = self.get(index) {
                        let cloned = obj.borrow().clone();
                        self.push(cloned);
                    }
                }
            }
            Put => {
                // PUT doesn't pop - it just peeks at TOS and stores in memo
                if let Some(arg_bytes) = arg_bytes {
                    if let Ok(index_str) = std::str::from_utf8(arg_bytes) {
                        if let Ok(index) = index_str.trim().parse() {
                            if let Some(top) = self.peek() {
                                let obj = top.borrow().clone();
                                if !matches!(obj, StackObject::Mark) {
                                    self.put(index, obj);
                                }
                            }
                        }
                    }
                }
            }
            BinPut => {
                // BINPUT doesn't pop - it just peeks at TOS and stores in memo
                if let Some(arg_bytes) = arg_bytes {
                    let index = arg_bytes[0] as usize;
                    if let Some(top) = self.peek() {
                        let obj = top.borrow().clone();
                        if !matches!(obj, StackObject::Mark) {
                            self.put(index, obj);
                        }
                    }
                }
            }
            LongBinPut => {
                // LONG_BINPUT doesn't pop - it just peeks at TOS and stores in memo
                if let Some(arg_bytes) = arg_bytes {
                    let index = u32::from_le_bytes([
                        arg_bytes[0],
                        arg_bytes[1],
                        arg_bytes[2],
                        arg_bytes[3],
                    ]) as usize;
                    if let Some(top) = self.peek() {
                        let obj = top.borrow().clone();
                        if !matches!(obj, StackObject::Mark) {
                            self.put(index, obj);
                        }
                    }
                }
            }
            Memoize => {
                if let Some(top) = self.pop() {
                    self.put(self.state.memo.len(), top.borrow().clone());
                    self.push(top.borrow().clone())
                }
            }
            Ext1 | Ext2 | Ext4 => {
                // EXT* opcodes look up objects in an extension registry.
                // Since we can't do real lookups, push a safe placeholder callable.
                // This prevents "int is not iterable" errors when REDUCE uses it.
                let placeholder = StackObjectRef::new(StackObject::Global {
                    module: "builtins".to_string(),
                    name: "object".to_string(),
                });
                self.push(StackObject::Callable(placeholder));
            }

            NextBuffer => {
                // NEXT_BUFFER pushes a buffer object to the stack
                self.push(StackObject::Bytes(Vec::new())); // Use empty bytes as placeholder
            }
            Proto | ReadOnlyBuffer | Stop | Frame => {
                // these opcodes don't manipulate the stack, but we're being
                // explicit about it so that we know we've covered all opcodes
            }
        }
        
        // uncomment for debugging:
        // let after = self.state.stack.len();
        // let delta = after as i32 - before as i32;
        // eprintln!("  @{:4} {:20} {} -> {} (Δ{:+})", 
        //     pos, format!("{:?}", opcode), before, after, delta);
    }
}
