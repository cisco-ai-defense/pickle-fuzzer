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

/// Applies off-by-one mutations (value ± 1).
#[derive(Debug)]
pub struct OffByOneMutator;

impl Mutator for OffByOneMutator {
    fn name(&self) -> &str {
        "offbyone"
    }

    fn mutate_int(&self, value: i32, source: &mut GenerationSource, rate: f64) -> Option<i32> {
        if source.gen_f64() > rate {
            return None;
        }
        if source.gen_bool() {
            Some(value.wrapping_add(1))
        } else {
            Some(value.wrapping_sub(1))
        }
    }

    fn mutate_long(&self, value: i64, source: &mut GenerationSource, rate: f64) -> Option<i64> {
        if source.gen_f64() > rate {
            return None;
        }
        if source.gen_bool() {
            Some(value.wrapping_add(1))
        } else {
            Some(value.wrapping_sub(1))
        }
    }

    fn mutate_memo_index(
        &self,
        index: usize,
        source: &mut GenerationSource,
        rate: f64,
    ) -> Option<usize> {
        if source.gen_f64() > rate {
            return None;
        }
        if source.gen_bool() {
            Some(index.saturating_add(1))
        } else {
            Some(index.saturating_sub(1))
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
    fn test_offbyone_name() {
        let mutator = OffByOneMutator;
        assert_eq!(mutator.name(), "offbyone");
    }

    #[test]
    fn test_offbyone_int_adds_or_subtracts_one() {
        let mutator = OffByOneMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let value = 100;
        for _ in 0..10 {
            if let Some(result) = mutator.mutate_int(value, &mut source, 1.0) {
                assert!(result == value + 1 || result == value - 1);
            }
        }
    }

    #[test]
    fn test_offbyone_int_wrapping() {
        let mutator = OffByOneMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        // test wrapping at MAX
        if let Some(result) = mutator.mutate_int(i32::MAX, &mut source, 1.0) {
            assert!(result == i32::MIN || result == i32::MAX - 1);
        }
    }

    #[test]
    fn test_offbyone_long_adds_or_subtracts_one() {
        let mutator = OffByOneMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let value = 1000_i64;
        for _ in 0..10 {
            if let Some(result) = mutator.mutate_long(value, &mut source, 1.0) {
                assert!(result == value + 1 || result == value - 1);
            }
        }
    }

    #[test]
    fn test_offbyone_memo_index() {
        let mutator = OffByOneMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let index = 5;
        for _ in 0..10 {
            if let Some(result) = mutator.mutate_memo_index(index, &mut source, 1.0) {
                assert!(result == index + 1 || result == index - 1);
            }
        }
    }

    #[test]
    fn test_offbyone_memo_index_saturating_at_zero() {
        let mutator = OffByOneMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        // at index 0, subtracting should saturate to 0
        for _ in 0..10 {
            if let Some(result) = mutator.mutate_memo_index(0, &mut source, 1.0) {
                assert!(result == 0 || result == 1);
            }
        }
    }

    #[test]
    fn test_offbyone_never_mutates_at_rate_0() {
        let mutator = OffByOneMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        assert!(mutator.mutate_int(100, &mut source, 0.0).is_none());
        assert!(mutator.mutate_long(100, &mut source, 0.0).is_none());
        assert!(mutator.mutate_memo_index(5, &mut source, 0.0).is_none());
    }
}
