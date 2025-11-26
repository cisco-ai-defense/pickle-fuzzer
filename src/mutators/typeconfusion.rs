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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GenerationSource;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_typeconfusion_name() {
        let mutator = TypeConfusionMutator::new(true);
        assert_eq!(mutator.name(), "typeconfusion");
    }

    #[test]
    fn test_typeconfusion_is_always_unsafe() {
        let mutator = TypeConfusionMutator::new(true);
        assert!(mutator.is_unsafe());

        let mutator_false = TypeConfusionMutator::new(false);
        assert!(mutator_false.is_unsafe());
    }

    #[test]
    fn test_typeconfusion_opcode_to_type() {
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x4a),
            Some(StackType::Int)
        ); // BinInt
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x47),
            Some(StackType::Float)
        ); // BinFloat
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x8c),
            Some(StackType::String)
        ); // ShortBinUnicode
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x43),
            Some(StackType::Bytes)
        ); // ShortBinBytes
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x5d),
            Some(StackType::List)
        ); // EmptyList
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x7d),
            Some(StackType::Dict)
        ); // EmptyDict
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x29),
            Some(StackType::Tuple)
        ); // EmptyTuple
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x4e),
            Some(StackType::None)
        ); // None
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(0x88),
            Some(StackType::Bool)
        ); // NewTrue
        assert_eq!(TypeConfusionMutator::opcode_to_type(0xFF), None); // Invalid opcode
    }

    #[test]
    fn test_typeconfusion_choose_wrong_type() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let original = StackType::Int;
        for _ in 0..10 {
            let wrong = TypeConfusionMutator::choose_wrong_type(original, &mut source);
            assert_ne!(wrong, original, "should choose different type");
        }
    }

    #[test]
    fn test_typeconfusion_generate_opcode_for_type() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        // test each type generates valid bytecode
        let int_bytes = TypeConfusionMutator::generate_opcode_for_type(StackType::Int, &mut source);
        assert_eq!(int_bytes[0], OpcodeKind::BinInt.as_u8());
        assert_eq!(int_bytes.len(), 5); // opcode + 4 bytes

        let float_bytes =
            TypeConfusionMutator::generate_opcode_for_type(StackType::Float, &mut source);
        assert_eq!(float_bytes[0], OpcodeKind::BinFloat.as_u8());
        assert_eq!(float_bytes.len(), 9); // opcode + 8 bytes

        let string_bytes =
            TypeConfusionMutator::generate_opcode_for_type(StackType::String, &mut source);
        assert_eq!(string_bytes[0], OpcodeKind::ShortBinUnicode.as_u8());

        let list_bytes =
            TypeConfusionMutator::generate_opcode_for_type(StackType::List, &mut source);
        assert_eq!(list_bytes[0], OpcodeKind::EmptyList.as_u8());
        assert_eq!(list_bytes.len(), 1);

        let none_bytes =
            TypeConfusionMutator::generate_opcode_for_type(StackType::None, &mut source);
        assert_eq!(none_bytes[0], OpcodeKind::None.as_u8());
        assert_eq!(none_bytes.len(), 1);
    }

    #[test]
    fn test_typeconfusion_post_process_disabled_in_safe_mode() {
        let mutator = TypeConfusionMutator::new(false);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let snapshot = EmissionSnapshot {
            stack_depth: 0,
            output_len: 0,
            memo_size: 0,
            stack_delta: vec![],
            output_delta: vec![0x4a, 0x01, 0x00, 0x00, 0x00], // BinInt
            memo_delta: vec![],
        };

        let mut output = vec![0x4a, 0x01, 0x00, 0x00, 0x00];
        let result = mutator.post_process(&snapshot, &mut output, &mut source, 1.0);

        assert!(!result, "should not mutate in safe mode");
    }

    #[test]
    fn test_typeconfusion_post_process_replaces_opcode() {
        let mutator = TypeConfusionMutator::new(true);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let snapshot = EmissionSnapshot {
            stack_depth: 0,
            output_len: 0,
            memo_size: 0,
            stack_delta: vec![],
            output_delta: vec![0x4a, 0x01, 0x00, 0x00, 0x00], // BinInt
            memo_delta: vec![],
        };

        let output = vec![0x4a, 0x01, 0x00, 0x00, 0x00];

        // try multiple times to get a mutation
        let mut mutated = false;
        for _ in 0..20 {
            let mut test_output = output.clone();
            if mutator.post_process(&snapshot, &mut test_output, &mut source, 1.0) {
                mutated = true;
                assert_ne!(
                    test_output[0], 0x4a,
                    "should replace BinInt with different opcode"
                );
                break;
            }
        }

        assert!(mutated, "should eventually mutate at rate 1.0");
    }
}
