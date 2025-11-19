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

//! opcode emission logic for pickle generation.
//!
//! this module implements the core emission logic that writes pickle opcodes and
//! their arguments to the output buffer. it handles all the format-specific details
//! for each opcode, including:
//!
//! - argument encoding (integers, floats, strings, bytes)
//! - length prefixes and terminators
//! - protocol version-specific formatting
//! - memo index management
//! - mutation application
//!
//! # Emission Flow
//!
//! 1. **snapshot**: capture pre-emission state for mutation tracking
//! 2. **emit**: write opcode byte and arguments to output buffer
//! 3. **process**: update simulated stack to reflect opcode effects
//! 4. **mutate**: apply post-emission mutations to the raw bytecode
//!
//! # Opcode Categories
//!
//! - **values**: Int, Long, Float, String, Bytes, None, Bool
//! - **containers**: List, Tuple, Dict, Set, FrozenSet
//! - **stack ops**: Mark, Pop, Dup, PopMark
//! - **memo ops**: Put, BinPut, LongBinPut, Get, BinGet, LongBinGet, Memoize
//! - **objects**: Global, StackGlobal, Reduce, Build, Inst, Obj, NewObj, NewObjEx
//! - **extensions**: Ext1, Ext2, Ext4, PersID, BinPersID
//!
//! each opcode has specific argument format requirements that this module handles.

use std::sync::OnceLock;

use color_eyre::eyre::eyre;
use color_eyre::Result;

use super::source::{EntropySource, GenerationSource};
use super::Generator;
use super::Version;
use crate::opcodes::{OpcodeKind, PICKLE_OPCODES};

static STDLIB_MODULES: OnceLock<Vec<String>> = OnceLock::new();

fn load_stdlib_complete() -> &'static Vec<String> {
    STDLIB_MODULES.get_or_init(|| {
        let content = include_str!("../../stdlib_complete.txt");
        content.lines().map(|s| s.to_string()).collect()
    })
}

