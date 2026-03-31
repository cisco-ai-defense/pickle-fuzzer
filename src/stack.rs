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

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

thread_local! {
    static DROP_CYCLE_CLEANUP_ACTIVE: Cell<bool> = const { Cell::new(false) };
}

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
/// Hash and Eq are based on pointer identity, not the contained value.
/// Cloned references to the same allocation compare equal and hash the same,
/// while distinct allocations remain distinct even if their contents match.
/// This avoids recursive hashing and equality on cyclic structures.
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

    fn ptr_addr(&self) -> usize {
        Rc::as_ptr(&self.0) as *const () as usize
    }

    fn has_child_refs(&self) -> bool {
        let Ok(obj) = self.0.try_borrow() else {
            return false;
        };

        match &*obj {
            StackObject::List(items) | StackObject::Tuple(items) => !items.is_empty(),
            StackObject::Dict(items) => !items.is_empty(),
            StackObject::Set(items) | StackObject::FrozenSet(items) => !items.is_empty(),
            StackObject::Callable(_) | StackObject::Instance(_) => true,
            _ => false,
        }
    }

    fn child_refs(&self) -> Vec<StackObjectRef> {
        let Ok(obj) = self.0.try_borrow() else {
            return Vec::new();
        };

        match &*obj {
            StackObject::List(items) | StackObject::Tuple(items) => items.clone(),
            StackObject::Dict(items) => {
                let mut children = Vec::with_capacity(items.len() * 2);
                for (key, value) in items {
                    children.push(key.clone());
                    children.push(value.clone());
                }
                children
            }
            StackObject::Set(items) | StackObject::FrozenSet(items) => {
                items.iter().cloned().collect()
            }
            StackObject::Callable(inner) => vec![inner.clone()],
            StackObject::Instance(instance) => {
                vec![instance.callable.clone(), instance.args.clone()]
            }
            _ => Vec::new(),
        }
    }

    fn clear_child_refs(&self) {
        let Ok(mut obj) = self.0.try_borrow_mut() else {
            return;
        };

        match &mut *obj {
            StackObject::List(items) | StackObject::Tuple(items) => items.clear(),
            StackObject::Dict(items) => items.clear(),
            StackObject::Set(items) | StackObject::FrozenSet(items) => items.clear(),
            StackObject::Callable(_) | StackObject::Instance(_) => {
                *obj = StackObject::Any;
            }
            _ => {}
        }
    }

    fn cleanup_isolated_subgraph(&self) {
        let root_ptr = self.ptr_addr();
        let mut pending = vec![Self(self.0.clone())];
        let mut nodes = HashMap::new();
        let mut internal_incoming = HashMap::new();

        while let Some(node) = pending.pop() {
            let ptr = node.ptr_addr();
            if nodes.contains_key(&ptr) {
                continue;
            }

            let children = node.child_refs();
            for child in &children {
                *internal_incoming.entry(child.ptr_addr()).or_insert(0usize) += 1;
            }
            pending.extend(children);
            nodes.insert(ptr, node);
        }

        let isolated = nodes.iter().all(|(ptr, node)| {
            let observed_strong = Rc::strong_count(&node.0).saturating_sub(1);
            let internal_refs = internal_incoming.get(ptr).copied().unwrap_or_default();
            if *ptr == root_ptr {
                observed_strong == internal_refs + 1
            } else {
                observed_strong == internal_refs
            }
        });

        if !isolated {
            return;
        }

        for node in nodes.values() {
            node.clear_child_refs();
        }
        drop(nodes);
    }
}

impl Drop for StackObjectRef {
    fn drop(&mut self) {
        if DROP_CYCLE_CLEANUP_ACTIVE.with(|active| active.get()) || !self.has_child_refs() {
            return;
        }

        DROP_CYCLE_CLEANUP_ACTIVE.with(|active| {
            active.set(true);
            self.cleanup_isolated_subgraph();
            active.set(false);
        });
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn stack_object_ref_uses_pointer_identity() {
        let first = StackObjectRef::new(StackObject::Int(7));
        let same = first.clone();
        let second = StackObjectRef::new(StackObject::Int(7));

        assert_eq!(first, same);
        assert_ne!(first, second);
    }

    #[test]
    fn dropping_last_external_ref_breaks_self_cycle() {
        let list = StackObjectRef::new(StackObject::List(Vec::new()));
        let weak = Rc::downgrade(&list.0);

        if let StackObject::List(items) = &mut *list.borrow_mut() {
            items.push(list.clone());
        }

        drop(list);

        assert!(weak.upgrade().is_none());
    }
}
