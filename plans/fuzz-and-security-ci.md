# Fuzz And Security CI Plan

## Scope

Fix the failing fuzz and security workflows on PR `#24`, keep the fuzz leak
investigation reproducible in GitHub CI, and clear the remaining Clippy failure
on the branch.

## Acceptance

- `rand` is updated past the advisory version in the root crate and fuzz crate
  lockfiles.
- `validate_with_python` can select a Python subprocess env policy through
  `PICKLE_FUZZ_PYTHON_ENV_POLICY`.
- GitHub CI can compare the effective child-process environment for the three
  policy variants on `ubuntu-latest`.
- The main fuzz workflow uses the targeted policy without broad leak
  suppression.
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
PICKLE_FUZZ_PYTHON_ENV_POLICY=strip_setup_python cargo +nightly-2026-04-16 run --manifest-path fuzz/Cargo.toml --example report_python_env --quiet
```

GitHub evidence already observed on PR `#24`:

- `Security Audit` green
- `Fuzz Testing` green
- `Fuzz Python Env Comparison` green for `inherit`,
  `strip-setup-python`, and `strip-setup-python-and-ld-library-path`

## Notes

- The comparison workflow is diagnostic. It verifies the effective Python child
  environment used by the fuzz target and uploads both the env report and fuzz
  artifacts for each policy.
- The current branch default remains `strip_setup_python` for scheduled and
  custom `validate_with_python` runs because it is the least invasive CI-only
  change under active observation.