impl Generator {
    /// emit an opcode with its arguments and update the simulated stack.
    ///
    /// this is the main entry point for opcode emission. it handles the complete
    /// emission flow: snapshot creation, opcode/argument writing, stack simulation,
    /// and post-emission mutations.
    ///
    /// # Opcode-Specific Handling
    ///
    /// - **integers**: delegates to `emit_int()` which chooses appropriate variant
    /// - **floats**: emits Float (ASCII) or BinFloat (binary) with mutations
    /// - **strings**: delegates to `emit_string()` for protocol-specific formatting
    /// - **bytes**: delegates to `emit_bytes()` for length-prefixed encoding
    /// - **globals**: delegates to `emit_global()` for module/class lookup
    /// - **memo ops**: generates or selects memo indices with mutations
    /// - **extensions**: generates extension registry codes (must be > 0)
    /// - **simple ops**: emits single byte via `emit_opcode()`
    ///
    /// # Parameters
    /// - `opcode`: the opcode kind to emit
    /// - `source`: entropy source for random values and mutation decisions
    ///
    /// # Returns
    /// `Ok(())` on success.
    ///
    /// # Errors
    /// returns an error if:
    /// - `emit_string()`, `emit_bytes()`, `emit_global()`, or `emit_int()` fail
    pub(super) fn emit_and_process(
        &mut self,
        opcode: OpcodeKind,
        source: &mut GenerationSource,
    ) -> Result<()> {
        use OpcodeKind::*;

        // create snapshot before emission
        let snapshot = self.create_snapshot();

        // emit the opcode and any required arguments
        match opcode {
            // integer opcodes - emit_int handles choosing the right variant
            Int | Long | Long1 | Long4 | BinInt | BinInt1 | BinInt2 => {
                self.emit_int(source)?;
            }

            // float opcodes
            Float => {
                let value = self.mutate_float(source.gen_f64(), source);
                self.output.push(Float.as_u8());
                let float_str = format!("{}\n", value);
                let arg_bytes = float_str.as_bytes();
                self.output.extend_from_slice(arg_bytes);
                self.process_stack_ops(Float, Some(arg_bytes));
            }
            BinFloat => {
                let value = self.mutate_float(source.gen_f64(), source);
                self.output.push(BinFloat.as_u8());
                let arg_bytes = value.to_be_bytes();
                self.output.extend_from_slice(&arg_bytes);
                self.process_stack_ops(BinFloat, Some(&arg_bytes));
            }

            // string opcodes
            String | Unicode | ShortBinUnicode | BinUnicode | BinUnicode8 => {
                self.emit_string(opcode, source)?;
            }

            // bytes opcodes
            BinString | ShortBinString | ShortBinBytes | BinBytes | BinBytes8 | ByteArray8 => {
                self.emit_bytes(opcode, source)?;
            }

            // global - needs module/class lookup
            Global => {
                self.emit_global(source)?;
            }

            // memo PUT operations need indices
            Put => {
                let index = self.state.memo.len();
                self.output.push(Put.as_u8());
                let index_str = format!("{}\n", index);
                let arg_bytes = index_str.as_bytes();
                self.output.extend_from_slice(arg_bytes);
                self.process_stack_ops(Put, Some(arg_bytes));
            }
            BinPut => {
                let index = (self.state.memo.len() % 256) as u8;
                self.output.push(BinPut.as_u8());
                self.output.push(index);
                self.process_stack_ops(BinPut, Some(&[index]));
            }
            LongBinPut => {
                let index = self.state.memo.len() as u32;
                self.output.push(LongBinPut.as_u8());
                self.output.extend_from_slice(&index.to_le_bytes());
                self.process_stack_ops(LongBinPut, Some(&index.to_le_bytes()));
            }

            // memo GET operations need to pick existing index
            Get => {
                let mut keys: Vec<_> = self.state.memo.keys().copied().collect();
                keys.sort_unstable();
                if !keys.is_empty() {
                    let index = keys[source.gen_range(0, keys.len())];
                    let index = self.mutate_memo_index(index, source);
                    self.output.push(Get.as_u8());
                    let index_str = format!("{}\n", index);
                    let arg_bytes = index_str.as_bytes();
                    self.output.extend_from_slice(arg_bytes);
                    self.process_stack_ops(Get, Some(arg_bytes));
                }
            }
            BinGet => {
                let mut valid_indices: Vec<usize> = self
                    .state
                    .memo
                    .keys()
                    .filter(|&&k| k < 256)
                    .copied()
                    .collect();
                valid_indices.sort_unstable();
                if !valid_indices.is_empty() {
                    let index = valid_indices[source.gen_range(0, valid_indices.len())];
                    let index = self.mutate_memo_index(index, source).min(255);
                    self.output.push(BinGet.as_u8());
                    self.output.push(index as u8);
                    self.process_stack_ops(BinGet, Some(&[index as u8]));
                }
            }
            LongBinGet => {
                let mut keys: Vec<_> = self.state.memo.keys().copied().collect();
                keys.sort_unstable();
                if !keys.is_empty() {
                    let index = keys[source.gen_range(0, keys.len())];
                    let index = self.mutate_memo_index(index, source);
                    self.output.push(LongBinGet.as_u8());
                    let index_bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&index_bytes);
                    self.process_stack_ops(LongBinGet, Some(&index_bytes));
                }
            }

            // extension registry codes (must be > 0)
            Ext1 => {
                // ext1: 1-byte unsigned, must be in range 1-255
                let code = source.gen_u8().saturating_add(1);
                debug_assert!(code >= 1, "EXT1 code out of range: {}", code);
                self.output.push(Ext1.as_u8());
                self.output.push(code);
                self.process_stack_ops(Ext1, Some(&[code]));
            }
            Ext2 => {
                // ext2: 2-byte unsigned, must be in range 1-65535
                let code = source.gen_u16().saturating_add(1);
                debug_assert!(code >= 1, "EXT2 code out of range: {}", code);
                self.output.push(Ext2.as_u8());
                self.output.extend_from_slice(&code.to_le_bytes());
                self.process_stack_ops(Ext2, Some(&code.to_le_bytes()));
            }
            Ext4 => {
                // ext4: 4-byte signed integer, must be > 0
                // use u32 and ensure it's positive
                let code = source.gen_u32().saturating_add(1);
                debug_assert!(code > 0, "EXT4 code must be > 0, got {}", code);
                self.output.push(Ext4.as_u8());
                self.output.extend_from_slice(&code.to_le_bytes());
                self.process_stack_ops(Ext4, Some(&code.to_le_bytes()));
            }

            // persid needs a persistent ID string
            PersID => {
                let pid = format!("pid_{}\n", source.gen_u32());
                self.output.push(PersID.as_u8());
                let arg_bytes = pid.as_bytes();
                self.output.extend_from_slice(arg_bytes);
                self.process_stack_ops(PersID, Some(arg_bytes));
            }

            // inst needs module and class name
            Inst => {
                if let Ok(module_class) = self.get_random_module(source) {
                    self.output.push(Inst.as_u8());
                    let arg_bytes = module_class.as_bytes();
                    self.output.extend_from_slice(arg_bytes);
                    self.process_stack_ops(Inst, Some(arg_bytes));
                }
            }

            // frame is handled specially in generate() - not emitted during normal generation
            Frame => {
                // this should never be called since can_emit returns false for Frame
                unreachable!("Frame should not be emitted during generation");
            }

            // opcodes without arguments - just emit directly
            _ => {
                self.emit_opcode(opcode);
            }
        }

