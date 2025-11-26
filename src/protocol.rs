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

use phf::PhfHash;

/// Pickle protocol versions supported by cisco-ai-defense-pickle-fuzzer.
///
/// Python's pickle module supports protocols 0-5, each adding new features:
/// - V0: Original ASCII protocol (Python 1.x)
/// - V1: Binary protocol (Python 1.x)
/// - V2: New-style classes (Python 2.3+)
/// - V3: Bytes support (Python 3.0+)
/// - V4: Large data support (Python 3.4+)
/// - V5: Out-of-band data (Python 3.8+)
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
    /// Protocol 0: Original ASCII protocol
    V0,
    /// Protocol 1: Binary protocol
    V1,
    /// Protocol 2: New-style classes (default)
    #[default]
    V2,
    /// Protocol 3: Bytes support
    V3,
    /// Protocol 4: Large data support
    V4,
    /// Protocol 5: Out-of-band data
    V5,
}

impl TryFrom<usize> for Version {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Version::V0),
            1 => Ok(Version::V1),
            2 => Ok(Version::V2),
            3 => Ok(Version::V3),
            4 => Ok(Version::V4),
            5 => Ok(Version::V5),
            _ => Err(color_eyre::eyre::eyre!("protocol version must be 0-5")),
        }
    }
}

impl PhfHash for Version {
    fn phf_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (*self as u8).phf_hash(state);
    }
}
