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

use pickle_fuzzer_fuzz::python_env::PythonEnvPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

const POLICY_KEYS: &[&str] = &[
    "pythonLocation",
    "Python_ROOT_DIR",
    "Python2_ROOT_DIR",
    "Python3_ROOT_DIR",
    "PKG_CONFIG_PATH",
    "LD_LIBRARY_PATH",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn child_env_for(policy: PythonEnvPolicy) -> BTreeMap<String, String> {
    let mut command = Command::new("python3");
    for key in POLICY_KEYS {
        command.env(key, format!("sentinel-{key}"));
    }
    command.env("PICKLE_FUZZ_TEST_PRESERVED", "keep-me");
    policy.apply(&mut command);

    let output = command
        .arg("-c")
        .arg(
            "import os\nfor key, value in sorted(os.environ.items()):\n    print(f\"{key}={value}\")\n",
        )
        .output()
        .expect("python3 must be available for fuzz env tests");

    assert!(
        output.status.success(),
        "python env report failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("python env report should be UTF-8")
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

#[test]
fn spawned_python_process_applies_each_env_policy() {
    let inherit = child_env_for(PythonEnvPolicy::Inherit);
    for key in POLICY_KEYS {
        assert_eq!(inherit.get(*key), Some(&format!("sentinel-{key}")));
    }
    assert_eq!(
        inherit.get("PICKLE_FUZZ_TEST_PRESERVED"),
        Some(&"keep-me".to_string())
    );

    let stripped = child_env_for(PythonEnvPolicy::StripSetupPython);
    for key in &POLICY_KEYS[..POLICY_KEYS.len() - 1] {
        assert!(!stripped.contains_key(*key), "{key} should be removed");
    }
    assert_eq!(
        stripped.get("LD_LIBRARY_PATH"),
        Some(&"sentinel-LD_LIBRARY_PATH".to_string())
    );
    assert_eq!(
        stripped.get("PICKLE_FUZZ_TEST_PRESERVED"),
        Some(&"keep-me".to_string())
    );

    let stripped_all = child_env_for(PythonEnvPolicy::StripSetupPythonAndLdLibraryPath);
    for key in POLICY_KEYS {
        assert!(!stripped_all.contains_key(*key), "{key} should be removed");
    }
    assert_eq!(
        stripped_all.get("PICKLE_FUZZ_TEST_PRESERVED"),
        Some(&"keep-me".to_string())
    );
}

#[test]
fn workflow_and_docs_stay_in_sync_with_supported_policies() {
    let workflow = std::fs::read_to_string(repo_root().join(".github/workflows/fuzz.yml"))
        .expect("workflow should exist");
    let readme = std::fs::read_to_string(repo_root().join("fuzz/README.md"))
        .expect("fuzz README should exist");

    for policy in [
        PythonEnvPolicy::Inherit,
        PythonEnvPolicy::StripSetupPython,
        PythonEnvPolicy::StripSetupPythonAndLdLibraryPath,
    ] {
        let name = policy.as_str();
        assert!(readme.contains(name), "README must mention policy {name}");
    }

    assert!(
        !readme.contains("fuzz-python-env-comparison.yml"),
        "README should not mention removed comparison workflow"
    );
    assert!(
        !readme.contains("fuzz-python-env-replay.yml"),
        "README should not mention removed replay workflow"
    );

    let policy_count = workflow
        .matches("PICKLE_FUZZ_PYTHON_ENV_POLICY: strip_setup_python_and_ld_library_path")
        .count();
    assert_eq!(
        policy_count, 2,
        "main fuzz workflow should set strip_setup_python_and_ld_library_path for both validate_with_python entry points"
    );
    assert!(
        !workflow.contains("fuzz-python-env-comparison"),
        "main workflow should not reference removed comparison jobs"
    );
    assert!(
        !workflow.contains("fuzz-python-env-replay"),
        "main workflow should not reference removed replay jobs"
    );
}
