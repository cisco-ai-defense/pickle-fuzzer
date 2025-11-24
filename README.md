# pickle-fuzzer

A structure-aware test case generator for Python pickle parsers and validators. `pickle-fuzzer` generates complex, valid pickle bytecode across all protocol versions (0-5) for use in fuzzing and testing pickle parsing implementations.

## Project Description

`pickle-fuzzer` is a Rust-based tool designed to help security researchers and developers test Python pickle parsing implementations. Unlike traditional fuzzers that generate random bytes, `pickle-fuzzer` understands pickle's structure and generates syntactically valid pickle bytecode that exercises edge cases, complex opcode sequences, and protocol-specific features.

**Key Use Cases:**
- Fuzzing pickle parsers and validators for security vulnerabilities
- Testing pickle implementations across different Python versions
- Generating test cases for custom pickle-based serialization systems
- Discovering edge cases in pickle handling code

## Overview

`pickle-fuzzer` provides a structure-aware approach to generating pickle test cases with proper opcode sequencing, stack/memo simulation, and protocol version compliance. It produces diverse pickle bytecode that can be used with fuzzing frameworks or standalone testing to discover bugs and edge cases in pickle parsing implementations.

## Features

- **Multi-Protocol Support**: Generate pickles for all protocol versions (0-5)
- **Stack/Memo Simulation**: Simulates pickle machine stack and memo to ensure valid opcode sequences
- **Comprehensive Opcode Coverage**: Supports all standard pickle opcodes including FRAME, EXT, GLOBAL, etc.
- **Parallel Generation**: Generate multiple pickle files concurrently
- **Configurable Output**: Single file or batch generation modes
- **Deterministic Fuzzing**: Optional seed-based generation for reproducibility

## Installation

### Prerequisites

- Rust 1.70 or later

### Building from Source

```bash
git clone https://github.com/cisco-ai-defense/pickle-fuzzer
cd pickle-fuzzer
cargo build --release
```

The binary will be available at `target/release/pickle-fuzzer`.

**Build Requirements:**
- Rust toolchain 1.70 or later
- Standard C toolchain (for some dependencies)

### Installing from Crates.io

```bash
cargo install pickle-fuzzer
```

## Usage

### Generate a Single Pickle File

```bash
# Generate a random pickle file
pickle-fuzzer output.pkl

# The protocol version is randomly selected (0-5)
```

### Generate Multiple Pickle Files

```bash
# Generate 100 pickle files in the samples directory
pickle-fuzzer --dir samples --samples 100

# Files will be named 0.pkl, 1.pkl, 2.pkl, etc.
```

### Command-Line Options

```
Usage: pickle-fuzzer [OPTIONS] [FILE]

Arguments:
  [FILE]  Output file path (for single file mode)

Options:
  -d, --dir <DIR>          Output directory for batch generation
  -n, --samples <SAMPLES>  Number of samples to generate [default: 1000]
  -h, --help              Print help
  -V, --version           Print version
```

## Python Bindings

`pickle-fuzzer` provides Python bindings for integration with Python-based fuzzing tools like Atheris.

### Installation

```bash
# Install from source with Python bindings
cd python
pip install maturin
maturin develop --release
```

### Basic Usage

```python
from pickle_fuzzer import Generator

# Create generator for protocol 3
gen = Generator(protocol=3)

# Generate a random pickle
pickle_bytes = gen.generate()

# Generate from fuzzer input (deterministic)
fuzzer_data = b"some_fuzzer_input"
pickle_bytes = gen.generate_from_bytes(fuzzer_data)

# Configure generation
gen.set_opcode_range(10, 50)  # Control pickle complexity
gen.reset()  # Reset internal state
```

### Integration with Atheris

Use the `PickleMutator` class for structure-aware fuzzing:

```python
import atheris
from pickle_fuzzer.fuzzer import PickleMutator
import pickle

mutator = PickleMutator(protocol=3)

@atheris.instrument_func
def test_one_input(data: bytes):
    # Generate valid pickle from fuzzer input
    pickle_bytes = mutator.mutate(data, max_size=10000)
    
    try:
        pickle.loads(pickle_bytes)
    except Exception:
        pass  # Expected - looking for crashes

atheris.Setup(sys.argv, test_one_input)
atheris.Fuzz()
```

See [python/examples/harness.py](python/examples/harness.py) for a complete example.

## Fuzzing pickle-fuzzer Itself

`pickle-fuzzer` includes comprehensive fuzz targets for testing its own generation logic using cargo-fuzz (libFuzzer).

### Quick Start

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# List available fuzz targets
cargo fuzz list

# Run fast protocol fuzzing
cargo fuzz run all_protocols

