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

use super::Mutator;
use crate::generator::{EntropySource, GenerationSource};

/// Mutates individual characters/bytes in strings.
#[derive(Debug)]
pub struct CharacterMutator;

impl Mutator for CharacterMutator {
    fn name(&self) -> &str {
        "character"
    }

    fn mutate_string(
        &self,
        value: String,
        source: &mut GenerationSource,
        rate: f64,
    ) -> Option<String> {
        if source.gen_f64() > rate || value.is_empty() {
            return None;
        }

        let mut chars: Vec<char> = value.chars().collect();
        let idx = source.gen_range(0, chars.len());
        // replace with random printable ASCII
        chars[idx] = (source.gen_u8() % 94 + 33) as char;

        Some(chars.into_iter().collect())
    }

    fn mutate_bytes(
        &self,
        value: Vec<u8>,
        source: &mut GenerationSource,
        rate: f64,
    ) -> Option<Vec<u8>> {
        if source.gen_f64() > rate || value.is_empty() {
            return None;
        }

        let mut result = value.clone();
        let idx = source.gen_range(0, result.len());
        result[idx] = source.gen_u8();

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GenerationSource;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_character_name() {
        let mutator = CharacterMutator;
        assert_eq!(mutator.name(), "character");
    }

    #[test]
    fn test_character_mutate_string() {
        let mutator = CharacterMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let original = "hello".to_string();
        let result = mutator.mutate_string(original.clone(), &mut source, 1.0);
        
        assert!(result.is_some());
        let mutated = result.unwrap();
        assert_eq!(mutated.len(), original.len());
        assert_ne!(mutated, original);
    }

    #[test]
    fn test_character_mutate_string_empty() {
        let mutator = CharacterMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let result = mutator.mutate_string(String::new(), &mut source, 1.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_character_mutate_bytes() {
        let mutator = CharacterMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let original = vec![1, 2, 3, 4, 5];
        let result = mutator.mutate_bytes(original.clone(), &mut source, 1.0);
        
        assert!(result.is_some());
        let mutated = result.unwrap();
        assert_eq!(mutated.len(), original.len());
        assert_ne!(mutated, original);
    }

    #[test]
    fn test_character_mutate_bytes_empty() {
        let mutator = CharacterMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let result = mutator.mutate_bytes(vec![], &mut source, 1.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_character_never_mutates_at_rate_0() {
        let mutator = CharacterMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        assert!(mutator.mutate_string("test".to_string(), &mut source, 0.0).is_none());
        assert!(mutator.mutate_bytes(vec![1, 2, 3], &mut source, 0.0).is_none());
    }
}
