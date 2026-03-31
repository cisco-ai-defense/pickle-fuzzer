# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Release workflow provenance generation for future attested releases
- Local-binary installation support in the GitHub Action smoke path
- `--allow-persistent-ids` and corresponding generator support for opt-in `PERSID`/`BINPERSID` coverage
- Rust and Python size-budget support that keeps generated pickles within a requested byte limit

### Changed
- GitHub Action entrypoints now require explicit version selection outside immutable release tags, support safer argument handling, and validate installation inputs more strictly
- Release, CI, smoke, and fuzz workflows now pin critical actions, validate their inputs, and test the candidate build instead of implicitly trusting the latest published release
- Generator contracts now normalize opcode ranges, derive deterministic per-sample batch seeds, auto-reset between calls, and apply opcode budgets to the full emitted pickle instead of only the random body
- Python bindings and examples now follow the same generation and budget semantics as the Rust CLI
- Standard library target discovery now stores unambiguous `module<TAB>member` entries and avoids importing package trees while building the shipped catalog
- Packaging docs and metadata now reference the published crate name and Python 3.11 minor-version support correctly

### Fixed
- Python validation once again enforces the whole-file `STOP` boundary and matches the standalone validator, fuzz target, and CI harness behavior
- Stack simulation and validation now handle protocol 0/1 cleanup, MARK-aware collection operations, memo aliasing, `BUILD` identity preservation, `READONLY_BUFFER`, long sign extension, and `INT` bool decoding correctly
- Unsafe mutator handling now re-simulates post-emission rewrites, rejects unsupported rewrites, gates unsafe mutators behind `--unsafe-mutations`, and makes type-confusion rewrites protocol-aware
- Python examples no longer ship broken harness syntax, noisy parser-error logging, or sample flows that encourage unsandboxed `pickle.loads()` on generated data
- Cargo packaging metadata no longer blocks publish due to invalid keyword counts or ambiguous install instructions

## [1.0.0] - 2025-12-23

### Added
- Initial release of pickle-fuzzer
- Structure-aware pickle generation
- Stack and memo simulation
- Multi-protocol support (protocols 0-5)
- CLI interface with batch generation
- Configurable opcode ranges
- Deterministic generation with seed support
- Parallel generation with rayon
- Mutation system with multiple strategies
- Comprehensive test suite
- Apache 2.0 license
- CODE_OF_CONDUCT.md
- CONTRIBUTING.md
- Basic documentation

[Unreleased]: https://github.com/cisco-ai-defense/pickle-fuzzer/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/cisco-ai-defense/pickle-fuzzer/releases/tag/v1.0.0