# Run with Python validation (slower but thorough)
cargo fuzz run validate_with_python
```

### Available Fuzz Targets

- **`all_protocols`**: Fast fuzzing of all protocols (0-5) with structural validation (~5000-10000 execs/sec)
- **`validate_with_python`**: Comprehensive validation with Python's `pickletools.genops()` (same logic as `scripts/validate-pickles.py`) including mutation testing (~100-500 execs/sec)

### Recommended Workflow

```bash
# Phase 1: Fast discovery (1-2 hours)
cargo fuzz run all_protocols -- -max_total_time=7200

# Phase 2: Thorough validation (30-60 minutes)
cargo fuzz run validate_with_python -- -max_total_time=3600
```

### Handling Crashes

```bash
# Reproduce a crash
cargo fuzz run all_protocols fuzz/artifacts/all_protocols/crash-abc123

# Minimize crashing input
cargo fuzz tmin all_protocols fuzz/artifacts/all_protocols/crash-abc123
```

For detailed fuzzing documentation, see [fuzz/README.md](fuzz/README.md).

## Fuzzing Other Python Projects with Atheris

`pickle-fuzzer` can be used to fuzz any Python project that parses pickle data.

### Fuzzing Custom Pickle Parsers

```python
#!/usr/bin/env python3
import atheris
import sys
from pickle_fuzzer.fuzzer import fuzz_pickle_parser

# Your custom pickle parser
def my_pickle_parser(data: bytes):
    # Your parsing logic here
    import pickle
    return pickle.loads(data)

if __name__ == "__main__":
    # Use structure-aware generation
    fuzz_pickle_parser(
        my_pickle_parser,
        protocol=4,
        use_structure_aware=True
    )
```

### Fuzzing with Custom Unpicklers

```python
import atheris
import pickle
from pickle_fuzzer.fuzzer import PickleMutator

class CustomUnpickler(pickle.Unpickler):
    def find_class(self, module, name):
        # Custom class resolution logic
        return super().find_class(module, name)

mutator = PickleMutator(protocol=3)

@atheris.instrument_func
def test_custom_unpickler(data: bytes):
    pickle_bytes = mutator.mutate(data, max_size=10000)
    try:
        CustomUnpickler(io.BytesIO(pickle_bytes)).load()
    except Exception:
        pass

atheris.Setup(sys.argv, test_custom_unpickler)
atheris.Fuzz()
```

### Running Atheris Fuzzing

```bash
# Install Atheris
uv add install atheris

# Run fuzzing campaign
uv run harness.py -max_total_time=3600

# With corpus
uv run harness.py corpus/ -max_total_time=3600

# Multiple workers
uv run harness.py -workers=4 -jobs=4
```

### Benefits of Structure-Aware Fuzzing

- **Higher Coverage**: Generates valid pickles that reach deeper code paths
- **Faster Bug Discovery**: Focuses on semantic bugs rather than parsing errors
- **Protocol Compliance**: Tests protocol-specific features and edge cases
- **Deterministic**: Same fuzzer input produces same pickle (reproducible bugs)

For more details on Atheris integration, see [ATHERIS_INTEGRATION_PLAN.md](ATHERIS_INTEGRATION_PLAN.md).

## How It Works

`pickle-fuzzer` uses a stack-based approach to generate valid pickle bytecode:

1. **Stack/Memo Simulation**: Maintains an internal stack and memo that mirrors the pickle machine's behavior
2. **Opcode Validation**: Only emits opcodes that are valid given the current stack state
3. **Protocol Compliance**: Respects protocol version constraints for opcode availability
4. **Random Generation**: Uses arbitrary crate for deterministic random data generation

### Example Generated Pickle

```bash
python3 scripts/validate-pickles.py samples/

