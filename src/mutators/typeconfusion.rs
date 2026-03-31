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

use super::{EmissionSnapshot, Mutator, PostProcessEmission};
use crate::generator::{EntropySource, GenerationSource};
use crate::opcodes::{OpcodeKind, PICKLE_OPCODES};
use crate::Version;

/// Type confusion mutator: replaces pure value-pushing opcodes with
/// incompatible but protocol-valid values.
///
/// This mutator is unsafe by design because it intentionally breaks type
/// expectations for later stack consumers such as `STACK_GLOBAL`.
#[derive(Debug)]
pub struct TypeConfusionMutator;

/// Types that pure push opcodes can place on the stack.
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
        assert!(
            unsafe_mode,
            "TypeConfusionMutator requires unsafe_mode=true"
        );
        Self
    }

    fn supports_opcode(version: Version, opcode: OpcodeKind) -> bool {
        PICKLE_OPCODES
            .get(&(version as u8))
            .is_some_and(|opcodes| opcodes.contains(&opcode))
    }

    fn available_replacement_types(version: Version) -> Vec<StackType> {
        let mut types = vec![
            StackType::Int,
            StackType::Float,
            StackType::String,
            StackType::None,
        ];

        if Self::supports_opcode(version, OpcodeKind::ShortBinString)
            || Self::supports_opcode(version, OpcodeKind::ShortBinBytes)
        {
            types.push(StackType::Bytes);
        }
        if Self::supports_opcode(version, OpcodeKind::EmptyList) {
            types.push(StackType::List);
        }
        if Self::supports_opcode(version, OpcodeKind::EmptyDict) {
            types.push(StackType::Dict);
        }
        if Self::supports_opcode(version, OpcodeKind::EmptyTuple) {
            types.push(StackType::Tuple);
        }
        if Self::supports_opcode(version, OpcodeKind::NewTrue)
            || Self::supports_opcode(version, OpcodeKind::Int)
        {
            types.push(StackType::Bool);
        }

        types
    }

    /// Determine what type a pure push opcode places onto the stack.
    ///
    /// Post-processing runs after stack simulation, so opcodes that consume
    /// existing stack state cannot be safely rewritten here.
    fn opcode_to_type(opcode_byte: u8) -> Option<StackType> {
        match opcode_byte {
            0x49 | 0x4a | 0x4b | 0x4d | 0x4c | 0x8a | 0x8b => Some(StackType::Int),
            0x46 | 0x47 => Some(StackType::Float),
            0x53 | 0x56 | 0x8c | 0x58 | 0x8d => Some(StackType::String),
            0x42 | 0x43 | 0x8e | 0x54 | 0x55 => Some(StackType::Bytes),
            0x5d => Some(StackType::List),
            0x7d => Some(StackType::Dict),
            0x29 => Some(StackType::Tuple),
            0x4e => Some(StackType::None),
            0x88 | 0x89 => Some(StackType::Bool),
            _ => None,
        }
    }

    fn choose_wrong_type(
        version: Version,
        original: StackType,
        source: &mut GenerationSource,
    ) -> Option<StackType> {
        let different_types: Vec<_> = Self::available_replacement_types(version)
            .into_iter()
            .filter(|candidate| *candidate != original)
            .collect();

        if different_types.is_empty() {
            None
        } else {
            Some(different_types[source.choose_index(different_types.len())])
        }
    }

    fn generate_opcode_for_type(
        stack_type: StackType,
        version: Version,
        source: &mut GenerationSource,
    ) -> Option<Vec<u8>> {
        match stack_type {
            StackType::Int => {
                if !Self::supports_opcode(version, OpcodeKind::Int) {
                    return None;
                }

                let mut bytes = vec![OpcodeKind::Int.as_u8()];
                bytes.extend_from_slice(format!("{}\n", source.gen_i32()).as_bytes());
                Some(bytes)
            }
            StackType::Float => {
                if !Self::supports_opcode(version, OpcodeKind::Float) {
                    return None;
                }

                let mut bytes = vec![OpcodeKind::Float.as_u8()];
                bytes.extend_from_slice(format!("{}\n", source.gen_f64()).as_bytes());
                Some(bytes)
            }
            StackType::String => {
                if !Self::supports_opcode(version, OpcodeKind::Unicode) {
                    return None;
                }

                let mut bytes = vec![OpcodeKind::Unicode.as_u8()];
                bytes.extend_from_slice(b"confused\n");
                Some(bytes)
            }
            StackType::Bytes => {
                let data = b"confused";
                if Self::supports_opcode(version, OpcodeKind::ShortBinBytes) {
                    let mut bytes = vec![OpcodeKind::ShortBinBytes.as_u8(), data.len() as u8];
                    bytes.extend_from_slice(data);
                    Some(bytes)
                } else if Self::supports_opcode(version, OpcodeKind::ShortBinString) {
                    let mut bytes = vec![OpcodeKind::ShortBinString.as_u8(), data.len() as u8];
                    bytes.extend_from_slice(data);
                    Some(bytes)
                } else {
                    None
                }
            }
            StackType::List => Self::supports_opcode(version, OpcodeKind::EmptyList)
                .then_some(vec![OpcodeKind::EmptyList.as_u8()]),
            StackType::Dict => Self::supports_opcode(version, OpcodeKind::EmptyDict)
                .then_some(vec![OpcodeKind::EmptyDict.as_u8()]),
            StackType::Tuple => Self::supports_opcode(version, OpcodeKind::EmptyTuple)
                .then_some(vec![OpcodeKind::EmptyTuple.as_u8()]),
            StackType::None => Self::supports_opcode(version, OpcodeKind::None)
                .then_some(vec![OpcodeKind::None.as_u8()]),
            StackType::Bool => {
                if Self::supports_opcode(version, OpcodeKind::NewTrue) {
                    Some(if source.gen_bool() {
                        vec![OpcodeKind::NewTrue.as_u8()]
                    } else {
                        vec![OpcodeKind::NewFalse.as_u8()]
                    })
                } else if Self::supports_opcode(version, OpcodeKind::Int) {
                    let literal = if source.gen_bool() { b"01\n" } else { b"00\n" };
                    let mut bytes = vec![OpcodeKind::Int.as_u8()];
                    bytes.extend_from_slice(literal);
                    Some(bytes)
                } else {
                    None
                }
            }
        }
    }

    fn describe_replacement(output: &[u8]) -> Option<PostProcessEmission> {
        let opcode = *output.first()?;

        match opcode {
            value if value == OpcodeKind::Int.as_u8() => Some(PostProcessEmission {
                opcode: OpcodeKind::Int,
                arg_bytes: Some(output[1..].to_vec()),
            }),
            value if value == OpcodeKind::Float.as_u8() => Some(PostProcessEmission {
                opcode: OpcodeKind::Float,
                arg_bytes: Some(output[1..].to_vec()),
            }),
            value if value == OpcodeKind::Unicode.as_u8() => Some(PostProcessEmission {
                opcode: OpcodeKind::Unicode,
                arg_bytes: Some(output[1..].to_vec()),
            }),
            value if value == OpcodeKind::ShortBinString.as_u8() => {
                let len = *output.get(1)? as usize;
                (output.len() == 2 + len).then(|| PostProcessEmission {
                    opcode: OpcodeKind::ShortBinString,
                    arg_bytes: Some(output[2..].to_vec()),
                })
            }
            value if value == OpcodeKind::ShortBinBytes.as_u8() => {
                let len = *output.get(1)? as usize;
                (output.len() == 2 + len).then(|| PostProcessEmission {
                    opcode: OpcodeKind::ShortBinBytes,
                    arg_bytes: Some(output[2..].to_vec()),
                })
            }
            value if value == OpcodeKind::EmptyList.as_u8() && output.len() == 1 => {
                Some(PostProcessEmission {
                    opcode: OpcodeKind::EmptyList,
                    arg_bytes: None,
                })
            }
            value if value == OpcodeKind::EmptyDict.as_u8() && output.len() == 1 => {
                Some(PostProcessEmission {
                    opcode: OpcodeKind::EmptyDict,
                    arg_bytes: None,
                })
            }
            value if value == OpcodeKind::EmptyTuple.as_u8() && output.len() == 1 => {
                Some(PostProcessEmission {
                    opcode: OpcodeKind::EmptyTuple,
                    arg_bytes: None,
                })
            }
            value if value == OpcodeKind::None.as_u8() && output.len() == 1 => {
                Some(PostProcessEmission {
                    opcode: OpcodeKind::None,
                    arg_bytes: None,
                })
            }
            value if value == OpcodeKind::NewTrue.as_u8() && output.len() == 1 => {
                Some(PostProcessEmission {
                    opcode: OpcodeKind::NewTrue,
                    arg_bytes: None,
                })
            }
            value if value == OpcodeKind::NewFalse.as_u8() && output.len() == 1 => {
                Some(PostProcessEmission {
                    opcode: OpcodeKind::NewFalse,
                    arg_bytes: None,
                })
            }
            _ => None,
        }
    }
}

