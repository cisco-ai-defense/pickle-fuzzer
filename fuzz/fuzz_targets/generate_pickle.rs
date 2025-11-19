#![no_main]
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

mod common;

use libfuzzer_sys::fuzz_target;
use pickle_whip::{Generator, Version};
use crate::common::validate_with_pickletools;

fuzz_target!(|data: &[u8]| {
    // skip empty input
    if data.is_empty() {
        return;
    }
    
    // create generator for protocol 3 (most common)
    let mut gen = Generator::new(Version::V3);
    
    // generate pickle from fuzzer bytes
    if let Ok(pickle) = gen.generate_from_arbitrary(data) {
        // basic validation: must end with STOP opcode
        assert!(!pickle.is_empty(), "generated empty pickle");
        assert_eq!(pickle[pickle.len() - 1], b'.', "missing STOP opcode");
    }
});
