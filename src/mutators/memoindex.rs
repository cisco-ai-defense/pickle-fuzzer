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
