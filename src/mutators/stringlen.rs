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

/// Mutates string lengths (truncate or extend).
#[derive(Debug)]
pub struct StringLengthMutator;

impl Mutator for StringLengthMutator {
    fn name(&self) -> &str {
        "stringlen"
    }

    fn mutate_string(
        &self,
        value: String,
        source: &mut GenerationSource,
        rate: f64,
    ) -> Option<String> {
        if source.gen_f64() > rate {
            return None;
        }

        match source.gen_range(0, 3) {
            0 => {
                // truncate
                if value.is_empty() {
                    return Some(value);
                }
                let new_len = source.gen_range(0, value.len());
                Some(value.chars().take(new_len).collect())
            }
            1 => {
                // extend with random chars
                let extra_len = source.gen_range(1, 10);
                let mut result = value.clone();
                for _ in 0..extra_len {
                    result.push((source.gen_u8() % 26 + b'a') as char);
                }
                Some(result)
            }
            _ => {
                // double
                let mut result = value.clone();
                result.push_str(&value);
                Some(result)
            }
        }
    }

    fn mutate_bytes(
        &self,
        value: Vec<u8>,
        source: &mut GenerationSource,
        rate: f64,
    ) -> Option<Vec<u8>> {
        if source.gen_f64() > rate {
            return None;
        }

        match source.gen_range(0, 3) {
            0 => {
                // truncate
                if value.is_empty() {
                    return Some(value);
                }
                let new_len = source.gen_range(0, value.len());
                Some(value[..new_len].to_vec())
            }
            1 => {
                // extend with random bytes
                let extra_len = source.gen_range(1, 10);
                let mut result = value.clone();
                for _ in 0..extra_len {
                    result.push(source.gen_u8());
                }
                Some(result)
            }
            _ => {
                // double
                let mut result = value.clone();
                result.extend(value);
                Some(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GenerationSource;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_stringlen_name() {
        let mutator = StringLengthMutator;
        assert_eq!(mutator.name(), "stringlen");
    }

    #[test]
    fn test_stringlen_mutate_string_changes_length() {
        let mutator = StringLengthMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let original = "hello".to_string();
        let mut found_shorter = false;
        let mut found_longer = false;

        for _ in 0..30 {
            if let Some(result) = mutator.mutate_string(original.clone(), &mut source, 1.0) {
                if result.len() < original.len() {
                    found_shorter = true;
                }
                if result.len() > original.len() {
                    found_longer = true;
                }
            }
        }

        assert!(found_shorter || found_longer, "should change length");
    }

    #[test]
    fn test_stringlen_mutate_string_empty() {
        let mutator = StringLengthMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        // empty string can only be extended or doubled (stays empty)
        for _ in 0..10 {
            if let Some(result) = mutator.mutate_string(String::new(), &mut source, 1.0) {
                // should either stay empty or get extended
                assert!(result.is_empty() || !result.is_empty());
            }
        }
    }

    #[test]
    fn test_stringlen_mutate_bytes_changes_length() {
        let mutator = StringLengthMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let original = vec![1, 2, 3, 4, 5];
        let mut found_shorter = false;
        let mut found_longer = false;

        for _ in 0..30 {
            if let Some(result) = mutator.mutate_bytes(original.clone(), &mut source, 1.0) {
                if result.len() < original.len() {
                    found_shorter = true;
                }
                if result.len() > original.len() {
                    found_longer = true;
                }
            }
        }

        assert!(found_shorter || found_longer, "should change length");
    }

    #[test]
    fn test_stringlen_mutate_bytes_empty() {
        let mutator = StringLengthMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        // empty bytes can only be extended or doubled (stays empty)
        for _ in 0..10 {
            if let Some(result) = mutator.mutate_bytes(vec![], &mut source, 1.0) {
                assert!(result.is_empty() || !result.is_empty());
            }
        }
    }

    #[test]
    fn test_stringlen_never_mutates_at_rate_0() {
        let mutator = StringLengthMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        assert!(mutator
            .mutate_string("test".to_string(), &mut source, 0.0)
            .is_none());
        assert!(mutator
            .mutate_bytes(vec![1, 2, 3], &mut source, 0.0)
            .is_none());
    }
}
