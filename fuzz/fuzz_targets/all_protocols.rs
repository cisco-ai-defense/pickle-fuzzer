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


use libfuzzer_sys::fuzz_target;
use pickle_fuzzer::{Generator, Version};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    // use first byte to select protocol
    let protocol = (data[0] % 6) as usize; // 0-5
    let version = Version::try_from(protocol).unwrap();
    
    let mut gen = Generator::new(version);
    
    // use remaining bytes for generation
    if let Ok(pickle) = gen.generate_from_arbitrary(&data[1..]) {
        assert!(!pickle.is_empty());
        assert_eq!(pickle[pickle.len() - 1], b'.');
        
        // protocol-specific validation
        match version {
            Version::V0 | Version::V1 => {
                // no PROTO opcode in v0/v1
                assert!(!pickle.starts_with(b"\x80"));
            }
            _ => {
                // v2+ should have PROTO opcode
                if pickle.len() > 2 {
                    assert_eq!(pickle[0], 0x80, "missing PROTO in v{}", protocol);
                }
            }
        }
    }
});
