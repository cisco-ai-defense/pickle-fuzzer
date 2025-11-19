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
