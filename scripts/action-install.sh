#!/usr/bin/env bash
set -euo pipefail

repo="cisco-ai-defense/pickle-fuzzer"

version_input="${INPUT_VERSION:-}"
action_ref="${GITHUB_ACTION_REF:-}"

if [[ -n "$version_input" ]]; then
  version="$version_input"
elif [[ -n "$action_ref" && "$action_ref" == v* ]]; then
  version="$action_ref"
else
  version="latest"
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

if [[ "$version" == "latest" ]]; then
  url="https://github.com/${repo}/releases/latest/download/${asset}"
  checksum_url="https://github.com/${repo}/releases/latest/download/${asset}.sha256"
else
  url="https://github.com/${repo}/releases/download/${version}/${asset}"
  checksum_url="https://github.com/${repo}/releases/download/${version}/${asset}.sha256"
fi

echo "Downloading ${url}"
curl -fsSL -o "${install_dir}/${bin_name}" "$url"

checksum_path="${install_dir}/${asset}.sha256"
echo "Downloading ${checksum_url}"
curl -fsSL -o "${checksum_path}" "$checksum_url"

expected_checksum="$(awk '{print $1}' "$checksum_path" | tr '[:upper:]' '[:lower:]')"
if [[ -z "$expected_checksum" ]]; then
  echo "Checksum file is empty or invalid: ${checksum_path}" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  actual_checksum="$(sha256sum "${install_dir}/${bin_name}" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  actual_checksum="$(shasum -a 256 "${install_dir}/${bin_name}" | awk '{print $1}')"
elif command -v certutil >/dev/null 2>&1; then
  file_native="${install_dir}/${bin_name}"
  if command -v cygpath >/dev/null 2>&1; then
    file_native="$(cygpath -w "${install_dir}/${bin_name}")"
  fi
  actual_checksum="$(certutil -hashfile "$file_native" SHA256 | awk 'NR==2 {print tolower($1)}')"
else
  echo "No SHA-256 tool available to verify checksum." >&2
  exit 1
fi

actual_checksum="$(echo "$actual_checksum" | tr '[:upper:]' '[:lower:]')"
if [[ -z "$actual_checksum" ]]; then
  echo "Failed to compute SHA-256 checksum." >&2
  exit 1
fi

if [[ "$expected_checksum" != "$actual_checksum" ]]; then
  echo "Checksum mismatch for ${bin_name}." >&2
  echo "Expected: ${expected_checksum}" >&2
  echo "Actual:   ${actual_checksum}" >&2
  exit 1
fi

chmod +x "${install_dir}/${bin_name}"

echo "${install_dir_native}" >> "${GITHUB_PATH}"
if [[ "$os" == "Windows" ]]; then
  echo "binary-path=${install_dir_native}\\${bin_name}" >> "${GITHUB_OUTPUT}"
else
  echo "binary-path=${install_dir_native}/${bin_name}" >> "${GITHUB_OUTPUT}"
fi
echo "resolved-version=${version}" >> "${GITHUB_OUTPUT}"
