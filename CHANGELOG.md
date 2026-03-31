# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1] - Unreleased

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
- GitHub Actions CI/CD workflows for automated testing
- Security audit workflow for dependency scanning
- Issue and pull request templates
- SECURITY.md with vulnerability reporting process
- Comprehensive documentation (TESTING.md, BENCHMARKS.md, AGENTS.md)
- Performance benchmarks using Criterion
- Python bindings with PyO3
- Atheris integration for structure-aware fuzzing
- Fuzzing targets with cargo-fuzz
- Support for all pickle protocol versions (0-5)
- CLI flags `--allow-ext` and `--allow-buffer` for opt-in opcode generation
- `--mutators all` option that excludes unsafe mutators by default
- Memo index validation for GET opcodes with unsafe mode bypass

### Changed
- Updated CONTRIBUTING.md with detailed guidelines
- Enhanced README.md with comprehensive documentation
- Improved error handling throughout codebase
- EXT and buffer opcodes now disabled by default, require explicit flags
- MemoIndexMutator excluded from fuzzer validation target
- Performance improvements: 10-20% faster across all benchmark categories

### Fixed
- Stack not empty after STOP errors in pickle validation
- DUP opcode incorrectly duplicating MARK objects
- Buffer opcodes (NEXT_BUFFER, READONLY_BUFFER) causing validation failures
- EXT opcodes generating without configured extension registry
- STRING opcode missing escape sequences for special characters
- UNICODE opcode generating invalid escape sequences with backslashes
- GET/BinGet/LongBinGet using mutated memo indices that don't exist
- Memo mutator generating invalid references even in safe mode

[Unreleased]: https://github.com/cisco-ai-defense/pickle-fuzzer/compare/v1.0.0...HEAD
[1.0.1]: https://github.com/cisco-ai-defense/pickle-fuzzer/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/cisco-ai-defense/pickle-fuzzer/releases/tag/v1.0.0
