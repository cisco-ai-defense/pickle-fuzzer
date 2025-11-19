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

use super::{EmissionSnapshot, Mutator};
use crate::generator::{EntropySource, GenerationSource};
use crate::opcodes::OpcodeKind;

/// Type confusion mutator: replaces value-pushing opcodes with incompatible types.
///
/// This mutator exploits vulnerabilities in pickle scanners that assume opcodes
/// push specific types. It detects when a value is pushed and replaces the entire
/// opcode+args with a different type.
///
/// Examples:
/// - String opcode → Int opcode (causes type errors in string-expecting code)
/// - Int opcode → Float opcode (causes type errors in int-expecting code)
/// - Float opcode → List opcode (causes type errors in numeric code)
///
/// This is more general than just string confusion - it creates type mismatches
/// for any value-pushing opcode.
#[derive(Debug)]
pub struct TypeConfusionMutator {
    unsafe_mode: bool,
}

/// Types that opcodes can push onto the stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackType {
    Int,
    Float,
    String,
    Bytes,
    List,
    Dict,
    Tuple,
    None,
    Bool,
}

impl TypeConfusionMutator {
    pub fn new(unsafe_mode: bool) -> Self {
        Self { unsafe_mode }
    }

    /// Determine what type an opcode pushes onto the stack
    fn opcode_to_type(opcode_byte: u8) -> Option<StackType> {
        // Convert byte to OpcodeKind
        match opcode_byte {
            // Integers
            0x49 | 0x4a | 0x4b | 0x4d => Some(StackType::Int), // Int, BinInt, BinInt1, BinInt2
            0x4c | 0x8a | 0x8b => Some(StackType::Int),        // Long, Long1, Long4

            // Floats
            0x46 | 0x47 => Some(StackType::Float), // Float, BinFloat

            // Strings
            0x53 | 0x56 | 0x8c | 0x58 | 0x8d => Some(StackType::String), // String, Unicode, ShortBinUnicode, BinUnicode, BinUnicode8

            // Bytes
            0x42 | 0x43 | 0x8e | 0x54 | 0x55 => Some(StackType::Bytes), // BinBytes, ShortBinBytes, BinBytes8, BinString, ShortBinString

            // Lists
            0x5d | 0x6c => Some(StackType::List), // EmptyList, List

            // Tuples
            0x29 | 0x74 | 0x85 | 0x86 | 0x87 => Some(StackType::Tuple), // EmptyTuple, Tuple, Tuple1, Tuple2, Tuple3

            // Dicts
            0x7d | 0x64 => Some(StackType::Dict), // EmptyDict, Dict

            // None
            0x4e => Some(StackType::None), // None

            // Booleans
            0x88 | 0x89 => Some(StackType::Bool), // NewTrue, NewFalse

            _ => None, // Not a value-pushing opcode
        }
    }

    /// Choose a different type to confuse with
    fn choose_wrong_type(original: StackType, source: &mut GenerationSource) -> StackType {
        let all_types = [
            StackType::Int,
            StackType::Float,
            StackType::String,
            StackType::Bytes,
            StackType::List,
            StackType::Dict,
            StackType::Tuple,
            StackType::None,
            StackType::Bool,
        ];

        // Filter out the original type
        let different_types: Vec<_> = all_types
            .iter()
            .filter(|&&t| t != original)
            .copied()
            .collect();

        different_types[source.choose_index(different_types.len())]
    }

    /// Generate bytecode for a specific type
    fn generate_opcode_for_type(stack_type: StackType, source: &mut GenerationSource) -> Vec<u8> {
        match stack_type {
            StackType::Int => {
                // BININT (0x4a) + 4 bytes little-endian
                let mut bytes = vec![OpcodeKind::BinInt.as_u8()];
                bytes.extend_from_slice(&source.gen_i32().to_le_bytes());
                bytes
            }
            StackType::Float => {
                // BINFLOAT (0x47) + 8 bytes big-endian
                let mut bytes = vec![OpcodeKind::BinFloat.as_u8()];
                bytes.extend_from_slice(&source.gen_f64().to_be_bytes());
                bytes
            }
            StackType::String => {
                // SHORT_BINUNICODE (0x8c) + 1 byte len + data
                let s = "confused";
                let mut bytes = vec![OpcodeKind::ShortBinUnicode.as_u8()];
                bytes.push(s.len() as u8);
                bytes.extend_from_slice(s.as_bytes());
                bytes
            }
            StackType::Bytes => {
                // SHORT_BINBYTES (0x43) + 1 byte len + data
                let data = b"confused";
                let mut bytes = vec![OpcodeKind::ShortBinBytes.as_u8()];
                bytes.push(data.len() as u8);
                bytes.extend_from_slice(data);
                bytes
            }
            StackType::List => {
                // EMPTY_LIST (0x5d)
                vec![OpcodeKind::EmptyList.as_u8()]
            }
            StackType::Dict => {
                // EMPTY_DICT (0x7d)
                vec![OpcodeKind::EmptyDict.as_u8()]
            }
            StackType::Tuple => {
                // EMPTY_TUPLE (0x29)
                vec![OpcodeKind::EmptyTuple.as_u8()]
            }
            StackType::None => {
                // NONE (0x4e)
                vec![OpcodeKind::None.as_u8()]
            }
            StackType::Bool => {
                // NEWTRUE (0x88) or NEWFALSE (0x89)
                if source.gen_bool() {
                    vec![OpcodeKind::NewTrue.as_u8()]
                } else {
                    vec![OpcodeKind::NewFalse.as_u8()]
                }
            }
        }
    }
}

impl Mutator for TypeConfusionMutator {
    fn name(&self) -> &str {
        "typeconfusion"
    }

    fn is_unsafe(&self) -> bool {
        true // Always unsafe - violates type safety
    }

    fn post_process(
        &self,
        snapshot: &EmissionSnapshot,
        output: &mut Vec<u8>,
        source: &mut GenerationSource,
        rate: f64,
    ) -> bool {
        if !self.unsafe_mode || source.gen_f64() > rate {
            return false;
        }

        // Check if anything was emitted
        if snapshot.output_delta.is_empty() {
            return false;
        }

        // Get the opcode that was just emitted
        let emitted_opcode = snapshot.output_delta[0];

        // Replace value-pushing opcodes with incompatible types
        // This will naturally cause type confusion when these values are later
        // used by opcodes like STACK_GLOBAL that expect specific types
        if let Some(original_type) = Self::opcode_to_type(emitted_opcode) {
            // Choose a different type to confuse with
            let wrong_type = Self::choose_wrong_type(original_type, source);

            // Generate replacement bytecode
            let replacement = Self::generate_opcode_for_type(wrong_type, source);

            // Replace the entire emission (from snapshot.output_len to current end)
            output.truncate(snapshot.output_len);
            output.extend_from_slice(&replacement);

            return true;
        }

        false
    }
}
