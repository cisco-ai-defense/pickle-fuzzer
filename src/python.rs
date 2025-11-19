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

#[pymethods]
impl PyGenerator {
    #[new]
    #[pyo3(signature = (protocol=3, seed=None))]
    fn new(protocol: usize, seed: Option<u64>) -> PyResult<Self> {
        let version = Version::try_from(protocol).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid protocol: {}", e))
        })?;

        let mut generator = Generator::new(version);
        if let Some(s) = seed {
            generator = generator.with_seed(s);
        }
        Ok(PyGenerator { inner: generator })
    }

    fn generate(&mut self, py: Python) -> PyResult<Py<PyBytes>> {
        let bytes = self.inner.generate().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Generation failed: {}", e))
        })?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    fn generate_from_bytes(&mut self, py: Python, data: &[u8]) -> PyResult<Py<PyBytes>> {
        let bytes = self.inner.generate_from_arbitrary(data).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Generation failed: {}", e))
        })?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    fn set_opcode_range(&mut self, min: usize, max: usize) {
        let version = self.inner.state.version;
        let new_gen = Generator::new(version).with_opcode_range(min, max);
        self.inner = new_gen;
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
