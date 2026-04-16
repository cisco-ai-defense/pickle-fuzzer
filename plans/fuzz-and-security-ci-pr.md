## Description

Fix the failing fuzz/security CI work on this branch and leave only the final
hosted-runner fix in the shipped workflows.

This updates `rand` from `0.9.2` to `0.9.4`, keeps
`validate_with_python` explicitly configurable via
`PICKLE_FUZZ_PYTHON_ENV_POLICY`, and sets the main GitHub-hosted fuzz workflow
to use `strip_setup_python_and_ld_library_path` for
`validate_with_python`. The branch also keeps the shared fuzz helper, its
unit/integration coverage, and the final Clippy fix for the PR.

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
- Pin the fuzz workflows to `nightly-2026-04-16` and `cargo-fuzz 0.13.1`,
  and keep the real hosted workflow scoped to the final verdict fix.
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

- Hosted-runner investigation on April 16, 2026 showed the saved `inherit`
  leak input still leaked under `inherit` and `strip_setup_python`, but went
  clean under `strip_setup_python_and_ld_library_path`.
- The zero-byte `crash-da39a3ee...` artifact from the earlier failing run did
  not replay as a deterministic crash input.
- The tracked plan and PR summary for this branch live under `plans/`.

## Breaking Changes

None.
