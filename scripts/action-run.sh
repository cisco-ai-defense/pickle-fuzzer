#!/usr/bin/env bash
set -euo pipefail

is_true() {
  case "${1:-}" in
    true|TRUE|True|1|yes|YES|Yes) return 0 ;;
    *) return 1 ;;
  esac
}

to_posix_path() {
  local path="${1:-}"
  if command -v cygpath >/dev/null 2>&1; then
    case "$path" in
      [A-Za-z]:[\\/]*|\\\\*)
        cygpath -u "$path"
        return
        ;;
      *)
        printf '%s\n' "${path//\\//}"
        return
        ;;
    esac
  fi

  printf '%s\n' "$path"
}

workspace_input="$(to_posix_path "${GITHUB_WORKSPACE:-$PWD}")"
workspace="$(cd "$workspace_input" && pwd -P)"

resolve_workspace_path() {
  local input="$1"
  local label="$2"
  local normalized candidate parent leaf resolved_parent resolved

  normalized="$(to_posix_path "$input")"
  if [[ "$normalized" = /* ]]; then
    candidate="$normalized"
  else
    candidate="${workspace}/${normalized}"
  fi

  if [[ -e "$candidate" || -L "$candidate" ]]; then
    resolved="$(realpath "$candidate" 2>/dev/null)" || {
      echo "Failed to resolve ${label}: ${input}" >&2
      return 1
    }
  else
    parent="$(dirname "$candidate")"
    leaf="$(basename "$candidate")"
    resolved_parent="$(cd "$parent" && pwd -P)" || {
      echo "${label} parent directory does not exist: ${input}" >&2
      return 1
    }

    if [[ "$leaf" == "." ]]; then
      resolved="$resolved_parent"
    else
      resolved="${resolved_parent}/${leaf}"
    fi
  fi

  case "$resolved" in
    "$workspace"|"$workspace"/*)
      printf '%s\n' "$resolved"
      ;;
    *)
      echo "${label} must stay within GITHUB_WORKSPACE: ${input}" >&2
      return 1
      ;;
  esac
}

if [[ -n "${INPUT_ARGS:-}" ]]; then
  echo "Running: pickle-fuzzer ${INPUT_ARGS}"
  # Preserve the existing argv splitting for args without allowing glob expansion.
  set -f
  # shellcheck disable=SC2086
  pickle-fuzzer ${INPUT_ARGS}
  exit 0
fi

args=()

if [[ -n "${INPUT_OUTPUT_DIR:-}" ]]; then
  output_dir="$(resolve_workspace_path "${INPUT_OUTPUT_DIR}" "output_dir")"
  args+=(--dir "${output_dir}")
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

resolved_min_opcodes="${INPUT_MIN_OPCODES:-60}"
resolved_max_opcodes="${INPUT_MAX_OPCODES:-300}"

if [[ "${resolved_min_opcodes}" =~ ^[0-9]+$ && "${resolved_max_opcodes}" =~ ^[0-9]+$ ]]; then
  if (( resolved_min_opcodes > resolved_max_opcodes )); then
    echo "min_opcodes (${resolved_min_opcodes}) cannot exceed max_opcodes (${resolved_max_opcodes})." >&2
    exit 1
  fi
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
  output_file="$(resolve_workspace_path "${INPUT_OUTPUT_FILE}" "output_file")"
  # Keep the positional FILE separate from greedy variadic options.
  args+=(-- "${output_file}")
fi

if [[ ${#args[@]} -eq 0 ]]; then
  echo "No arguments provided; skipping execution."
  exit 0
fi

echo "Running: pickle-fuzzer ${args[*]}"
pickle-fuzzer "${args[@]}"
