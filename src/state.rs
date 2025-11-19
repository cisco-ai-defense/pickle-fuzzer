// SPDX-License-Identifier: Apache-2.0
//
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

use std::collections::HashMap;

use crate::stack::{Stack, StackObjectRef};

use super::protocol::Version;

/// Generator state tracking the pickle virtual machine (PVM) state.
///
/// Maintains the stack, memo table, protocol version, and other state
/// needed during pickle generation.
#[derive(Default, Debug, Clone)]
pub struct State {
    /// Current protocol version being generated
    pub version: Version,

    /// Whether the PROTO opcode has been emitted (can only emit once)
    pub proto_emitted: bool,

    /// Current PVM stack
    pub stack: Stack,

    /// Memoization table mapping indices to stack objects
    pub memo: HashMap<usize, StackObjectRef>,
}

impl State {
    /// Create a new state with the specified protocol version.
    pub fn new(version: Version) -> Self {
        Self {
            version,
            stack: Stack::new(),
            ..Default::default()
        }
    }

    /// Reset the state for generating a new pickle.
    ///
    /// Clears the stack, memo table, and resets flags.
    pub fn reset(&mut self) {
        self.proto_emitted = false;
        self.memo.clear();
        self.stack.reset();
    }
}
