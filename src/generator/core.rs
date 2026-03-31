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
use crate::stack::StackObject;

impl Generator {
    fn fixed_opcode_count(&self, use_frame: bool) -> usize {
        usize::from(!matches!(self.state.version, Version::V0 | Version::V1))
            + usize::from(use_frame)
            + 1
    }

    fn minimum_total_opcode_count(&self, use_frame: bool) -> usize {
        self.fixed_opcode_count(use_frame) + Self::cleanup_opcode_count_for_shape(Vec::new())
    }

    fn current_cleanup_opcode_count(&self) -> usize {
        let stack_shape = self
            .state
            .stack
            .inner
            .iter()
            .map(|obj| matches!(*obj.borrow(), StackObject::Mark))
            .collect();

        Self::cleanup_opcode_count_for_shape(stack_shape)
    }

    fn cleanup_opcode_count_after(&self, opcode: OpcodeKind) -> usize {
        let mut stack_shape: Vec<bool> = self
            .state
            .stack
            .inner
            .iter()
            .map(|obj| matches!(*obj.borrow(), StackObject::Mark))
            .collect();

        Self::apply_abstract_stack_effect(&mut stack_shape, opcode);
        Self::cleanup_opcode_count_for_shape(stack_shape)
    }

    fn cleanup_opcode_count_for_shape(mut stack_shape: Vec<bool>) -> usize {
        let mut cleanup_opcodes = 0;

        while let Some(mark_idx) = stack_shape.iter().rposition(|is_mark| *is_mark) {
            stack_shape.truncate(mark_idx);
            stack_shape.push(false);
            cleanup_opcodes += 1;
        }

        if stack_shape.is_empty() {
            cleanup_opcodes + 1
        } else {
            cleanup_opcodes + (stack_shape.len() / 2)
        }
    }

    fn apply_abstract_stack_effect(stack_shape: &mut Vec<bool>, opcode: OpcodeKind) {
        use OpcodeKind::*;

        match opcode {
            Pop => {
                stack_shape.pop();
            }
            Dup => {
                if !stack_shape.last().copied().unwrap_or(false) {
                    stack_shape.push(false);
                }
            }
            Mark => {
                stack_shape.push(true);
            }
            Append => {
                stack_shape.pop();
            }
            Appends | SetItems | AddItems => {
                while let Some(is_mark) = stack_shape.pop() {
                    if is_mark {
                        break;
                    }
                }
            }
            Tuple | List | FrozenSet | Dict | Inst | Obj => {
                while let Some(is_mark) = stack_shape.pop() {
                    if is_mark {
                        break;
                    }
                }
                stack_shape.push(false);
            }
            PopMark => {
                while let Some(is_mark) = stack_shape.pop() {
                    if is_mark {
                        break;
                    }
                }
            }
            Tuple1 | Memoize | BinPersID => {
                stack_shape.pop();
                stack_shape.push(false);
            }
            Tuple2 | Reduce | NewObj | Build | StackGlobal => {
                stack_shape.pop();
                stack_shape.pop();
                stack_shape.push(false);
            }
            Tuple3 | NewObjEx => {
                stack_shape.pop();
                stack_shape.pop();
                stack_shape.pop();
                stack_shape.push(false);
            }
            SetItem => {
                stack_shape.pop();
                stack_shape.pop();
            }
            Get | BinGet | LongBinGet | None | NewTrue | NewFalse | Int | Long | Long1 | Long4
            | BinInt | BinInt1 | BinInt2 | Float | BinFloat | String | BinString
            | ShortBinString | Unicode | ShortBinUnicode | BinUnicode | BinUnicode8
            | ShortBinBytes | BinBytes | BinBytes8 | ByteArray8 | EmptyList | EmptyDict
            | EmptyTuple | EmptySet | Global | PersID | Ext1 | Ext2 | Ext4 | NextBuffer => {
                stack_shape.push(false);
            }
            Put | BinPut | LongBinPut | Proto | ReadOnlyBuffer | Stop | Frame => {}
        }
    }