impl Mutator for TypeConfusionMutator {
    fn name(&self) -> &str {
        "typeconfusion"
    }

    fn is_unsafe(&self) -> bool {
        true
    }

    fn post_process(
        &self,
        snapshot: &EmissionSnapshot,
        output: &mut Vec<u8>,
        source: &mut GenerationSource,
        rate: f64,
    ) -> bool {
        if source.gen_f64() > rate {
            return false;
        }

        let emitted_opcode = match snapshot.output_delta.first() {
            Some(opcode) => *opcode,
            None => return false,
        };

        let original_type = match Self::opcode_to_type(emitted_opcode) {
            Some(stack_type) => stack_type,
            None => return false,
        };

        let wrong_type = match Self::choose_wrong_type(snapshot.version, original_type, source) {
            Some(stack_type) => stack_type,
            None => return false,
        };

        let replacement = match Self::generate_opcode_for_type(wrong_type, snapshot.version, source)
        {
            Some(bytes) => bytes,
            None => return false,
        };

        output.truncate(snapshot.output_len);
        output.extend_from_slice(&replacement);
        true
    }

    fn describe_post_process(
        &self,
        _snapshot: &EmissionSnapshot,
        output: &[u8],
    ) -> Option<PostProcessEmission> {
        Self::describe_replacement(output)
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
    }

