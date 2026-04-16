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

use pickle_fuzzer_fuzz::python_env::{
    spawn_python_command, PythonEnvPolicy, OBSERVED_PYTHON_ENV_KEYS,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    let policy = PythonEnvPolicy::from_env_var();
    let keys = OBSERVED_PYTHON_ENV_KEYS
        .iter()
        .map(|key| format!("{key:?}"))
        .collect::<Vec<_>>()
        .join(", ");

    let script = format!(
        "import os\nfor key in [{keys}]:\n    value = os.environ.get(key)\n    print(f\"{{key}}={{value if value is not None else '<unset>'}}\")\n"
    );

    let output = match spawn_python_command(policy).arg("-c").arg(script).output() {
        Ok(output) => output,
        Err(err) => {
            eprintln!("failed to run python3 with policy {}: {err}", policy.as_str());
            return ExitCode::FAILURE;
        }
    };

    eprintln!("policy={}", policy.as_str());

    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        return ExitCode::FAILURE;
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));
    ExitCode::SUCCESS
}