    pub(super) fn generate_internal(
        &mut self,
        source: &mut GenerationSource,
        target_total_override: Option<usize>,
        force_frame: Option<bool>,
    ) -> Result<Vec<u8>> {
        let (configured_min, configured_max) = self.normalized_opcode_range();
        let minimum_total_without_frame = self.minimum_total_opcode_count(false);
        if configured_max < minimum_total_without_frame {
            return Err(color_eyre::eyre::eyre!(
                "opcode budget {}..={} cannot fit a valid protocol {} pickle; minimum total is {} opcodes",
                configured_min,
                configured_max,
                self.state.version as u8,
                minimum_total_without_frame
            ));
        }

        let requested_budget = target_total_override.unwrap_or(configured_max);
        let use_frame = match force_frame {
            Some(force_frame) => {
                force_frame
                    && self.state.version >= Version::V4
                    && requested_budget >= self.minimum_total_opcode_count(true)
            }
            None => {
                self.state.version >= Version::V4
                    && requested_budget >= self.minimum_total_opcode_count(true)
                    && source.gen_bool()
            }
        };

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

        // choose a target total budget for the full emitted opcode stream
        let minimum_total = if target_total_override.is_some() {
            self.minimum_total_opcode_count(use_frame)
        } else {
            configured_min.max(self.minimum_total_opcode_count(use_frame))
        };
        let max_total = target_total_override
            .unwrap_or(configured_max)
            .max(minimum_total);
        if max_total < minimum_total {
            return Err(color_eyre::eyre::eyre!(
                "opcode budget {}..={} cannot fit a valid protocol {} pickle; minimum total is {} opcodes",
                configured_min,
                configured_max,
                self.state.version as u8,
                minimum_total
            ));
        }

        let target_total_opcodes = if target_total_override.is_some() || minimum_total == max_total
        {
            max_total
        } else {
            source.gen_range(minimum_total, max_total + 1)
        };
        let body_and_cleanup_budget =
            target_total_opcodes.saturating_sub(self.fixed_opcode_count(use_frame));
        let mut emitted_body_opcodes = 0;

        // generation phase - allow stack to grow and build complex structures
        loop {
            let cleanup_budget = self.current_cleanup_opcode_count();
            if emitted_body_opcodes + cleanup_budget >= body_and_cleanup_budget {
                break;
            }

            let valid_ops = self.get_valid_opcodes();
            if valid_ops.is_empty() {
                // no valid moves available, move to cleanup
                break;
            }

            let remaining_budget = body_and_cleanup_budget - emitted_body_opcodes;
            let budgeted_ops: Vec<_> = valid_ops
                .into_iter()
                .filter(|opcode| 1 + self.cleanup_opcode_count_after(*opcode) <= remaining_budget)
                .collect();
            if budgeted_ops.is_empty() {
                break;
            }

            let chosen = self.weighted_choice(budgeted_ops, source);
            self.emit_and_process(chosen, source)?;
            emitted_body_opcodes += 1;
        }

        // cleanup phase - reduce stack to exactly 1 item for STOP
        self.cleanup_for_stop();

        self.emit_opcode(OpcodeKind::Stop);

        // if we reserved space for FRAME, fill it in now with the correct size
        if let Some(pos) = frame_position {
            let frame_size = self.output.len().checked_sub(pos + 9).ok_or_else(|| {
                color_eyre::eyre::eyre!("FRAME size calculation underflow: output too small")
            })?;
            let frame_size = u64::try_from(frame_size).map_err(|_| {
                color_eyre::eyre::eyre!("FRAME size {} exceeds u64::MAX", frame_size)
            })?;

            self.output[pos] = OpcodeKind::Frame.as_u8();
            self.output[pos + 1..pos + 9].copy_from_slice(&frame_size.to_le_bytes());
        }

        Ok(self.output.clone())
    }
}
