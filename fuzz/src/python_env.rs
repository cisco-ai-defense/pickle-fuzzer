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

use std::env;
use std::ffi::OsString;
use std::process::Command;

pub const PYTHON_ENV_POLICY_VAR: &str = "PICKLE_FUZZ_PYTHON_ENV_POLICY";
pub const REPORTED_PYTHON_ENV_KEYS: &[&str] = &[
    "pythonLocation",
    "Python_ROOT_DIR",
    "Python2_ROOT_DIR",
    "Python3_ROOT_DIR",
    "PKG_CONFIG_PATH",
    "LD_LIBRARY_PATH",
    "PATH",
    "PYTHONHOME",
    "PYTHONPATH",
    "VIRTUAL_ENV",
    "CONDA_PREFIX",
];

const STRIP_SETUP_PYTHON_REMOVALS: &[&str] = &[
    "pythonLocation",
    "Python_ROOT_DIR",
    "Python2_ROOT_DIR",
    "Python3_ROOT_DIR",
    "PKG_CONFIG_PATH",
];

const STRIP_SETUP_PYTHON_AND_LD_LIBRARY_PATH_REMOVALS: &[&str] = &[
    "pythonLocation",
    "Python_ROOT_DIR",
    "Python2_ROOT_DIR",
    "Python3_ROOT_DIR",
    "PKG_CONFIG_PATH",
    "LD_LIBRARY_PATH",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PythonEnvPolicy {
    Inherit,
    StripSetupPython,
    StripSetupPythonAndLdLibraryPath,
}

impl PythonEnvPolicy {
    pub fn from_env_var() -> Self {
        match env::var_os(PYTHON_ENV_POLICY_VAR) {
            Some(value) => Self::resolve_os(Some(&value)),
            None => Self::resolve(None),
        }
    }

    pub fn resolve(raw: Option<&str>) -> Self {
        match raw {
            Some("inherit") => Self::Inherit,
            Some("strip_setup_python") => Self::StripSetupPython,
            Some("strip_setup_python_and_ld_library_path") => {
                Self::StripSetupPythonAndLdLibraryPath
            }
            Some(value) => {
                eprintln!(
                    "warning: invalid {PYTHON_ENV_POLICY_VAR}={value}; defaulting to strip_setup_python"
                );
                Self::StripSetupPython
            }
            None => Self::StripSetupPython,
        }
    }

    pub fn resolve_os(raw: Option<&OsString>) -> Self {
        match raw.and_then(|value| value.to_str()) {
            Some(value) => Self::resolve(Some(value)),
            None if raw.is_some() => {
                eprintln!(
                    "warning: invalid non-UTF-8 {PYTHON_ENV_POLICY_VAR}; defaulting to strip_setup_python"
                );
                Self::StripSetupPython
            }
            None => Self::resolve(None),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::StripSetupPython => "strip_setup_python",
            Self::StripSetupPythonAndLdLibraryPath => {
                "strip_setup_python_and_ld_library_path"
            }
        }
    }

    pub const fn removed_keys(self) -> &'static [&'static str] {
        match self {
            Self::Inherit => &[],
            Self::StripSetupPython => STRIP_SETUP_PYTHON_REMOVALS,
            Self::StripSetupPythonAndLdLibraryPath => {
                STRIP_SETUP_PYTHON_AND_LD_LIBRARY_PATH_REMOVALS
            }
        }
    }

    pub fn apply(self, command: &mut Command) {
        for key in self.removed_keys() {
            command.env_remove(key);
        }
    }
}

pub fn spawn_python_command(policy: PythonEnvPolicy) -> Command {
    let mut command = Command::new("python3");
    policy.apply(&mut command);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_defaults_to_strip_setup_python() {
        assert_eq!(PythonEnvPolicy::resolve(None), PythonEnvPolicy::StripSetupPython);
    }

    #[test]
    fn resolve_accepts_supported_values() {
        assert_eq!(
            PythonEnvPolicy::resolve(Some("inherit")),
            PythonEnvPolicy::Inherit
        );
        assert_eq!(
            PythonEnvPolicy::resolve(Some("strip_setup_python")),
            PythonEnvPolicy::StripSetupPython
        );
        assert_eq!(
            PythonEnvPolicy::resolve(Some("strip_setup_python_and_ld_library_path")),
            PythonEnvPolicy::StripSetupPythonAndLdLibraryPath
        );
    }

    #[test]
    fn resolve_invalid_values_fall_back_to_safe_default() {
        assert_eq!(
            PythonEnvPolicy::resolve(Some("definitely_not_valid")),
            PythonEnvPolicy::StripSetupPython
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolve_non_utf8_values_fall_back_to_safe_default() {
        use std::os::unix::ffi::OsStringExt;

        let value = OsString::from_vec(b"\xffinvalid".to_vec());

        assert_eq!(
            PythonEnvPolicy::resolve_os(Some(&value)),
            PythonEnvPolicy::StripSetupPython
        );
    }

    #[test]
    fn removed_keys_match_expected_policy_scope() {
        assert_eq!(PythonEnvPolicy::Inherit.removed_keys(), &[] as &[&str]);
        assert_eq!(
            PythonEnvPolicy::StripSetupPython.removed_keys(),
            STRIP_SETUP_PYTHON_REMOVALS
        );
        assert_eq!(
            PythonEnvPolicy::StripSetupPythonAndLdLibraryPath.removed_keys(),
            STRIP_SETUP_PYTHON_AND_LD_LIBRARY_PATH_REMOVALS
        );
    }
}
