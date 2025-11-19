# Testing Guide

## Running Tests

### Unit and Integration Tests

```bash
cargo test
```

### Code Coverage

We use `cargo-tarpaulin` for code coverage analysis.

#### Install Tarpaulin

```bash
cargo install cargo-tarpaulin
```

#### Generate Coverage Report

```bash
# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage

# Generate multiple formats
cargo tarpaulin --out Html --out Lcov --output-dir coverage

# View HTML report
open coverage/tarpaulin-report.html  # macOS
xdg-open coverage/tarpaulin-report.html  # Linux
```

#### Coverage Configuration

Coverage settings are configured in `tarpaulin.toml`. Current configuration:
- Output formats: HTML and Lcov
- Includes doc tests
- Timeout: 120 seconds
- Excludes: test files and benchmarks

### Test Organization

```
pickle-fuzzer/
├── src/
│   ├── generator.rs    # Unit tests at bottom of file
│   └── ...
└── tests/
    └── integration_test.rs  # Integration tests
```

## Validation Testing

### Pickle Validation

Use the included Python script to validate generated pickles:

```bash
# Validate a single pickle
python3 scripts/validate-pickles.py output.pkl

# Validate all pickles in a directory
python3 scripts/validate-pickles.py samples/

# Get detailed disassembly
python3 scripts/validate-pickles.py --verbose output.pkl
```

### Manual Testing

```bash
# Generate a single pickle
cargo run -- test.pkl --seed 42

# Generate batch
cargo run -- --dir samples --samples 100 --seed 123

# Test determinism
cargo run -- test1.pkl --seed 42
cargo run -- test2.pkl --seed 42
diff test1.pkl test2.pkl  # Should be identical
```

## Performance Testing

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench single_generation
cargo bench protocol_versions
cargo bench batch_generation

# View HTML reports
open target/criterion/report/index.html
```

See [BENCHMARKS.md](BENCHMARKS.md) for detailed performance analysis and results.

### Quick Performance Check

```bash
# Profile generation
cargo build --release
time ./target/release/pickle-fuzzer --dir samples --samples 10000
```
