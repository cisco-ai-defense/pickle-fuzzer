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

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Pickle virtual machine (PVM) stack.
///
/// The stack holds objects during pickle generation, mirroring the behavior
/// of Python's pickle unpickler.
#[derive(Debug, Default, Clone)]
pub struct Stack {
    /// Internal stack storage
    pub inner: Vec<StackObjectRef>,
}

impl Stack {
    /// Create a new empty stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all items from the stack.
    pub fn reset(&mut self) {
        self.inner.clear();
    }

    /// Push a value onto the stack.
    pub fn push(&mut self, value: StackObject) {
        self.inner.push(StackObjectRef::new(value));
    }

    /// Pop a value from the stack.
    pub fn pop(&mut self) -> Option<StackObjectRef> {
        self.inner.pop()
    }

    /// Peek at the top value without removing it.
    pub fn peek(&self) -> Option<&StackObjectRef> {
        self.inner.last()
    }

    /// Get the current stack depth.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Reference-counted wrapper around a stack object with interior mutability.
///
/// This type enables shared ownership of stack objects while allowing mutation,
/// which is essential for:
/// - Recursive structures (lists/dicts that contain themselves)
/// - Shared references via pickle memo operations (BINPUT/BINGET)
/// - Efficient cloning without deep copying
///
/// Implements Hash and Eq by dereferencing and comparing the contained value,
/// not by pointer identity. This means two StackObjectRef instances pointing
/// to different allocations with the same value will compare as equal and hash
/// to the same value. This is necessary for using StackObjectRef as dict keys.
#[derive(Debug, Clone)]
pub struct StackObjectRef(pub Rc<RefCell<StackObject>>);

impl StackObjectRef {
    /// Create a new reference-counted stack object.
    pub fn new(obj: StackObject) -> Self {
        Self(Rc::new(RefCell::new(obj)))
    }

    /// Create from an existing Rc<RefCell<StackObject>>.
    #[allow(dead_code)]
    pub fn from_rc(rc: Rc<RefCell<StackObject>>) -> Self {
        Self(rc)
    }

    /// Borrow the inner object immutably.
    pub fn borrow(&self) -> std::cell::Ref<'_, StackObject> {
        self.0.borrow()
    }

    /// Borrow the inner object mutably.
    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, StackObject> {
        self.0.borrow_mut()
    }

    /// Get a reference to the inner Rc.
    #[allow(dead_code)]
    pub fn as_rc(&self) -> &Rc<RefCell<StackObject>> {
        &self.0
    }

