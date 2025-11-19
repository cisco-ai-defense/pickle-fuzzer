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

use color_eyre::Result;

use super::source::{EntropySource, GenerationSource};
use super::Generator;
use super::Version;
use crate::opcodes::OpcodeKind;

impl Generator {
    pub(super) fn generate_internal(&mut self, source: &mut GenerationSource) -> Result<Vec<u8>> {
        // decide if we'll use FRAME (only for protocol >= 4, randomly chosen)
        let use_frame = self.state.version >= Version::V4 && source.gen_bool();

        self.emit_proto(source);

        // reserve space for FRAME if we're going to use it
        let frame_position = if use_frame {
            let pos = self.output.len();
            // reserve 9 bytes: 1 for opcode + 8 for frame size
            self.output.extend_from_slice(&[0u8; 9]);
            Some(pos)
        } else {
            None
        };

        // determine target complexity (random number of opcodes to emit)
        let range = self.max_opcodes.saturating_sub(self.min_opcodes);
        let target_opcodes = if range > 0 {
            self.min_opcodes + source.choose_index(range)
        } else {
            self.min_opcodes
        };

        // generation phase - allow stack to grow and build complex structures
        for _ in 0..target_opcodes {
            let valid_ops = self.get_valid_opcodes();
            if valid_ops.is_empty() {
                // no valid moves available, move to cleanup
                break;
            }
            let chosen = self.weighted_choice(valid_ops, source);
            self.emit_and_process(chosen, source)?;
        }

        // cleanup phase - reduce stack to exactly 1 item for STOP
        self.cleanup_for_stop();

        self.emit_opcode(OpcodeKind::Stop);

        // if we reserved space for FRAME, fill it in now with the correct size
        if let Some(pos) = frame_position {
            let frame_size = self.output.len().checked_sub(pos + 9).ok_or_else(|| {
                color_eyre::eyre::eyre!("FRAME size calculation underflow: output too small")
            })?;

            if frame_size > u64::MAX as usize {
                return Err(color_eyre::eyre::eyre!(
                    "FRAME size {} exceeds u64::MAX",
                    frame_size
                ));
            }

            self.output[pos] = OpcodeKind::Frame.as_u8();
            self.output[pos + 1..pos + 9].copy_from_slice(&(frame_size as u64).to_le_bytes());
        }

        Ok(self.output.clone())
    }
}
