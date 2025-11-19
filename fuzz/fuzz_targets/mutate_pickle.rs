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
use pickle_whip::{Generator, Version};
use pickle_whip::mutators::{
    BitFlipMutator, BoundaryMutator, OffByOneMutator,
    StringLengthMutator, CharacterMutator, MemoIndexMutator,
};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    
    // use first byte for configuration
    let config = data[0];
    let mutation_rate = (config as f64) / 255.0; // 0.0 to 1.0
    
    let mut gen = Generator::new(Version::V3)
        .with_mutation_rate(mutation_rate);
    
    // add mutators based on bits in second byte
    let mutator_flags = data[1];
    if mutator_flags & 0x01 != 0 {
        gen = gen.with_mutator(Box::new(BitFlipMutator));
    }
    if mutator_flags & 0x02 != 0 {
        gen = gen.with_mutator(Box::new(BoundaryMutator));
    }
    if mutator_flags & 0x04 != 0 {
        gen = gen.with_mutator(Box::new(OffByOneMutator));
    }
    if mutator_flags & 0x08 != 0 {
        gen = gen.with_mutator(Box::new(StringLengthMutator));
    }
    if mutator_flags & 0x10 != 0 {
        gen = gen.with_mutator(Box::new(CharacterMutator));
    }
    if mutator_flags & 0x20 != 0 {
        gen = gen.with_mutator(Box::new(MemoIndexMutator::new(false)));
    }
    
    // generate with mutations
    if let Ok(pickle) = gen.generate_from_arbitrary(&data[2..]) {
        assert!(!pickle.is_empty());
        // mutations may produce invalid pickles, but shouldn't crash
    }
});
