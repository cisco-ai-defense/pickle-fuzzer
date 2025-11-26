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

/// Applies bit flips to integer arguments.
#[derive(Debug)]
pub struct BitFlipMutator;

impl Mutator for BitFlipMutator {
    fn name(&self) -> &str {
        "bitflip"
    }

    fn mutate_int(&self, value: i32, source: &mut GenerationSource, rate: f64) -> Option<i32> {
        if source.gen_f64() > rate {
            return None;
        }
        let bit_pos = source.gen_range(0, 32);
        Some(value ^ (1 << bit_pos))
    }

    fn mutate_long(&self, value: i64, source: &mut GenerationSource, rate: f64) -> Option<i64> {
        if source.gen_f64() > rate {
            return None;
        }
        let bit_pos = source.gen_range(0, 64);
        Some(value ^ (1 << bit_pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GenerationSource;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_bitflip_name() {
        let mutator = BitFlipMutator;
        assert_eq!(mutator.name(), "bitflip");
    }

    #[test]
    fn test_bitflip_int_always_mutates_at_rate_1() {
        let mutator = BitFlipMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let original = 0b1010_1010;
        let result = mutator.mutate_int(original, &mut source, 1.0);
        
        assert!(result.is_some());
        let mutated = result.unwrap();
        assert_ne!(mutated, original);
        
        // verify exactly one bit flipped
        let diff = original ^ mutated;
        assert_eq!(diff.count_ones(), 1);
    }

    #[test]
    fn test_bitflip_int_never_mutates_at_rate_0() {
        let mutator = BitFlipMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let result = mutator.mutate_int(100, &mut source, 0.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_bitflip_long_always_mutates_at_rate_1() {
        let mutator = BitFlipMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let original = 0x1234_5678_9ABC_DEF0_i64;
        let result = mutator.mutate_long(original, &mut source, 1.0);
        
        assert!(result.is_some());
        let mutated = result.unwrap();
        assert_ne!(mutated, original);
        
        // verify exactly one bit flipped
        let diff = original ^ mutated;
        assert_eq!(diff.count_ones(), 1);
    }

    #[test]
    fn test_bitflip_long_never_mutates_at_rate_0() {
        let mutator = BitFlipMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let result = mutator.mutate_long(1000, &mut source, 0.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_bitflip_int_zero() {
        let mutator = BitFlipMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let result = mutator.mutate_int(0, &mut source, 1.0);
        assert!(result.is_some());
        
        // flipping any bit in 0 gives a power of 2
        let mutated = result.unwrap();
        assert_ne!(mutated, 0);
        assert_eq!(mutated.count_ones(), 1);
    }

    #[test]
    fn test_bitflip_produces_different_results() {
        let mutator = BitFlipMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let value = 0xFF00;
        let mut results = std::collections::HashSet::new();
        
        // collect multiple mutations
        for _ in 0..20 {
            if let Some(mutated) = mutator.mutate_int(value, &mut source, 1.0) {
                results.insert(mutated);
            }
        }
        
        // should produce multiple different results
        assert!(results.len() > 1);
    }
}
