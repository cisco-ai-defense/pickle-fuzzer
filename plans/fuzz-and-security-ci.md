# Fuzz And Security CI Plan

## Scope

Fix the failing fuzz and security workflows on PR `#24` and leave the branch on
the final hosted-runner fix for `validate_with_python`.

## Acceptance

- `rand` is updated past the advisory version in the root crate and fuzz crate
  lockfiles.
- `validate_with_python` can select a Python subprocess env policy through
  `PICKLE_FUZZ_PYTHON_ENV_POLICY`.
- The main GitHub-hosted fuzz workflow uses
  `strip_setup_python_and_ld_library_path` for `validate_with_python` without
  broad leak suppression.
- The fuzz helper contract tests run in regular PR CI.
- Security CI covers both `Cargo.lock` and `fuzz/Cargo.lock`.
- `cargo clippy --all-targets --all-features -- -D warnings` passes on the
  stable toolchain used by GitHub CI.

## Evidence

Local verification run on April 16, 2026:

```bash
cargo +1.95.0 fmt --all -- --check
cargo +1.95.0 clippy --all-targets --all-features -- -D warnings
cargo test --all-features
uv run pytest
cargo audit
cargo deny check advisories
cd fuzz && cargo audit
cargo +nightly-2026-04-16 test --manifest-path fuzz/Cargo.toml
PICKLE_FUZZ_PYTHON_ENV_POLICY=strip_setup_python_and_ld_library_path cargo +nightly-2026-04-16 run --manifest-path fuzz/Cargo.toml --example report_python_env --quiet
```

GitHub evidence already observed on PR `#24`:

- `Security Audit` green
- `Fuzz Testing` green
- Investigation on April 16, 2026 showed the saved `inherit` leak input
  reproduces under `inherit` and `strip_setup_python`, but goes clean under
  `strip_setup_python_and_ld_library_path`

## Notes

- The main GitHub-hosted `validate_with_python` workflow now uses
  `strip_setup_python_and_ld_library_path`, while the fuzz target's unset local
  default remains `strip_setup_python`.
