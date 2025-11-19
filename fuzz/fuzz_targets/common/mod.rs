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

use std::process::{Command, Stdio};
use std::io::Write;

/// validate pickle bytecode structure with pickletools.dis()
/// 
/// this is safer than pickle.loads() because it only parses the bytecode
/// without executing it, and provides stricter validation of the structure.
pub fn validate_with_pickletools(pickle_bytes: &[u8]) -> Result<(), String> {
    let mut child = Command::new("python3")
        .arg("-c")
        .arg("import sys, pickletools; pickletools.dis(sys.stdin.buffer.read())")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn python: {}", e))?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(pickle_bytes)
            .map_err(|e| format!("failed to write to python: {}", e))?;
    }
    
    let output = child.wait_with_output()
        .map_err(|e| format!("failed to wait for python: {}", e))?;
    
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("pickletools validation failed: {}", stderr))
    }
}

/// validate pickle can be loaded (optional, more permissive test)
/// 
/// warning: this executes the pickle, which could run arbitrary code.
/// only use with trusted/generated pickles in isolated environments.
pub fn validate_with_loads(pickle_bytes: &[u8]) -> Result<(), String> {
    let mut child = Command::new("python3")
        .arg("-c")
        .arg("import sys, pickle; pickle.loads(sys.stdin.buffer.read())")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn python: {}", e))?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(pickle_bytes)
            .map_err(|e| format!("failed to write to python: {}", e))?;
    }
    
    let output = child.wait_with_output()
        .map_err(|e| format!("failed to wait for python: {}", e))?;
    
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("pickle.loads() failed: {}", stderr))
    }
}
