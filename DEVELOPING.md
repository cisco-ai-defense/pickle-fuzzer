# Development Guide

This guide covers the technical details for developing `cisco-ai-defense-pickle-fuzzer`.

## Prerequisites

- Rust 1.70 or later
- Python 3.11.14 (for Python bindings)
- Git

## Initial Setup

### Clone and Build

```bash
git clone https://github.com/cisco-ai-defense/pickle-fuzzer
cd pickle-fuzzer
cargo build
```

### Development Build

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized runtime)
cargo build --release

# With Python bindings
cargo build --features python-bindings
```

### Running Locally

```bash
# Run from source
cargo run -- output.pkl

# Run release binary
cargo run --release -- output.pkl

# Run with arguments
cargo run -- --dir samples --samples 100 --protocol 3
```

## Code Organization

```
cisco-ai-defense-pickle-fuzzer/
├── src/
│   ├── lib.rs              # Library root and public API
│   ├── main.rs             # CLI entry point
│   ├── cli.rs              # Command-line argument parsing
│   ├── generator/          # Core generation logic
│   │   ├── mod.rs          # Generator struct and public API
│   │   ├── core.rs         # Main generation algorithm
│   │   ├── emission.rs     # Opcode emission and encoding
│   │   ├── stack_ops.rs    # Stack simulation
│   │   ├── validation.rs   # Opcode validation
│   │   └── mutation.rs     # Mutation orchestration
│   ├── mutators/           # Mutation strategies
│   │   ├── mod.rs          # Mutator trait and registry
│   │   ├── bitflip.rs      # Bit flip mutations
│   │   ├── boundary.rs     # Boundary value mutations
│   │   └── ...
│   ├── opcodes.rs          # Opcode definitions and protocol mappings
│   ├── protocol.rs         # Protocol version enum
│   ├── stack.rs            # Stack object types
│   ├── state.rs            # Generator state management
│   └── python.rs           # Python bindings (PyO3)
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
├── fuzz/                   # Fuzzing targets (cargo-fuzz)
└── python/                 # Python package
    ├── cisco_ai_defense_pickle_fuzzer/
    │   ├── __init__.py     # Python module entry point
    │   └── fuzzer.py       # Atheris integration
    ├── examples/           # Python usage examples
    └── tests/              # Python tests
```

## Writing Tests

### Unit Tests

Add unit tests at the bottom of source files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        let result = my_function();
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_error_case() {
        let result = my_function_with_error();
        assert!(result.is_err());
    }
}
```

### Integration Tests

Add integration tests in `tests/` directory:

```rust
use cisco_ai_defense_pickle_fuzzer::{Generator, Version};

#[test]
fn test_integration_scenario() {
    let mut gen = Generator::new(Version::V3);
    let pickle = gen.generate();
    
    assert!(!pickle.is_empty());
    assert_eq!(pickle[pickle.len() - 1], b'.');  // Ends with STOP
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test generator::

# Run doc tests only
cargo test --doc
```

See [TESTING.md](TESTING.md) for comprehensive testing guidelines including coverage, validation, and performance testing.

## Code Style

### Formatting

We use `rustfmt` with default settings:

```bash
# Format all code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Linting

We use `clippy` with strict warnings:

```bash
# Run clippy
cargo clippy

# Treat warnings as errors (CI requirement)
cargo clippy -- -D warnings

# Fix auto-fixable issues
cargo clippy --fix
```

### Pre-commit Checklist

Before committing, run:

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Code Coverage

We maintain >70% code coverage:

```bash
# Install tarpaulin (first time only)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/tarpaulin-report.html  # macOS
xdg-open coverage/tarpaulin-report.html  # Linux
```

**Coverage Requirements:**
- Overall coverage: >70%
- New code: >80%
- All public APIs must be tested
- Include both unit and integration tests

## Python Development

### Setup Python Environment

```bash
cd python
python3.11 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install maturin
```

### Build Python Package

```bash
# Development build (editable install)
maturin develop

# Release build
maturin develop --release

# Build wheel
maturin build --release -o dist/
```

### Python Tests

```bash
cd python
pytest tests/
```

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench single_generation

# Generate detailed reports
cargo bench -- --save-baseline my-baseline
```

### Viewing Results

```bash
# Open HTML report
open target/criterion/report/index.html
```

See [BENCHMARKS.md](BENCHMARKS.md) for detailed performance analysis.

## Debugging

### Debug Logging

Add debug output:

```rust
dbg!(variable);
eprintln!("Debug: {:?}", value);
```

### Using a Debugger

```bash
# Build with debug symbols
cargo build

# Run with lldb (macOS) or gdb (Linux)
rust-lldb target/debug/cisco-ai-defense-pickle-fuzzer
```

### Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph -- --dir samples --samples 1000
```

## Common Development Tasks

### Adding a New Opcode

1. Add opcode to `src/opcodes.rs`
2. Add to protocol version map
3. Implement stack effects in `src/generator/stack_ops.rs`
4. Add emission logic in `src/generator/emission.rs`
5. Add validation rules in `src/generator/validation.rs`
6. Add tests

### Adding a New Mutator

1. Create file in `src/mutators/`
2. Implement `Mutator` trait
3. Add to `MutatorKind` enum in `src/mutators/mod.rs`
4. Add tests
5. Update documentation

### Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Update to latest compatible versions
cargo upgrade  # Requires cargo-edit
```

## Continuous Integration

Our CI pipeline runs:

1. **Formatting check**: `cargo fmt -- --check`
2. **Linting**: `cargo clippy -- -D warnings`
3. **Tests**: `cargo test --all`
4. **Coverage**: `cargo tarpaulin`
5. **Benchmarks**: `cargo bench` (on main branch)
6. **Fuzzing**: `cargo fuzz` (nightly)

Ensure all checks pass locally before pushing.

## Release Process

1. Update version in `Cargo.toml` and `pyproject.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create git tag: `git tag -a v0.x.0 -m "Release v0.x.0"`
5. Push tag: `git push origin v0.x.0`
6. CI will automatically build and publish

## Getting Help

- **Documentation**: Check inline docs with `cargo doc --open`
- **Examples**: See `examples/` directory
- **Discussions**: [GitHub Discussions](https://github.com/cisco-ai-defense/pickle-fuzzer/discussions)
- **Issues**: [GitHub Issues](https://github.com/cisco-ai-defense/pickle-fuzzer/issues)
