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

/// Applies boundary value mutations (0, -1, MAX, MIN).
#[derive(Debug)]
pub struct BoundaryMutator;

impl Mutator for BoundaryMutator {
    fn name(&self) -> &str {
        "boundary"
    }

    fn mutate_int(&self, _value: i32, source: &mut GenerationSource, rate: f64) -> Option<i32> {
        if source.gen_f64() > rate {
            return None;
        }
        let boundaries = [0, -1, 1, i32::MAX, i32::MIN];
        Some(boundaries[source.gen_range(0, boundaries.len())])
    }

    fn mutate_long(&self, _value: i64, source: &mut GenerationSource, rate: f64) -> Option<i64> {
        if source.gen_f64() > rate {
            return None;
        }
        let boundaries = [0, -1, 1, i64::MAX, i64::MIN];
        Some(boundaries[source.gen_range(0, boundaries.len())])
    }

    fn mutate_float(&self, _value: f64, source: &mut GenerationSource, rate: f64) -> Option<f64> {
        if source.gen_f64() > rate {
            return None;
        }
        let boundaries = [
            0.0,
            -1.0,
            1.0,
            f64::MAX,
            f64::MIN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ];
        Some(boundaries[source.gen_range(0, boundaries.len())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GenerationSource;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_boundary_name() {
        let mutator = BoundaryMutator;
        assert_eq!(mutator.name(), "boundary");
    }

    #[test]
    fn test_boundary_int_returns_boundary_values() {
        let mutator = BoundaryMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let boundaries = [0, -1, 1, i32::MAX, i32::MIN];
        
        for _ in 0..20 {
            if let Some(result) = mutator.mutate_int(100, &mut source, 1.0) {
                assert!(boundaries.contains(&result));
            }
        }
    }

    #[test]
    fn test_boundary_int_never_mutates_at_rate_0() {
        let mutator = BoundaryMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let result = mutator.mutate_int(100, &mut source, 0.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_boundary_long_returns_boundary_values() {
        let mutator = BoundaryMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        let boundaries = [0, -1, 1, i64::MAX, i64::MIN];
        
        for _ in 0..20 {
            if let Some(result) = mutator.mutate_long(1000, &mut source, 1.0) {
                assert!(boundaries.contains(&result));
            }
        }
    }

    #[test]
    fn test_boundary_float_returns_boundary_values() {
        let mutator = BoundaryMutator;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut source = GenerationSource::Rand(&mut rng);
        
        for _ in 0..20 {
            if let Some(result) = mutator.mutate_float(1.5, &mut source, 1.0) {
                // check it's one of the expected boundary values
                assert!(
                    result == 0.0 || result == -1.0 || result == 1.0 ||
                    result == f64::MAX || result == f64::MIN ||
                    result.is_infinite() || result.is_nan()
                );
            }
        }
    }
}
