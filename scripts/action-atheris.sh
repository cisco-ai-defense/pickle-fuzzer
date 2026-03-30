#!/usr/bin/env bash
set -euo pipefail

harness_input="${INPUT_HARNESS:-}"
if [[ -z "$harness_input" ]]; then
  echo "Input 'harness' is required for mode=atheris." >&2
  exit 1
fi

workspace_input="${GITHUB_WORKSPACE:-$PWD}"
if ! workspace="$(realpath "$workspace_input")"; then
  echo "Workspace not found: ${workspace_input}" >&2
  exit 1
fi

if [[ "$harness_input" = /* ]]; then
  harness_candidate="$harness_input"
else
  harness_candidate="${workspace}/${harness_input}"
fi

if ! harness_path="$(realpath "$harness_candidate")"; then
  echo "Harness not found: ${harness_candidate}" >&2
  exit 1
fi

if [[ "$workspace" == "/" ]]; then
  workspace_prefix="/"
else
  workspace_prefix="${workspace}/"
fi

if [[ "${harness_path:0:${#workspace_prefix}}" != "$workspace_prefix" ]]; then
  echo "Harness must be within GITHUB_WORKSPACE: ${harness_path}" >&2
  exit 1
fi

if [[ ! -f "$harness_path" ]]; then
  echo "Harness not found: ${harness_path}" >&2
  exit 1
fi

python -m pip install --upgrade pip
python -m pip install maturin atheris

wheel_dir="${RUNNER_TEMP:-/tmp}/pickle-fuzzer-wheels"
rm -rf "$wheel_dir"
mkdir -p "$wheel_dir"

pushd "${GITHUB_ACTION_PATH}" >/dev/null
maturin build --release -o "$wheel_dir"
popd >/dev/null

shopt -s nullglob
wheels=("${wheel_dir}"/*.whl)
shopt -u nullglob

if [[ "${#wheels[@]}" -ne 1 ]]; then
  echo "Expected exactly one wheel in ${wheel_dir}, found ${#wheels[@]}" >&2
  exit 1
fi

python -m pip install "${wheels[0]}"

if [[ -n "${INPUT_HARNESS_ARGS:-}" ]]; then
  IFS=$' \t\n' read -r -a harness_args <<< "${INPUT_HARNESS_ARGS}"
  echo "Running harness: ${harness_path} ${harness_args[*]}"
  python "${harness_path}" "${harness_args[@]}"
else
  echo "Running harness: ${harness_path}"
  python "${harness_path}"
fi
