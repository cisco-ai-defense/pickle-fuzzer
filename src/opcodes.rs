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

use phf::phf_map;

/// enumeration of all pickle opcodes we care about
/// source: https://github.com/python/cpython/blob/main/Lib/pickletools.py
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Opcode {
    // Integer opcodes
    Int(i32),       // 0x49
    BinInt(i32),    // 0x4a
    BinInt1(u8),    // 0x4b
    BinInt2(u16),   // 0x4d
    Long(i64),      // 0x4c
    Long1(Vec<u8>), // 0x8a
    Long4(Vec<u8>), // 0x8b

    // String/bytes opcodes
    String(String),          // 0x53
    BinString(Vec<u8>),      // 0x54
    ShortBinString(Vec<u8>), // 0x55
    BinBytes(Vec<u8>),       // 0x42
    ShortBinBytes(Vec<u8>),  // 0x43
    BinBytes8(Vec<u8>),      // 0x8e
    ByteArray8(Vec<u8>),     // 0x96
    NextBuffer,              // 0x97
    ReadOnlyBuffer,          // 0x98

    // None/boolean
    None,     // 0x4e
    NewTrue,  // 0x88
    NewFalse, // 0x89

    // Unicode
    Unicode(String),         // 0x56
    ShortBinUnicode(String), // 0x8c
    BinUnicode(String),      // 0x58
    BinUnicode8(String),     // 0x8d

    // Float
    Float(f64),    // 0x46
    BinFloat(f64), // 0x47

    // List/tuple/dict/set
    EmptyList,                                     // 0x5d
    Append,                                        // 0x61
    Appends,                                       // 0x65
    List(Vec<Opcode>),                             // 0x6c
    EmptyTuple,                                    // 0x29
    Tuple(Vec<Opcode>),                            // 0x74
    Tuple1(Box<Opcode>),                           // 0x85
    Tuple2(Box<Opcode>, Box<Opcode>),              // 0x86
    Tuple3(Box<Opcode>, Box<Opcode>, Box<Opcode>), // 0x87
    EmptyDict,                                     // 0x7d
    Dict(Vec<(Opcode, Opcode)>),                   // 0x64
    SetItem,                                       // 0x73
    SetItems,                                      // 0x75
    EmptySet,                                      // 0x8f
    AddItems,                                      // 0x90
    FrozenSet(Vec<Opcode>),                        // 0x91

    // Stack/memo opcodes
    Pop,             // 0x30
    Dup,             // 0x32
    Mark,            // 0x28
    PopMark,         // 0x31
    Get(u32),        // 0x67
    BinGet(u8),      // 0x68
    LongBinGet(u32), // 0x6a
    Put(u32),        // 0x70
    BinPut(u8),      // 0x71
    LongBinPut(u32), // 0x72
    Memoize,         // 0x94

    // Extension/global
    Ext1(u8),       // 0x82
    Ext2(u16),      // 0x83
    Ext4(u32),      // 0x84
    Global(String), // 0x63
    StackGlobal,    // 0x93
    Reduce,         // 0x52
    Build,          // 0x62
    Inst,           // 0x69
    Obj,            // 0x6f
    NewObj,         // 0x81
    NewObjEx,       // 0x92
    Proto(u8),      // 0x80
    Stop,           // 0x2e
    Frame(u64),     // 0x95
    PersID,         // 0x50
    BinPersID,      // 0x51
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OpcodeKind {
    Int,             // 0x49
    BinInt,          // 0x4a
    BinInt1,         // 0x4b
    BinInt2,         // 0x4d
    Long,            // 0x4c
    Long1,           // 0x8a
    Long4,           // 0x8b
    String,          // 0x53
    BinString,       // 0x54
    ShortBinString,  // 0x55
    BinBytes,        // 0x42
    ShortBinBytes,   // 0x43
    BinBytes8,       // 0x8e
    ByteArray8,      // 0x96
    NextBuffer,      // 0x97
    ReadOnlyBuffer,  // 0x98
    None,            // 0x4e
    NewTrue,         // 0x88
    NewFalse,        // 0x89
    Unicode,         // 0x56
    ShortBinUnicode, // 0x8c
    BinUnicode,      // 0x58
    BinUnicode8,     // 0x8d
    Float,           // 0x46
    BinFloat,        // 0x47
    EmptyList,       // 0x5d
    Append,          // 0x61
    Appends,         // 0x65
    List,            // 0x6c
    EmptyTuple,      // 0x29
    Tuple,           // 0x74
    Tuple1,          // 0x85
    Tuple2,          // 0x86
    Tuple3,          // 0x87
    EmptyDict,       // 0x7d
    Dict,            // 0x64
    SetItem,         // 0x73
    SetItems,        // 0x75
    EmptySet,        // 0x8f
    AddItems,        // 0x90
    FrozenSet,       // 0x91
    Pop,             // 0x30
    Dup,             // 0x32
    Mark,            // 0x28
    PopMark,         // 0x31
    Get,             // 0x67
    BinGet,          // 0x68
    LongBinGet,      // 0x6a
    Put,             // 0x70
    BinPut,          // 0x71
    LongBinPut,      // 0x72
    Memoize,         // 0x94
    Ext1,            // 0x82
    Ext2,            // 0x83
    Ext4,            // 0x84
    Global,          // 0x63
    StackGlobal,     // 0x93
    Reduce,          // 0x52
    Build,           // 0x62
    Inst,            // 0x69
    Obj,             // 0x6f
    NewObj,          // 0x81
    NewObjEx,        // 0x92
    Proto,           // 0x80
    Stop,            // 0x2e
    Frame,           // 0x95
    PersID,          // 0x50
    BinPersID,       // 0x51
}

impl OpcodeKind {
    pub fn as_u8(self) -> u8 {
        match self {
            OpcodeKind::Int => 0x49,
            OpcodeKind::BinInt => 0x4a,
            OpcodeKind::BinInt1 => 0x4b,
            OpcodeKind::BinInt2 => 0x4d,
            OpcodeKind::Long => 0x4c,
            OpcodeKind::Long1 => 0x8a,
            OpcodeKind::Long4 => 0x8b,
            OpcodeKind::String => 0x53,
            OpcodeKind::BinString => 0x54,
            OpcodeKind::ShortBinString => 0x55,
            OpcodeKind::BinBytes => 0x42,
            OpcodeKind::ShortBinBytes => 0x43,
            OpcodeKind::BinBytes8 => 0x8e,
            OpcodeKind::ByteArray8 => 0x96,
            OpcodeKind::NextBuffer => 0x97,
            OpcodeKind::ReadOnlyBuffer => 0x98,
            OpcodeKind::None => 0x4e,
            OpcodeKind::NewTrue => 0x88,
            OpcodeKind::NewFalse => 0x89,
            OpcodeKind::Unicode => 0x56,
            OpcodeKind::ShortBinUnicode => 0x8c,
            OpcodeKind::BinUnicode => 0x58,
            OpcodeKind::BinUnicode8 => 0x8d,
            OpcodeKind::Float => 0x46,
            OpcodeKind::BinFloat => 0x47,
            OpcodeKind::EmptyList => 0x5d,
            OpcodeKind::Append => 0x61,
            OpcodeKind::Appends => 0x65,
            OpcodeKind::List => 0x6c,
            OpcodeKind::EmptyTuple => 0x29,
            OpcodeKind::Tuple => 0x74,
            OpcodeKind::Tuple1 => 0x85,
            OpcodeKind::Tuple2 => 0x86,
            OpcodeKind::Tuple3 => 0x87,
            OpcodeKind::EmptyDict => 0x7d,
            OpcodeKind::Dict => 0x64,
            OpcodeKind::SetItem => 0x73,
            OpcodeKind::SetItems => 0x75,
            OpcodeKind::EmptySet => 0x8f,
            OpcodeKind::AddItems => 0x90,
            OpcodeKind::FrozenSet => 0x91,
            OpcodeKind::Pop => 0x30,
            OpcodeKind::Dup => 0x32,
            OpcodeKind::Mark => 0x28,
            OpcodeKind::PopMark => 0x31,
            OpcodeKind::Get => 0x67,
            OpcodeKind::BinGet => 0x68,
            OpcodeKind::LongBinGet => 0x6a,
            OpcodeKind::Put => 0x70,
            OpcodeKind::BinPut => 0x71,
            OpcodeKind::LongBinPut => 0x72,
            OpcodeKind::Memoize => 0x94,
            OpcodeKind::Ext1 => 0x82,
            OpcodeKind::Ext2 => 0x83,
            OpcodeKind::Ext4 => 0x84,
            OpcodeKind::Global => 0x63,
            OpcodeKind::StackGlobal => 0x93,
            OpcodeKind::Reduce => 0x52,
            OpcodeKind::Build => 0x62,
            OpcodeKind::Inst => 0x69,
            OpcodeKind::Obj => 0x6f,
            OpcodeKind::NewObj => 0x81,
            OpcodeKind::NewObjEx => 0x92,
            OpcodeKind::Proto => 0x80,
            OpcodeKind::Stop => 0x2e,
            OpcodeKind::Frame => 0x95,
            OpcodeKind::PersID => 0x50,
            OpcodeKind::BinPersID => 0x51,
        }
    }
}

pub static PICKLE_OPCODES: phf::Map<u8, &'static [OpcodeKind]> = phf_map! {
    0_u8 => &[
        OpcodeKind::Int,
        OpcodeKind::Long,
        OpcodeKind::String,
        OpcodeKind::None,
        OpcodeKind::Unicode,
        OpcodeKind::Float,
        OpcodeKind::Append,
        OpcodeKind::List,
        OpcodeKind::Tuple,
        OpcodeKind::Dict,
        OpcodeKind::SetItem,
        OpcodeKind::Pop,
        OpcodeKind::Dup,
        OpcodeKind::Mark,
        OpcodeKind::Get,
        OpcodeKind::Put,
        OpcodeKind::Global,
        OpcodeKind::Reduce,
        OpcodeKind::Build,
        OpcodeKind::Inst,
        OpcodeKind::Stop,
        OpcodeKind::PersID
    ],
    1_u8 => &[
        // V0 opcodes
        OpcodeKind::Int,
        OpcodeKind::Long,
        OpcodeKind::String,
        OpcodeKind::None,
        OpcodeKind::Unicode,
        OpcodeKind::Float,
        OpcodeKind::Append,
        OpcodeKind::List,
        OpcodeKind::Tuple,
        OpcodeKind::Dict,
        OpcodeKind::SetItem,
        OpcodeKind::Pop,
        OpcodeKind::Dup,
        OpcodeKind::Mark,
        OpcodeKind::Get,
        OpcodeKind::Put,
        OpcodeKind::Global,
        OpcodeKind::Reduce,
        OpcodeKind::Build,
        OpcodeKind::Inst,
        OpcodeKind::Stop,
        OpcodeKind::PersID,
        // plus additional V1 opcodes
        OpcodeKind::BinInt,
        OpcodeKind::BinInt1,
        OpcodeKind::BinInt2,
        OpcodeKind::BinString,
        OpcodeKind::ShortBinString,
        OpcodeKind::BinUnicode,
        OpcodeKind::BinFloat,
        OpcodeKind::EmptyList,
        OpcodeKind::Appends,
        OpcodeKind::EmptyTuple,
        OpcodeKind::EmptyDict,
        OpcodeKind::SetItems,
        OpcodeKind::PopMark,
        OpcodeKind::BinGet,
        OpcodeKind::LongBinGet,
        OpcodeKind::BinPut,
        OpcodeKind::LongBinPut,
        OpcodeKind::Obj,
        OpcodeKind::BinPersID,

    ],
    2_u8 => &[
        // V0 & V1 opcodes
        OpcodeKind::Int,
        OpcodeKind::Long,
        OpcodeKind::String,
        OpcodeKind::None,
        OpcodeKind::Unicode,
        OpcodeKind::Float,
        OpcodeKind::Append,
        OpcodeKind::List,
        OpcodeKind::Tuple,
        OpcodeKind::Dict,
        OpcodeKind::SetItem,
        OpcodeKind::Pop,
        OpcodeKind::Dup,
        OpcodeKind::Mark,
        OpcodeKind::Get,
        OpcodeKind::Put,
        OpcodeKind::Global,
        OpcodeKind::Reduce,
        OpcodeKind::Build,
        OpcodeKind::Inst,
        OpcodeKind::Stop,
        OpcodeKind::PersID,
        OpcodeKind::BinInt,
        OpcodeKind::BinInt1,
        OpcodeKind::BinInt2,
        OpcodeKind::BinString,
        OpcodeKind::ShortBinString,
        OpcodeKind::BinUnicode,
        OpcodeKind::BinFloat,
        OpcodeKind::EmptyList,
        OpcodeKind::Appends,
        OpcodeKind::EmptyTuple,
        OpcodeKind::EmptyDict,
        OpcodeKind::SetItems,
        OpcodeKind::PopMark,
        OpcodeKind::BinGet,
        OpcodeKind::LongBinGet,
        OpcodeKind::BinPut,
        OpcodeKind::LongBinPut,
        OpcodeKind::Obj,
        OpcodeKind::BinPersID,
        // plus additional V2 opcodes
        OpcodeKind::Long1,
        OpcodeKind::Long4,
        OpcodeKind::NewTrue,
        OpcodeKind::NewFalse,
        OpcodeKind::Tuple1,
        OpcodeKind::Tuple2,
        OpcodeKind::Tuple3,
        OpcodeKind::Ext1,
        OpcodeKind::Ext2,
        OpcodeKind::Ext4,
        OpcodeKind::NewObj,
        OpcodeKind::Proto,
    ],
    3_u8 => &[
        // V0, V1, V2 opcodes
        OpcodeKind::Int,
        OpcodeKind::Long,
        OpcodeKind::String,
        OpcodeKind::None,
        OpcodeKind::Unicode,
        OpcodeKind::Float,
        OpcodeKind::Append,
        OpcodeKind::List,
        OpcodeKind::Tuple,
        OpcodeKind::Dict,
        OpcodeKind::SetItem,
        OpcodeKind::Pop,
        OpcodeKind::Dup,
        OpcodeKind::Mark,
        OpcodeKind::Get,
        OpcodeKind::Put,
        OpcodeKind::Global,
        OpcodeKind::Reduce,
        OpcodeKind::Build,
        OpcodeKind::Inst,
        OpcodeKind::Stop,
        OpcodeKind::PersID,
        OpcodeKind::BinInt,
        OpcodeKind::BinInt1,
        OpcodeKind::BinInt2,
        OpcodeKind::BinString,
        OpcodeKind::ShortBinString,
        OpcodeKind::BinUnicode,
        OpcodeKind::BinFloat,
        OpcodeKind::EmptyList,
        OpcodeKind::Appends,
        OpcodeKind::EmptyTuple,
        OpcodeKind::EmptyDict,
        OpcodeKind::SetItems,
        OpcodeKind::PopMark,
        OpcodeKind::BinGet,
        OpcodeKind::LongBinGet,
        OpcodeKind::BinPut,
        OpcodeKind::LongBinPut,
        OpcodeKind::Obj,
        OpcodeKind::BinPersID,
        OpcodeKind::Long1,
        OpcodeKind::Long4,
        OpcodeKind::NewTrue,
        OpcodeKind::NewFalse,
        OpcodeKind::Tuple1,
        OpcodeKind::Tuple2,
        OpcodeKind::Tuple3,
        OpcodeKind::Ext1,
        OpcodeKind::Ext2,
        OpcodeKind::Ext4,
        OpcodeKind::NewObj,
        OpcodeKind::Proto,
        // plus additional V3 opcodes
        OpcodeKind::BinBytes,
        OpcodeKind::ShortBinBytes,
    ],
    4_u8 => &[
        // V0, V1, V2, V3 opcodes
        OpcodeKind::Int,
        OpcodeKind::Long,
        OpcodeKind::String,
        OpcodeKind::None,
        OpcodeKind::Unicode,
        OpcodeKind::Float,
        OpcodeKind::Append,
        OpcodeKind::List,
        OpcodeKind::Tuple,
        OpcodeKind::Dict,
        OpcodeKind::SetItem,
        OpcodeKind::Pop,
        OpcodeKind::Dup,
        OpcodeKind::Mark,
        OpcodeKind::Get,
        OpcodeKind::Put,
        OpcodeKind::Global,
        OpcodeKind::Reduce,
        OpcodeKind::Build,
        OpcodeKind::Inst,
        OpcodeKind::Stop,
        OpcodeKind::PersID,
        OpcodeKind::BinInt,
        OpcodeKind::BinInt1,
        OpcodeKind::BinInt2,
        OpcodeKind::BinString,
        OpcodeKind::ShortBinString,
        OpcodeKind::BinUnicode,
        OpcodeKind::BinFloat,
        OpcodeKind::EmptyList,
        OpcodeKind::Appends,
        OpcodeKind::EmptyTuple,
        OpcodeKind::EmptyDict,
        OpcodeKind::SetItems,
        OpcodeKind::PopMark,
        OpcodeKind::BinGet,
        OpcodeKind::LongBinGet,
        OpcodeKind::BinPut,
        OpcodeKind::LongBinPut,
        OpcodeKind::Obj,
        OpcodeKind::BinPersID,
        OpcodeKind::Long1,
        OpcodeKind::Long4,
        OpcodeKind::NewTrue,
        OpcodeKind::NewFalse,
        OpcodeKind::Tuple1,
        OpcodeKind::Tuple2,
        OpcodeKind::Tuple3,
        OpcodeKind::Ext1,
        OpcodeKind::Ext2,
        OpcodeKind::Ext4,
        OpcodeKind::NewObj,
        OpcodeKind::Proto,
        OpcodeKind::BinBytes,
        OpcodeKind::ShortBinBytes,
        // plus additional V4 opcodes
        OpcodeKind::BinBytes8,
        OpcodeKind::ShortBinUnicode,
        OpcodeKind::BinUnicode8,
        OpcodeKind::EmptySet,
        OpcodeKind::AddItems,
        OpcodeKind::FrozenSet,
        OpcodeKind::Memoize,
        OpcodeKind::StackGlobal,
        OpcodeKind::NewObjEx,
        OpcodeKind::Frame,
    ],
    5_u8 => &[
        // V0, V1, V2, V3, V4 opcodes
        OpcodeKind::Int,
        OpcodeKind::Long,
        OpcodeKind::String,
        OpcodeKind::None,
        OpcodeKind::Unicode,
        OpcodeKind::Float,
        OpcodeKind::Append,
        OpcodeKind::List,
        OpcodeKind::Tuple,
        OpcodeKind::Dict,
        OpcodeKind::SetItem,
        OpcodeKind::Pop,
        OpcodeKind::Dup,
        OpcodeKind::Mark,
        OpcodeKind::Get,
        OpcodeKind::Put,
        OpcodeKind::Global,
        OpcodeKind::Reduce,
        OpcodeKind::Build,
        OpcodeKind::Inst,
        OpcodeKind::Stop,
        OpcodeKind::PersID,
        OpcodeKind::BinInt,
        OpcodeKind::BinInt1,
        OpcodeKind::BinInt2,
        OpcodeKind::BinString,
        OpcodeKind::ShortBinString,
        OpcodeKind::BinUnicode,
        OpcodeKind::BinFloat,
        OpcodeKind::EmptyList,
        OpcodeKind::Appends,
        OpcodeKind::EmptyTuple,
        OpcodeKind::EmptyDict,
        OpcodeKind::SetItems,
        OpcodeKind::PopMark,
        OpcodeKind::BinGet,
        OpcodeKind::LongBinGet,
        OpcodeKind::BinPut,
        OpcodeKind::LongBinPut,
        OpcodeKind::Obj,
        OpcodeKind::BinPersID,
        OpcodeKind::Long1,
        OpcodeKind::Long4,
        OpcodeKind::NewTrue,
        OpcodeKind::NewFalse,
        OpcodeKind::Tuple1,
        OpcodeKind::Tuple2,
        OpcodeKind::Tuple3,
        OpcodeKind::Ext1,
        OpcodeKind::Ext2,
        OpcodeKind::Ext4,
        OpcodeKind::NewObj,
        OpcodeKind::Proto,
        OpcodeKind::BinBytes,
        OpcodeKind::ShortBinBytes,
        OpcodeKind::BinBytes8,
        OpcodeKind::ShortBinUnicode,
        OpcodeKind::BinUnicode8,
        OpcodeKind::EmptySet,
        OpcodeKind::AddItems,
        OpcodeKind::FrozenSet,
        OpcodeKind::Memoize,
        OpcodeKind::StackGlobal,
        OpcodeKind::NewObjEx,
        OpcodeKind::Frame,
        // plus additional V5 opcodes
        OpcodeKind::ByteArray8,
        OpcodeKind::NextBuffer,
        OpcodeKind::ReadOnlyBuffer,
    ],
};
