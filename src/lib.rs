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

//! A structure-aware test case generator for Python pickle parsers and validators.
//!
//! `pickle-fuzzer` generates complex, valid pickle bytecode across all protocol versions (0-5)
//! for use in fuzzing and testing pickle parsing implementations. It uses a stack-based
//! approach with proper opcode sequencing, stack/memo simulation, and protocol version
//! compliance to produce diverse test cases.
//!
//! # Examples
//!
//! ```no_run
//! use pickle_fuzzer::{Generator, Version};
//!
//! // Generate a single pickle file
//! let mut generator = Generator::new(Version::V4);
//! let pickle_bytes = generator.generate().unwrap();
//! std::fs::write("output.pkl", pickle_bytes).unwrap();
//! ```

mod cli;
mod generator;
pub mod mutators;
mod opcodes;
mod protocol;
#[cfg(feature = "python-bindings")]
mod python;
mod stack;
mod state;

pub use cli::Cli;
pub use generator::Generator;
pub use mutators::{EmissionSnapshot, Mutator, MutatorKind};
pub use protocol::Version;
