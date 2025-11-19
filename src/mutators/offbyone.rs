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
