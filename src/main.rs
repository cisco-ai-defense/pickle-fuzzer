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

use clap::Parser;
use color_eyre::Result;
use pickle_whip::{Cli, Generator, Version};
use rand::Rng;
use rayon::prelude::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    // Expand "all" meta-option and create mutators
    let mutator_kinds: Vec<pickle_whip::MutatorKind> =
        if args.mutators.contains(&pickle_whip::MutatorKind::All) {
            // If "all" is specified, use all mutators
            pickle_whip::MutatorKind::all_mutators()
        } else {
            // Otherwise use the specified mutators
            args.mutators.clone()
        };

    let mutators: Vec<Box<dyn pickle_whip::Mutator>> = mutator_kinds
        .iter()
        .map(|kind| kind.create(args.unsafe_mutations))
        .collect();

    if let Some(file) = args.file {
        // single file mode - generate one pickle
        let version = if let Some(protocol) = args.protocol {
            // if --protocol was used, prefer that
            Version::try_from(protocol).unwrap_or(Version::V3)
        } else if let Some(seed) = args.seed {
            // otoh, if --seed need to use that to deterministically select protocol version
            // so we don't need to dork with seeding the rand::rng
            Version::try_from((seed % 6) as usize).unwrap_or(Version::V3)
        } else {
            Version::try_from(rand::rng().random_range(0..=5)).unwrap_or(Version::V3)
        };

        let mut generator =
            Generator::new(version).with_opcode_range(args.min_opcodes, args.max_opcodes);

        if let Some(seed) = args.seed {
            generator = generator.with_seed(seed);
        }

        if !mutators.is_empty() {
            generator = generator
                .with_mutators(mutators)
                .with_mutation_rate(args.mutation_rate)
                .with_unsafe_mutations(args.unsafe_mutations);
        }

        let bytecode = generator.generate()?;
        std::fs::write(&file, &bytecode)?;
        println!("Generated {} bytes to {:?}", bytecode.len(), file);
    } else if let Some(dir) = args.dir {
        if !dir.exists() {
            std::fs::create_dir(&dir)?;
        }

        // Collect errors from parallel generation
        let seed = args.seed;
        let protocol = args.protocol;
        let min_opcodes = args.min_opcodes;
        let max_opcodes = args.max_opcodes;
        let mutation_rate = args.mutation_rate;
        let unsafe_mutations = args.unsafe_mutations;
        let mutator_kinds_for_batch = mutator_kinds.clone();

        let errors: Vec<_> = (0..args.samples)
            .into_par_iter()
            .filter_map(|idx| {
                let version = if let Some(proto) = protocol {
                    // same version selection logic as what's used above
                    Version::try_from(proto).unwrap_or(Version::V3)
                } else if let Some(seed) = seed {
                    Version::try_from((seed % 6) as usize).unwrap_or(Version::V3)
                } else {
                    Version::try_from(rand::rng().random_range(0..=5)).unwrap_or(Version::V3)
                };

                let mut generator =
                    Generator::new(version).with_opcode_range(min_opcodes, max_opcodes);

                if let Some(s) = seed {
                    generator = generator.with_seed(s);
                }

                // Create mutators for this thread
                if !mutator_kinds_for_batch.is_empty() {
                    let thread_mutators: Vec<Box<dyn pickle_whip::Mutator>> =
                        mutator_kinds_for_batch
                            .iter()
                            .map(|kind| kind.create(unsafe_mutations))
                            .collect();
                    generator = generator
                        .with_mutators(thread_mutators)
                        .with_mutation_rate(mutation_rate)
                        .with_unsafe_mutations(unsafe_mutations);
                }

                let bytecode = match generator.generate() {
                    Ok(b) => b,
                    Err(e) => return Some((idx, format!("generation error: {}", e))),
                };

                let mut file_path = dir.clone();
                file_path.push(format!("{idx}.pkl"));

                if let Err(e) = std::fs::write(&file_path, &bytecode) {
                    return Some((idx, format!("write error: {}", e)));
                }

                None
            })
            .collect();

        if !errors.is_empty() {
            eprintln!("Encountered {} errors during generation:", errors.len());
            for (idx, error) in errors.iter().take(10) {
                eprintln!("  Sample {}: {}", idx, error);
            }
            if errors.len() > 10 {
                eprintln!("  ... and {} more errors", errors.len() - 10);
            }
            return Err(color_eyre::eyre::eyre!(
                "Failed to generate {} out of {} samples",
                errors.len(),
                args.samples
            ));
        }

        println!(
            "Successfully generated {} pickle files to {:?}",
            args.samples, dir
        );
    } else {
        unreachable!("clap should ensure either file or dir is provided");
    }

    Ok(())
}
