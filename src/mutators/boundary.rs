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