    #[test]
    #[should_panic(expected = "TypeConfusionMutator requires unsafe_mode=true")]
    fn test_typeconfusion_requires_unsafe_mode() {
        let _ = TypeConfusionMutator::new(false);
    }

    #[test]
    fn test_typeconfusion_opcode_to_type_only_matches_pure_pushes() {
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::BinInt.as_u8()),
            Some(StackType::Int)
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::BinFloat.as_u8()),
            Some(StackType::Float)
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::ShortBinUnicode.as_u8()),
            Some(StackType::String)
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::ShortBinBytes.as_u8()),
            Some(StackType::Bytes)
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::EmptyList.as_u8()),
            Some(StackType::List)
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::List.as_u8()),
            None
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::EmptyDict.as_u8()),
            Some(StackType::Dict)
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::Dict.as_u8()),
            None
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::EmptyTuple.as_u8()),
            Some(StackType::Tuple)
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::Tuple.as_u8()),
            None
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::Tuple1.as_u8()),
            None
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::Tuple2.as_u8()),
            None
        );
        assert_eq!(
            TypeConfusionMutator::opcode_to_type(OpcodeKind::Tuple3.as_u8()),
            None
        );
    }

    #[test]
    fn test_typeconfusion_choose_wrong_type_stays_within_supported_protocol_types() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        for _ in 0..10 {
            let wrong =
                TypeConfusionMutator::choose_wrong_type(Version::V0, StackType::Int, &mut source)
                    .expect("should find replacement type");
            assert_ne!(wrong, StackType::Int);
            assert_ne!(wrong, StackType::Bytes);
            assert_ne!(wrong, StackType::List);
            assert_ne!(wrong, StackType::Dict);
            assert_ne!(wrong, StackType::Tuple);
        }
    }

    #[test]
    fn test_typeconfusion_generate_opcode_for_type_is_protocol_aware() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let bytes_v0 = TypeConfusionMutator::generate_opcode_for_type(
            StackType::Bytes,
            Version::V0,
            &mut source,
        );
        assert!(bytes_v0.is_none());

        let bytes_v1 = TypeConfusionMutator::generate_opcode_for_type(
            StackType::Bytes,
            Version::V1,
            &mut source,
        )
        .expect("protocol 1 should support a bytes-like replacement");
        assert_eq!(bytes_v1[0], OpcodeKind::ShortBinString.as_u8());

        let bool_v0 = TypeConfusionMutator::generate_opcode_for_type(
            StackType::Bool,
            Version::V0,
            &mut source,
        )
        .expect("protocol 0 bool should fall back to INT literals");
        assert_eq!(bool_v0[0], OpcodeKind::Int.as_u8());

        let bool_v4 = TypeConfusionMutator::generate_opcode_for_type(
            StackType::Bool,
            Version::V4,
            &mut source,
        )
        .expect("protocol 4 should use NEWTRUE/NEWFALSE");
        assert!(matches!(
            bool_v4[0],
            value if value == OpcodeKind::NewTrue.as_u8()
                || value == OpcodeKind::NewFalse.as_u8()
        ));
    }

    #[test]
    fn test_typeconfusion_describe_replacement_parses_supported_outputs() {
        let described = TypeConfusionMutator::describe_replacement(&[
            OpcodeKind::ShortBinBytes.as_u8(),
            3,
            b'a',
            b'b',
            b'c',
        ])
        .expect("replacement should be described");

        assert_eq!(described.opcode, OpcodeKind::ShortBinBytes);
        assert_eq!(described.arg_bytes, Some(b"abc".to_vec()));
    }

    #[test]
    fn test_typeconfusion_post_process_replaces_opcode() {
        let mutator = TypeConfusionMutator::new(true);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let snapshot = EmissionSnapshot {
            version: Version::V4,
            stack_depth: 0,
            output_len: 0,
            memo_size: 0,
            stack_delta: vec![],
            output_delta: vec![OpcodeKind::BinInt.as_u8(), 1, 0, 0, 0],
            memo_delta: vec![],
        };

        let output = vec![OpcodeKind::BinInt.as_u8(), 1, 0, 0, 0];

        let mut mutated = false;
        for _ in 0..20 {
            let mut test_output = output.clone();
            if mutator.post_process(&snapshot, &mut test_output, &mut source, 1.0) {
                mutated = true;
                assert_ne!(test_output[0], OpcodeKind::BinInt.as_u8());
                let described = mutator
                    .describe_post_process(&snapshot, &test_output)
                    .expect("rewritten opcode should be replayable");
                assert_ne!(described.opcode.as_u8(), OpcodeKind::BinInt.as_u8());
                break;
            }
        }

        assert!(mutated, "should eventually mutate at rate 1.0");
    }

    #[test]
    fn test_typeconfusion_post_process_skips_stack_consuming_constructors() {
        let mutator = TypeConfusionMutator::new(true);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let snapshot = EmissionSnapshot {
            version: Version::V4,
            stack_depth: 1,
            output_len: 0,
            memo_size: 0,
            stack_delta: vec![],
            output_delta: vec![OpcodeKind::List.as_u8()],
            memo_delta: vec![],
        };

        let mut output = vec![OpcodeKind::List.as_u8()];
        assert!(!mutator.post_process(&snapshot, &mut output, &mut source, 1.0));
        assert_eq!(output, vec![OpcodeKind::List.as_u8()]);
    }
}
