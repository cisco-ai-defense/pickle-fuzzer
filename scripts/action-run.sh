#!/usr/bin/env bash
set -euo pipefail

is_true() {
  case "${1:-}" in
    true|TRUE|True|1|yes|YES|Yes) return 0 ;;
    *) return 1 ;;
  esac
}

if [[ -n "${INPUT_ARGS:-}" ]]; then
  echo "Running: pickle-fuzzer ${INPUT_ARGS}"
  # shellcheck disable=SC2086
  pickle-fuzzer ${INPUT_ARGS}
  exit 0
fi

args=()

if [[ -n "${INPUT_OUTPUT_DIR:-}" ]]; then
  args+=(--dir "${INPUT_OUTPUT_DIR}")
fi

if [[ -n "${INPUT_SAMPLES:-}" ]]; then
  args+=(--samples "${INPUT_SAMPLES}")
fi

if [[ -n "${INPUT_PROTOCOL:-}" ]]; then
  args+=(--protocol "${INPUT_PROTOCOL}")
fi

if [[ -n "${INPUT_SEED:-}" ]]; then
  args+=(--seed "${INPUT_SEED}")
fi

if [[ -n "${INPUT_MIN_OPCODES:-}" ]]; then
  args+=(--min-opcodes "${INPUT_MIN_OPCODES}")
fi

if [[ -n "${INPUT_MAX_OPCODES:-}" ]]; then
  args+=(--max-opcodes "${INPUT_MAX_OPCODES}")
fi

if [[ -n "${INPUT_MUTATORS:-}" ]]; then
  IFS=', ' read -r -a mutators <<< "${INPUT_MUTATORS}"
  for mutator in "${mutators[@]}"; do
    [[ -z "$mutator" ]] && continue
    args+=(--mutators "$mutator")
  done
fi

if [[ -n "${INPUT_MUTATION_RATE:-}" ]]; then
  args+=(--mutation-rate "${INPUT_MUTATION_RATE}")
fi

if is_true "${INPUT_UNSAFE_MUTATIONS:-}"; then
  args+=(--unsafe-mutations)
fi

if is_true "${INPUT_ALLOW_EXT:-}"; then
  args+=(--allow-ext)
fi

if is_true "${INPUT_ALLOW_BUFFER:-}"; then
  args+=(--allow-buffer)
fi

if [[ -n "${INPUT_OUTPUT_FILE:-}" ]]; then
  if [[ -n "${INPUT_OUTPUT_DIR:-}" ]]; then
    echo "Both output_dir and output_file were set; choose one." >&2
    exit 1
  fi
  args+=("${INPUT_OUTPUT_FILE}")
fi

if [[ ${#args[@]} -eq 0 ]]; then
  echo "No arguments provided; skipping execution."
  exit 0
fi

echo "Running: pickle-fuzzer ${args[*]}"
pickle-fuzzer "${args[@]}"
