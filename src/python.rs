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

//! Python bindings for pickle-fuzzer generator.
//!
//! This module provides a Python interface to the Rust-based pickle generator
//! using PyO3. It allows Python code to generate pickle bytecode with the same
//! capabilities as the Rust API.

use crate::{Generator, Version};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyclass(name = "Generator", unsendable)]
struct PyGenerator {
    inner: Generator,
}

impl PyGenerator {
    fn effective_bufsize(&self, max_size: Option<usize>) -> Option<usize> {
        match (self.inner.bufsize, max_size) {
            (Some(existing), Some(max_size)) => Some(existing.min(max_size)),
            (None, Some(max_size)) => Some(max_size),
            (existing, None) => existing,
        }
    }
}

#[pymethods]
impl PyGenerator {
    #[new]
    #[pyo3(signature = (protocol=3, seed=None, allow_persistent_ids=false))]
    fn new(protocol: usize, seed: Option<u64>, allow_persistent_ids: bool) -> PyResult<Self> {
        let version = Version::try_from(protocol).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid protocol: {}", e))
        })?;

        let mut generator = Generator::new(version);
        if let Some(s) = seed {
            generator = generator.with_seed(s);
        }
        generator = generator.with_persistent_id_opcodes(allow_persistent_ids);
        Ok(PyGenerator { inner: generator })
    }

    #[pyo3(signature = (max_size=None))]
    fn generate(&mut self, py: Python, max_size: Option<usize>) -> PyResult<Py<PyBytes>> {
        let previous_bufsize = self.inner.bufsize;
        self.inner.bufsize = self.effective_bufsize(max_size);
        let result = self.inner.generate();
        self.inner.bufsize = previous_bufsize;

        let bytes = result.map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Generation failed: {}", e))
        })?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    #[pyo3(signature = (data, max_size=None))]
    fn generate_from_bytes(
        &mut self,
        py: Python,
        data: &[u8],
        max_size: Option<usize>,
    ) -> PyResult<Py<PyBytes>> {
        let previous_bufsize = self.inner.bufsize;
        self.inner.bufsize = self.effective_bufsize(max_size);
        let result = self.inner.generate_from_arbitrary(data);
        self.inner.bufsize = previous_bufsize;

        let bytes = result.map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Generation failed: {}", e))
        })?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    fn set_opcode_range(&mut self, min: usize, max: usize) {
        self.inner.set_opcode_range(min, max);
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
}

#[pymodule]
fn _native(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    parent_module.add_class::<PyGenerator>()?;
    Ok(())
}
