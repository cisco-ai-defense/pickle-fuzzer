## Description

Fix the failing fuzz/security CI work on this branch and add a GitHub-hosted
comparison workflow for the Python validator environment hypothesis.

This updates `rand` from `0.9.2` to `0.9.4`, makes the
`validate_with_python` fuzz target explicitly configurable via
`PICKLE_FUZZ_PYTHON_ENV_POLICY`, sets the main GitHub-hosted fuzz workflow to
use the `strip_setup_python_and_ld_library_path` policy, and adds a
PR/workflow-dispatch comparison matrix that runs `inherit`,
`strip_setup_python`, and
`strip_setup_python_and_ld_library_path` side by side on `ubuntu-latest`.
It also adds a workflow-dispatch replay workflow for saved comparison
artifacts, a shared fuzz helper with unit and integration coverage, a
child-env reporting example, and the final Clippy fix for the PR.

## Related Issue

N/A

## Type of Change

- [x] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [x] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring
- [x] Test addition or update

## Changes Made

- Bump `rand` to `0.9.4` in `Cargo.toml`, `Cargo.lock`, and `fuzz/Cargo.lock`.
- Add `PICKLE_FUZZ_PYTHON_ENV_POLICY` handling to
  `fuzz/src/python_env.rs` and reuse it from
  `fuzz/fuzz_targets/validate_with_python.rs` with three supported modes:
  `inherit`, `strip_setup_python`, and
  `strip_setup_python_and_ld_library_path`.
- Add `fuzz/examples/report_python_env.rs` so GitHub CI can capture the
  effective child-process environment after the policy is applied.
- Add fuzz-crate tests that verify both policy-to-child-env behavior and the
  workflow/README policy contract.
- Switch the scheduled and custom GitHub-hosted fuzz workflow to the targeted
  `strip_setup_python_and_ld_library_path` policy instead of broad leak
  suppression.
- Add `.github/workflows/fuzz-python-env-comparison.yml` so the PR can compare
  the three environment policies on GitHub-hosted x86_64 runners, using the
  same helper code that the fuzz target uses.
- Add `.github/workflows/fuzz-python-env-replay.yml` so a saved `inherit`
  leak input and the `strip-setup-python` zero-byte artifact can be replayed
  under all three policies on GitHub-hosted x86_64 runners.
- Pin the fuzz workflows to `nightly-2026-04-16` and `cargo-fuzz 0.13.1`,
  make the comparison workflow cache matrix-specific, and upload the env report
  artifact for each matrix job.
- Update `src/generator/emission.rs` to satisfy newer Clippy stable releases by
  making short-bin emission explicit via `u8::try_from`.

## Testing Performed

- [x] Unit tests added/updated
- [x] Integration tests added/updated
- [x] Manual testing performed
- [x] All existing tests pass (`cargo test`)

**Test commands:**
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

## Checklist

- [x] Code follows project style guidelines (`cargo fmt`)
- [x] No clippy warnings (`cargo clippy -- -D warnings`)
- [x] All tests pass (`cargo test`)
- [x] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (if applicable)
- [x] No breaking changes (or documented if necessary)
- [ ] Commit messages follow [conventional commit format](https://www.conventionalcommits.org/)

## Performance Impact

- [x] No significant performance regression

**Benchmark results:**
```
N/A
```

## Screenshots/Examples

Not applicable.

## Additional Notes

- The scheduled failures on April 13, 2026 and April 16, 2026 both minimized
  to a zero-byte artifact, but the leak sizes differed (`537 bytes in 8
  allocations` vs `464 bytes in 6 allocations`), which points to an
  intermittent shutdown/runtime path rather than a deterministic bad pickle
  input.
- The comparison workflow uploads both the fuzz artifacts and the
  `fuzz-python-env-report-*` artifact for each policy.
- The replay workflow runs on relevant PR updates and on `workflow_dispatch`;
  it either resolves the latest completed comparison run on the branch or uses
  an explicit run id, then downloads artifacts with the repo's `GITHUB_TOKEN`.
- The tracked plan and PR summary for this branch live under `plans/`.

## Breaking Changes

None.
