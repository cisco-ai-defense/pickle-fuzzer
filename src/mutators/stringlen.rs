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