    /// Check if this is a Global object and return its module and name.
    ///
    /// Also unwraps Callable and Instance objects to check their inner Global.
    #[allow(dead_code)]
    pub fn as_global(&self) -> Option<(String, String)> {
        match &*self.borrow() {
            StackObject::Global { module, name } => Some((module.clone(), name.clone())),
            StackObject::Callable(inner) => {
                // unwrap callable to check if it's a global
                match &*inner.borrow() {
                    StackObject::Global { module, name } => Some((module.clone(), name.clone())),
                    _ => None,
                }
            }
            StackObject::Instance(inst) => {
                // unwrap instance callable to check if it's a global
                match &*inst.callable.borrow() {
                    StackObject::Global { module, name } => Some((module.clone(), name.clone())),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Check if this is a Bytes object and return its contents.
    #[allow(dead_code)]
    pub fn as_bytes(&self) -> Option<Vec<u8>> {
        match &*self.borrow() {
            StackObject::Bytes(b) => Some(b.clone()),
            _ => None,
        }
    }
}

impl Hash for StackObjectRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Use pointer-based hashing to avoid infinite recursion with circular references
        // Hash the Rc pointer address as a usize to avoid any borrow issues
        let ptr = Rc::as_ptr(&self.0) as *const () as usize;
        ptr.hash(state);
    }
}

impl PartialEq for StackObjectRef {
    fn eq(&self, other: &Self) -> bool {
        // Use pointer equality to match our Hash implementation
        // This means two StackObjectRef are equal if they point to the same allocation
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for StackObjectRef {}

/// Runtime value on the pickle virtual machine stack.
///
/// Represents the various types of objects that can exist on the stack
/// during pickle generation, mirroring Python's object types.
#[derive(Debug, Clone)]
pub enum StackObject {
    // Scalar types
    /// Integer value
    Int(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Python None
    None,

    // String/bytes types
    /// Byte string (Python bytes)
    Bytes(Vec<u8>),
    /// Unicode string (Python str)
    String(String),
    /// Mutable byte array (Python bytearray)
    ByteArray(Vec<u8>),

    // Container types (can be recursive)
    /// List of objects
    List(Vec<StackObjectRef>),
    /// Tuple of objects
    Tuple(Vec<StackObjectRef>),
    /// Dictionary mapping keys to values
    Dict(HashMap<StackObjectRef, StackObjectRef>),
    /// Set of unique objects
    Set(HashSet<StackObjectRef>),
    /// Immutable set of unique objects
    FrozenSet(HashSet<StackObjectRef>),

    // Special VM markers
    /// Stack marker used by MARK opcode
    Mark,

    // Object references
    /// Global object reference (module.name)
    Global { module: String, name: String },
    /// Instance created via REDUCE/BUILD
    Instance(InstanceObject),
    /// Callable object wrapper
    Callable(StackObjectRef),

    // Extension objects
    /// Extension registry reference (placeholder for external objects)
    #[allow(dead_code)]
    Extension(u8),

    /// Generic placeholder for unimplemented types
    #[allow(dead_code)]
    Any,
}

impl Hash for StackObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            StackObject::Int(v) => v.hash(state),
            StackObject::Float(v) => {
                let bits = v.to_bits();
                bits.hash(state);
            }
            StackObject::Bool(v) => v.hash(state),
            StackObject::None => 0.hash(state),
            StackObject::Bytes(v) => v.hash(state),
            StackObject::String(v) => v.hash(state),
            StackObject::ByteArray(v) => v.hash(state),
            StackObject::List(v) => {
                for item in v {
                    item.borrow().hash(state);
                }
            }
            StackObject::Tuple(v) => {
                for item in v {
                    item.borrow().hash(state);
                }
            }
            StackObject::Dict(v) => {
                for (k, v) in v {
                    k.hash(state);
                    v.borrow().hash(state);
                }
            }
            StackObject::Set(v) => {
                for item in v {
                    item.hash(state);
                }
            }
            StackObject::FrozenSet(v) => {
                for item in v {
                    item.hash(state);
                }
            }
            StackObject::Mark => 0.hash(state),
            StackObject::Global { module, name } => {
                module.hash(state);
                name.hash(state);
            }
            StackObject::Instance(_) => 0.hash(state), // shallow hash
            StackObject::Any => 0.hash(state),
            StackObject::Extension(v) => v.hash(state),
            StackObject::Callable(inner) => inner.borrow().hash(state),
        }
    }
}

impl PartialEq for StackObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StackObject::Int(a), StackObject::Int(b)) => a == b,
            (StackObject::Float(a), StackObject::Float(b)) => a == b,
            (StackObject::Bool(a), StackObject::Bool(b)) => a == b,
            (StackObject::None, StackObject::None) => true,
            (StackObject::Bytes(a), StackObject::Bytes(b)) => a == b,
            (StackObject::String(a), StackObject::String(b)) => a == b,
            (StackObject::ByteArray(a), StackObject::ByteArray(b)) => a == b,
            (StackObject::List(a), StackObject::List(b)) => a == b,
            (StackObject::Tuple(a), StackObject::Tuple(b)) => a == b,
            (StackObject::Dict(a), StackObject::Dict(b)) => a == b,
            (StackObject::Set(a), StackObject::Set(b)) => a == b,
            (StackObject::FrozenSet(a), StackObject::FrozenSet(b)) => a == b,
            (StackObject::Mark, StackObject::Mark) => true,
            (
                StackObject::Global {
                    module: am,
                    name: an,
                },
                StackObject::Global {
                    module: bm,
                    name: bn,
                },
            ) => am == bm && an == bn,
            (StackObject::Instance(_), StackObject::Instance(_)) => true, // shallow compare
            (StackObject::Any, StackObject::Any) => true,
            (StackObject::Extension(a), StackObject::Extension(b)) => a == b,
            (StackObject::Callable(a), StackObject::Callable(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for StackObject {}

/// Represents a Python instance created via REDUCE/BUILD opcodes.
///
/// In pickle, instances are created by calling a callable with arguments,
/// typically a class constructor with initialization data.
#[derive(Debug, Clone)]
pub struct InstanceObject {
    /// The callable (class or function) being invoked
    #[allow(dead_code)]
    pub callable: StackObjectRef,
    /// The arguments or state passed to the callable
    pub args: StackObjectRef,
}
