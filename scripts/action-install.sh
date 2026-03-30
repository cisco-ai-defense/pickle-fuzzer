#!/usr/bin/env bash
set -euo pipefail

repo="cisco-ai-defense/pickle-fuzzer"

binary_path_input="${INPUT_BINARY_PATH:-}"
version_input="${INPUT_VERSION:-}"
action_ref="${GITHUB_ACTION_REF:-}"

is_safe_release_tag() {
  [[ "$1" =~ ^v[0-9A-Za-z][0-9A-Za-z._+-]*$ ]]
}

if [[ -n "$binary_path_input" && -n "$version_input" ]]; then
  echo "version and binary_path are mutually exclusive." >&2
  exit 1
fi

if [[ -n "$binary_path_input" ]]; then
  version="local"
elif [[ -n "$version_input" ]]; then
  version="$version_input"
  version_source="inputs.version"
elif [[ -n "$action_ref" && ( "$action_ref" == v* || "$action_ref" == refs/tags/v* ) ]]; then
  version="${action_ref##refs/tags/}"
  version_source="GITHUB_ACTION_REF"
else
  echo "INPUT_VERSION is required when the action ref is not a release tag." >&2
  if [[ -n "$action_ref" ]]; then
    echo "Received GITHUB_ACTION_REF=${action_ref}." >&2
  else
    echo "Received an empty GITHUB_ACTION_REF (for example from a local checkout)." >&2
  fi
  echo "Set the action version input to an immutable release tag such as v1.2.3, or explicitly set it to latest if you accept a mutable release." >&2
  exit 1
fi

if [[ "$version" != "latest" && "$version" != "local" ]] && ! is_safe_release_tag "$version"; then
  echo "Unsupported release tag from ${version_source}: ${version}" >&2
  echo "Expected a release tag like v1, v1.2.3, or v1.2.3-rc1." >&2
  exit 1
fi

if [[ "$version" == "latest" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    echo "GitHub CLI is required to resolve the latest release tag." >&2
    exit 1
  fi
  version="$(gh release view --repo "${repo}" --json tagName --jq '.tagName')"
  if [[ -z "$version" ]]; then
    echo "Failed to resolve the latest release tag for ${repo}." >&2
    exit 1
  fi
  if ! is_safe_release_tag "$version"; then
    echo "Resolved latest release tag is invalid: ${version}" >&2
    exit 1
  fi
fi

os="${RUNNER_OS:-}"
arch="${RUNNER_ARCH:-}"
bin_name="pickle-fuzzer"

case "$os" in
  Linux)
    case "$arch" in
      X64) asset="pickle-fuzzer-linux-x86_64" ;;
      *) echo "Unsupported Linux arch: $arch" >&2; exit 1 ;;
    esac
    ;;
  macOS)
    case "$arch" in
      X64) asset="pickle-fuzzer-macos-x86_64" ;;
      ARM64) asset="pickle-fuzzer-macos-aarch64" ;;
      *) echo "Unsupported macOS arch: $arch" >&2; exit 1 ;;
    esac
    ;;
  Windows)
    case "$arch" in
      X64) asset="pickle-fuzzer-windows-x86_64.exe" ;;
      *) echo "Unsupported Windows arch: $arch" >&2; exit 1 ;;
    esac
    bin_name="pickle-fuzzer.exe"
    ;;
  *)
    echo "Unsupported runner OS: $os" >&2
    exit 1
    ;;
esac

install_root="${RUNNER_TEMP:-/tmp}/pickle-fuzzer"
if [[ "$os" == "Windows" ]] && command -v cygpath >/dev/null 2>&1; then
  install_root="$(cygpath -u "${RUNNER_TEMP:-/tmp}")/pickle-fuzzer"
  install_root_native="$(cygpath -w "${RUNNER_TEMP:-/tmp}")\\pickle-fuzzer"
else
  install_root_native="$install_root"
fi

install_dir="${install_root}/${version}/${os}/${arch}"
if [[ "$os" == "Windows" ]]; then
  install_dir_native="${install_root_native}\\${version}\\${os}\\${arch}"
else
  install_dir_native="${install_root_native}/${version}/${os}/${arch}"
fi
mkdir -p "$install_dir"

if [[ -n "$binary_path_input" ]]; then
  local_binary_path="$binary_path_input"
  if [[ "$os" == "Windows" ]]; then
    if command -v cygpath >/dev/null 2>&1 && [[ "$binary_path_input" =~ ^[A-Za-z]:[\\/] ]]; then
      local_binary_path="$(cygpath -u "$binary_path_input")"
    else
      local_binary_path="${binary_path_input//\\//}"
    fi
  fi

  if [[ ! -f "$local_binary_path" ]]; then
    echo "Local binary not found: ${binary_path_input}" >&2
    exit 1
  fi

  echo "Installing local binary ${binary_path_input}"
  cp "$local_binary_path" "${install_dir}/${bin_name}"
else
  url="https://github.com/${repo}/releases/download/${version}/${asset}"

  echo "Downloading ${url}"
  curl -fsSL -o "${install_dir}/${bin_name}" "$url"
fi

chmod +x "${install_dir}/${bin_name}"

echo "${install_dir_native}" >> "${GITHUB_PATH}"
if [[ "$os" == "Windows" ]]; then
  echo "binary-path=${install_dir_native}\\${bin_name}" >> "${GITHUB_OUTPUT}"
else
  echo "binary-path=${install_dir_native}/${bin_name}" >> "${GITHUB_OUTPUT}"
fi
echo "resolved-version=${version}" >> "${GITHUB_OUTPUT}"
