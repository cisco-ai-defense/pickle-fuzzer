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

use std::path::PathBuf;

use clap::Parser;

/// Parse and validate a pickle protocol version string.
///
/// Accepts version numbers 0-5 (inclusive).
fn parse_version(s: &str) -> Result<usize, String> {
    let v = s
        .parse::<usize>()
        .map_err(|_| format!("invalid version: {}", s))?;
    if v <= 5 {
        Ok(v)
    } else {
        Err(format!("version must be 0-5, got {}", v))
    }
}

/// Command-line interface for pickle-whip.
///
/// Supports two modes:
/// - Single file mode: Generate one pickle file
/// - Batch mode: Generate multiple pickle files in a directory
#[derive(Parser, Debug)]
#[command(name = "pickle-whip")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// path to a single pickle output file
    #[arg(
        value_name = "FILE",
        conflicts_with = "dir",
        required_unless_present = "dir"
    )]
    pub file: Option<PathBuf>,

    /// directory where generated samples will be written.
    /// conflicts with FILE argument
    #[arg(
        long = "dir",
        short = 'd',
        value_name = "DIR",
        conflicts_with = "file",
        required_unless_present = "file"
    )]
    pub dir: Option<PathBuf>,

    /// pickle protocol version (0-5)
    #[arg(short, long, value_name="PROTOCOL", value_parser = parse_version)]
    pub protocol: Option<usize>,

    /// number of pickle samples to generate in batch mode
    #[arg(short, long, default_value_t = 10_000, requires = "dir")]
    pub samples: usize,

    /// seed for random number generator (for reproducible, byte-identical generation)
    #[arg(long)]
    pub seed: Option<u64>,

    /// minimum number of opcodes to generate
    #[arg(long, default_value_t = 60)]
    pub min_opcodes: usize,

    /// maximum number of opcodes to generate
    #[arg(long, default_value_t = 300)]
    pub max_opcodes: usize,

    /// enable specific mutators (can be specified multiple times)
    #[arg(long = "mutators", value_name = "MUTATOR", num_args = 1..)]
    pub mutators: Vec<crate::mutators::MutatorKind>,

    /// mutation rate (0.0-1.0, probability of applying mutation)
    #[arg(long, default_value_t = 0.1)]
    pub mutation_rate: f64,

    /// allow unsafe mutations that may produce invalid pickles
    #[arg(long)]
    pub unsafe_mutations: bool,
}

impl Cli {
    /// Check if running in batch mode (generating multiple files).
    pub fn is_batch_mode(&self) -> bool {
        self.dir.is_some()
    }

    /// Check if running in single-file mode (generating one file).
    pub fn is_single_file_mode(&self) -> bool {
        self.file.is_some()
    }
}
