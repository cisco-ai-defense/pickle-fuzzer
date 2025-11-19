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
