#!/usr/bin/env bash
set -euo pipefail

harness_input="${INPUT_HARNESS:-}"
if [[ -z "$harness_input" ]]; then
  echo "Input 'harness' is required for mode=atheris." >&2
  exit 1
fi

workspace="${GITHUB_WORKSPACE:-$PWD}"
if [[ "$harness_input" = /* ]]; then
  harness_path="$harness_input"
else
  harness_path="${workspace}/${harness_input}"
fi

if [[ ! -f "$harness_path" ]]; then
  echo "Harness not found: ${harness_path}" >&2
  exit 1
fi

python -m pip install --upgrade pip
python -m pip install maturin atheris

pushd "${GITHUB_ACTION_PATH}" >/dev/null
maturin develop --release
popd >/dev/null

if [[ -n "${INPUT_HARNESS_ARGS:-}" ]]; then
  echo "Running harness: ${harness_path} ${INPUT_HARNESS_ARGS}"
  # shellcheck disable=SC2086
  python "${harness_path}" ${INPUT_HARNESS_ARGS}
else
  echo "Running harness: ${harness_path}"
  python "${harness_path}"
fi