[OK] samples/0.pkl
[OK] samples/1.pkl
[OK] samples/10.pkl
[OK] samples/100.pkl
[OK] samples/1000.pkl
[OK] samples/10000.pkl
[OK] samples/10001.pkl
[OK] samples/10002.pkl
...
[OK] samples/9997.pkl
[OK] samples/9998.pkl
[OK] samples/9999.pkl
Validated 50000 pickle file(s); 0 failure(s).
```

## Architecture Overview

`pickle-fuzzer` uses a simulation-based approach to generate valid pickle bytecode:

### Core Components

- **Generator** (`src/generator.rs`): Core test case generation engine with stack/memo simulation. Maintains internal state and emits valid opcode sequences.
- **Opcodes** (`src/opcodes.rs`): Complete opcode definitions for all protocol versions (0-5) with protocol-specific availability mappings.
- **Stack** (`src/stack.rs`): Simulates the pickle virtual machine stack, tracking all stack objects and their types.
- **State** (`src/state.rs`): Manages generator state including memo table, protocol version, and stack state.
- **Mutators** (`src/mutators/`): Optional mutation strategies for introducing controlled variations (bit flips, boundary values, type confusion, etc.).

### Generation Process

1. **Initialization**: Select protocol version and initialize stack/memo state
2. **Opcode Selection**: Choose valid opcodes based on current stack state and protocol version
3. **Argument Generation**: Generate appropriate arguments for selected opcodes
4. **Stack Simulation**: Update internal stack/memo to reflect opcode effects
5. **Validation**: Ensure final state is valid (exactly one item on stack)
6. **Output**: Emit complete pickle bytecode

## Performance

`pickle-fuzzer` is highly optimized for fast pickle generation with excellent scalability.

### Benchmark Results

| Metric | Performance |
|--------|-------------|
| **Small pickles** (10-30 opcodes) | ~5.3 µs |
| **Medium pickles** (60-300 opcodes) | ~53 µs |
| **Large pickles** (500-1000 opcodes) | ~610 µs |
| **Single-threaded throughput** | ~8,400 pickles/sec |
| **Multi-core (8 cores)** | ~67,000 pickles/sec |

### Protocol Performance

| Protocol | Time | Use Case |
|----------|------|----------|
| V0 | 36 µs | ASCII-based, legacy |
| V1 | 48 µs | **Fastest**, binary |
| V2 | 50 µs | Fast, PROTO opcode |
| V3 | 53 µs | **Default**, balanced |
| V4 | 171 µs | FRAME support |
| V5 | 161 µs | Out-of-band buffers |

### Running Benchmarks

```bash
# Install criterion (if needed)
cargo install cargo-criterion

# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench single_generation
cargo bench protocol_versions
cargo bench batch_generation

# View detailed HTML reports
open target/criterion/report/index.html
```

### Performance Tips

```bash
# Use seeded generation for 2x speedup
pickle-fuzzer --seed 42 output.pkl

# Use faster protocols (V1 or V2)
pickle-fuzzer --protocol 1 output.pkl

# Smaller opcode ranges generate faster
pickle-fuzzer --min-opcodes 10 --max-opcodes 50 output.pkl
```

For detailed benchmark analysis, see [BENCHMARKS.md](BENCHMARKS.md).

## Safety Warning

**Important**: `pickle-fuzzer` generates potentially malicious pickle data for testing purposes only. 

- **DO NOT** use generated pickles in production systems
- **DO NOT** unpickle generated data without proper sandboxing
- **DO** use in isolated testing environments only
- **DO** follow responsible disclosure for any vulnerabilities found

## Development Setup

For contributors and developers working on `pickle-fuzzer`:

### Setting Up Your Environment

```bash
# Clone the repository
git clone https://github.com/cisco-ai-defense/pickle-fuzzer
cd pickle-fuzzer

# Build in debug mode
cargo build

# Run tests
cargo test

# Run with local changes
cargo run -- output.pkl
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Generate coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

### Code Quality Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run all checks before committing
cargo fmt && cargo clippy -- -D warnings && cargo test
```

For detailed development guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Documentation

### Core Documentation
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute to the project
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - Community guidelines and expectations
- [TESTING.md](TESTING.md) - Testing procedures and validation
- [BENCHMARKS.md](BENCHMARKS.md) - Performance benchmarks and optimization

### Fuzzing Documentation
- [fuzz/README.md](fuzz/README.md) - Fuzzing pickle-fuzzer itself with cargo-fuzz
- [ATHERIS_INTEGRATION_PLAN.md](ATHERIS_INTEGRATION_PLAN.md) - Atheris integration and Python fuzzing
- [CARGO_FUZZ_PLAN.md](CARGO_FUZZ_PLAN.md) - cargo-fuzz integration details
- [FUZZING_COMPARISON.md](FUZZING_COMPARISON.md) - Comparison with other fuzzing approaches

## Community Resources

### Getting Help
- **Issues**: Report bugs or request features via [GitHub Issues](https://github.com/cisco-ai-defense/pickle-fuzzer/issues)
- **Discussions**: Ask questions and share ideas in [GitHub Discussions](https://github.com/cisco-ai-defense/pickle-fuzzer/discussions)
- **Security**: Report security vulnerabilities responsibly (see Contributing guide)

### Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:

- Reporting bugs and security issues
- Suggesting features and improvements
- Submitting pull requests
- Code style guidelines and testing requirements
- Development workflow

## Versioning

This project follows [Semantic Versioning 2.0.0](https://semver.org/).

Given a version number `MAJOR.MINOR.PATCH`:
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible new functionality
- **PATCH**: Backwards-compatible bug fixes

**Current version**: 0.1.0 (pre-release)

Pre-1.0 versions (0.x.x) may introduce breaking changes in minor versions as the API stabilizes.

## License

Distributed under the Apache 2.0 License. See [LICENSE](LICENSE) for more information.

Project Link: [https://github.com/cisco-ai-defense/pickle-fuzzer](https://github.com/cisco-ai-defense/pickle-fuzzer)