        // post-process mutations
        self.post_process_emission(snapshot, source);

        Ok(())
    }

    /// emit a string opcode with protocol-specific formatting.
    ///
    /// generates a random ASCII string (up to 32 characters) and emits it using
    /// the specified string opcode. each opcode has different format requirements:
    ///
    /// - **String** (protocol 0): quoted Python string literal with newline
    /// - **Unicode** (protocol 0): raw unicode string with newline
    /// - **ShortBinUnicode** (protocol 4): 1-byte length prefix + UTF-8 bytes
    /// - **BinUnicode** (protocol 3): 4-byte length prefix + UTF-8 bytes
    /// - **BinUnicode8** (protocol 4): 8-byte length prefix + UTF-8 bytes
    ///
    /// applies string mutations before encoding.
    ///
    /// # Parameters
    /// - `opcode`: the string opcode variant to emit
    /// - `source`: entropy source for random string generation and mutations
    ///
    /// # Returns
    /// `Ok(())` on success.
    pub(super) fn emit_string(
        &mut self,
        opcode: OpcodeKind,
        source: &mut GenerationSource,
    ) -> Result<()> {
        use OpcodeKind::*;

        // generate a random short string
        let len = (source.gen_u8() % 32) as usize;
        let s: std::string::String = (0..len).map(|_| source.gen_ascii_char()).collect();

        // apply mutations
        let s = self.mutate_string(s, source);

        match opcode {
            String => {
                // string opcode (protocol 0) requires properly quoted python string
                self.output.push(opcode.as_u8());
                let quoted = format!("'{}'\n", s); // add single quotes
                let arg_bytes = quoted.into_bytes();
                self.output.extend_from_slice(&arg_bytes);
                self.process_stack_ops(opcode, Some(&arg_bytes));
            }
            Unicode => {
                // unicode opcode (protocol 0) - raw unicode string with newline
                self.output.push(opcode.as_u8());
                let s_with_newline = format!("{}\n", s);
                let arg_bytes = s_with_newline.into_bytes();
                self.output.extend_from_slice(&arg_bytes);
                self.process_stack_ops(opcode, Some(&arg_bytes));
            }
            ShortBinUnicode => {
                let bytes = s.into_bytes();
                if bytes.len() < 256 {
                    self.output.push(ShortBinUnicode.as_u8());
                    self.output.push(bytes.len() as u8);
                    self.output.extend_from_slice(&bytes);
                    self.process_stack_ops(ShortBinUnicode, Some(&bytes));
                }
            }
            BinUnicode => {
                let bytes = s.into_bytes();
                self.output.push(BinUnicode.as_u8());
                self.output
                    .extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                self.output.extend_from_slice(&bytes);
                self.process_stack_ops(BinUnicode, Some(&bytes));
            }
            BinUnicode8 => {
                let bytes = s.into_bytes();
                self.output.push(BinUnicode8.as_u8());
                self.output
                    .extend_from_slice(&(bytes.len() as u64).to_le_bytes());
                self.output.extend_from_slice(&bytes);
                self.process_stack_ops(BinUnicode8, Some(&bytes));
            }
            _ => {}
        }

        Ok(())
    }

    /// emit a bytes opcode with length-prefixed encoding.
    ///
    /// generates random bytes (up to 32 bytes) and emits them using the specified
    /// bytes opcode. each opcode has different length encoding:
    ///
    /// - **BinString** (protocol 0): 4-byte signed int length + bytes
    /// - **ShortBinString/ShortBinBytes**: 1-byte length + bytes (max 255)
    /// - **BinBytes** (protocol 3): 4-byte unsigned int length + bytes
    /// - **BinBytes8/ByteArray8** (protocol 4/5): 8-byte unsigned int length + bytes
    ///
    /// applies byte mutations before encoding.
    ///
    /// # Parameters
    /// - `opcode`: the bytes opcode variant to emit
    /// - `source`: entropy source for random byte generation and mutations
    ///
    /// # Returns
    /// `Ok(())` on success.
    pub(super) fn emit_bytes(
        &mut self,
        opcode: OpcodeKind,
        source: &mut GenerationSource,
    ) -> Result<()> {
        use OpcodeKind::*;

        // generate random bytes
        let len = (source.gen_u8() % 32) as usize;
        let bytes: Vec<u8> = (0..len).map(|_| source.gen_u8()).collect();

        // apply mutations
        let bytes = self.mutate_bytes(bytes, source);

        match opcode {
            BinString => {
                // binstring uses 4-byte signed int for length (protocol 0)
                self.output.push(opcode.as_u8());
                self.output
                    .extend_from_slice(&(bytes.len() as i32).to_le_bytes());
                self.output.extend_from_slice(&bytes);
                self.process_stack_ops(opcode, Some(&bytes));
            }
            ShortBinString | ShortBinBytes => {
                // short_binstring and short_binbytes use 1-byte length
                if bytes.len() < 256 {
                    self.output.push(opcode.as_u8());
                    self.output.push(bytes.len() as u8);
                    self.output.extend_from_slice(&bytes);
                    self.process_stack_ops(opcode, Some(&bytes));
                }
            }
            BinBytes => {
                // binbytes uses 4-byte unsigned int for length (protocol 3)
                self.output.push(opcode.as_u8());
                self.output
                    .extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                self.output.extend_from_slice(&bytes);
                self.process_stack_ops(opcode, Some(&bytes));
            }
            BinBytes8 | ByteArray8 => {
                // binbytes8 and bytearray8 use 8-byte unsigned int for length (protocol 4/5)
                self.output.push(opcode.as_u8());
                self.output
                    .extend_from_slice(&(bytes.len() as u64).to_le_bytes());
                self.output.extend_from_slice(&bytes);
                self.process_stack_ops(opcode, Some(&bytes));
            }
            _ => {}
        }

        Ok(())
    }

    /// emit a single opcode byte with no arguments.
    ///
    /// used for simple opcodes that don't require arguments (e.g., Mark, Pop, Dup,
    /// EmptyList, EmptyDict, None, NewTrue, NewFalse). writes the opcode byte to
    /// the output buffer and updates the simulated stack state.
    ///
    /// # Parameters
    /// - `opcode`: the opcode to emit
    pub(super) fn emit_opcode(&mut self, opcode: OpcodeKind) {
        self.output.push(opcode.as_u8());
        self.process_stack_ops(opcode, None);
    }

    /// emit the PROTO opcode if appropriate for the protocol version.
    ///
    /// the PROTO opcode declares the pickle protocol version and is required for
    /// protocols 2 and above. it must be the first opcode in the pickle stream.
    /// this method ensures PROTO is only emitted once.
    ///
    /// protocols 0 and 1 don't use PROTO and are identified by their opcodes.
    ///
    /// # Parameters
    /// - `_source`: entropy source (currently unused, reserved for future use)
    pub(super) fn emit_proto(&mut self, _source: &mut GenerationSource) {
        if self.state.version == Version::V0 || self.state.version == Version::V1 {
            return;
        }

        if self.state.proto_emitted {
            return; // already emitted
        }

        self.output.push(OpcodeKind::Proto.as_u8());
        self.output.push(self.state.version as u8);
        self.state.proto_emitted = true;
    }

    /// emit a GLOBAL opcode with a random module and class name.
    ///
    /// selects a random Python standard library module and class from the embedded
    /// `stdlib_complete.txt` data. emits the GLOBAL opcode followed by two newline-terminated
    /// strings: module name and class name.
    ///
    /// # Parameters
    /// - `source`: entropy source for random module selection
    ///
    /// # Returns
    /// `Ok(())` on success.
    pub(super) fn emit_global(&mut self, source: &mut GenerationSource) -> Result<()> {
        let module = self.get_random_module(source)?;

        self.output.push(OpcodeKind::Global.as_u8());

        let arg_bytes = module.as_bytes().to_vec();
        self.output.extend_from_slice(&arg_bytes);

        self.process_stack_ops(OpcodeKind::Global, Some(&arg_bytes));

        Ok(())
    }

    /// emit an integer opcode with protocol-appropriate encoding.
    ///
    /// randomly selects an integer opcode variant available in the current protocol
    /// version and emits it with a random integer value. handles all integer encoding
    /// formats:
    ///
    /// - **Int** (protocol 0): ASCII decimal with newline
    /// - **Long** (protocol 0): ASCII decimal with 'L' suffix and newline
    /// - **Long1** (protocol 1): 1-byte size + little-endian bytes
    /// - **Long4** (protocol 1): 4-byte size + little-endian bytes
    /// - **BinInt** (protocol 1): 4-byte signed little-endian
    /// - **BinInt1** (protocol 1): 1-byte unsigned (0-255)
    /// - **BinInt2** (protocol 1): 2-byte unsigned little-endian
    ///
    /// applies integer mutations before encoding.
    ///
    /// # Parameters
    /// - `source`: entropy source for random value generation and mutations
    ///
    /// # Returns
    /// `Ok(())` on success.
    pub(super) fn emit_int(&mut self, source: &mut GenerationSource) -> Result<()> {
        // get all opcodes for the current version
        let version = self.state.version as u8;
        let Some(valid_kinds) = PICKLE_OPCODES.get(&version) else {
            return Err(eyre!("No opcodes available for version {}", version));
        };

        // filter for int-like opcodes
        let int_like: Vec<OpcodeKind> = valid_kinds
            .iter()
            .cloned()
            .filter(|k| {
                matches!(
                    k,
                    OpcodeKind::Int
                        | OpcodeKind::Long
                        | OpcodeKind::Long1
                        | OpcodeKind::Long4
                        | OpcodeKind::BinInt
                        | OpcodeKind::BinInt1
                        | OpcodeKind::BinInt2
                )
            })
            .collect();

        // grab random opcode kind and emit it with a random integer argument
        let idx = source.choose_index(int_like.len());
        let chosen = int_like[idx];

        // write the opcode byte directly (don't use emit_opcode which would process stack ops prematurely)
        self.output.push(chosen.as_u8());

        let int = self.mutate_int(source.gen_i32(), source);

        let arg: Vec<u8> = match chosen {
            OpcodeKind::Int => format!("{int}\n").into_bytes(),
            OpcodeKind::Long => format!("{int}L\n").into_bytes(),
            OpcodeKind::Long1 => {
                let size = 4u8;
                let bytes = int.to_le_bytes();
                let mut v = vec![size];
                v.extend_from_slice(&bytes);
                v
            }
            OpcodeKind::Long4 => {
                let size = 4u32;
                let size_bytes = size.to_le_bytes();
                let int_bytes = int.to_le_bytes();
                let mut v = Vec::with_capacity(8);
                v.extend_from_slice(&size_bytes);
                v.extend_from_slice(&int_bytes);
                v
            }
            OpcodeKind::BinInt => int.to_le_bytes().to_vec(),
            OpcodeKind::BinInt1 => vec![(int & 0xFF) as u8],
            OpcodeKind::BinInt2 => {
                let bytes = (int & 0xFFFF).to_le_bytes();
                bytes[..2].to_vec()
            }
            _ => {
                return Err(eyre!(
                    "Unexpected opcode kind for integer emission: {:?}",
                    chosen
                ));
            }
        };

        self.output.extend_from_slice(&arg);
        self.process_stack_ops(chosen, Some(&arg));
        Ok(())
    }

    /// get a random module and class name from the Python standard library.
    ///
    /// uses embedded `stdlib_complete.txt` data (cached after first access) which contains
    /// lines in the format "module.class", randomly selects one, and formats it as
    /// "module\nclass\n" for use with GLOBAL or INST opcodes.
    ///
    /// # Parameters
    /// - `source`: entropy source for random selection
    ///
    /// # Returns
    /// a string formatted as "module\nclass\n".
    pub(super) fn get_random_module(&self, source: &mut GenerationSource) -> Result<String> {
        let modules = load_stdlib_complete();
        let idx = source.choose_index(modules.len());
        let chosen = &modules[idx];

        let mut iter = chosen.splitn(2, '.');
        let module = iter.next().unwrap_or("builtins");
        let attr = iter.next().unwrap_or("object");

        Ok(format!("{}\n{}\n", module, attr))
    }
}
