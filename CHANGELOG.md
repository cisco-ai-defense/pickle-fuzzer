# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

### Changed
- Updated CONTRIBUTING.md with detailed guidelines
- Enhanced README.md with comprehensive documentation
- Improved error handling throughout codebase

### Fixed
- Various code quality improvements based on clippy suggestions

## [0.1.0] - 2024-11-19

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

[Unreleased]: https://github.com/cisco-ai-defense/pickle-fuzzer/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/cisco-ai-defense/pickle-fuzzer/releases/tag/v0.1.0
