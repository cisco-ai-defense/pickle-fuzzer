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

/// Mutates memo indices to reference different slots (potentially invalid).
#[derive(Debug)]
pub struct MemoIndexMutator {
    unsafe_mode: bool,
}

impl MemoIndexMutator {
    pub fn new(unsafe_mode: bool) -> Self {
        Self { unsafe_mode }
    }
}

impl Mutator for MemoIndexMutator {
    fn name(&self) -> &str {
        "memoindex"
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

        if self.unsafe_mode {
            // unsafe: any random index
            Some(source.gen_range(0, 1000))
        } else {
            // safe: small perturbations
            match source.gen_range(0, 3) {
                0 => Some(index.saturating_add(1)),
                1 => Some(index.saturating_sub(1)),
                _ => Some(index),
            }
        }
    }

    fn is_unsafe(&self) -> bool {
        self.unsafe_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GenerationSource;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_memoindex_name() {
        let mutator = MemoIndexMutator::new(false);
        assert_eq!(mutator.name(), "memoindex");
    }

    #[test]
    fn test_memoindex_safe_mode_small_perturbations() {
        let mutator = MemoIndexMutator::new(false);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let index = 10;
        for _ in 0..20 {
            if let Some(result) = mutator.mutate_memo_index(index, &mut source, 1.0) {
                // safe mode: only ±1 or same
                assert!(result == index || result == index + 1 || result == index - 1);
            }
        }
    }

    #[test]
    fn test_memoindex_unsafe_mode_any_value() {
        let mutator = MemoIndexMutator::new(true);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        let index = 10;
        let mut found_far_value = false;

        for _ in 0..20 {
            if let Some(result) = mutator.mutate_memo_index(index, &mut source, 1.0) {
                // unsafe mode: can be any value 0-999
                assert!(result < 1000);
                if result > index + 10 || result < index.saturating_sub(10) {
                    found_far_value = true;
                }
            }
        }

        assert!(
            found_far_value,
            "unsafe mode should produce values far from original"
        );
    }

    #[test]
    fn test_memoindex_never_mutates_at_rate_0() {
        let mutator = MemoIndexMutator::new(false);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);

        assert!(mutator.mutate_memo_index(5, &mut source, 0.0).is_none());
    }

    #[test]
    fn test_memoindex_is_unsafe() {
        let safe_mutator = MemoIndexMutator::new(false);
        let unsafe_mutator = MemoIndexMutator::new(true);

        assert!(!safe_mutator.is_unsafe());
        assert!(unsafe_mutator.is_unsafe());
    }
}
