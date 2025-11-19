# Fuzzing pickle-whip with cargo-fuzz

This directory contains fuzz targets for testing pickle-whip's generation logic using libFuzzer via cargo-fuzz.

## Quick Start

```bash
# Install cargo-fuzz (one-time setup)
cargo install cargo-fuzz

# List all fuzz targets
cargo fuzz list

# Run basic generation fuzzing
cargo fuzz run generate_pickle

# Run with Python validation (slower but more thorough)
cargo fuzz run validate_with_pickletools
```

## Fuzz Targets

### 1. `generate_pickle` - Basic Generation
**Purpose**: Fast fuzzing of core generation logic  
**Validation**: Basic structural checks (non-empty, ends with STOP)  
**Speed**: ~5000-10000 execs/sec  
**Use**: Quick smoke testing, finding crashes

```bash
cargo fuzz run generate_pickle -- -max_total_time=3600
```

### 2. `validate_with_pickletools` - Python Validation
**Purpose**: Validate generated pickles with `pickletools.dis()`  
**Validation**: Full bytecode structure validation via Python  
**Speed**: ~100-500 execs/sec (subprocess overhead)  
**Use**: Ensuring generated pickles are valid

```bash
cargo fuzz run validate_with_pickletools -- -max_total_time=1800
```

**Why pickletools.dis()?**
- ✅ Safer than `pickle.loads()` (no code execution)
- ✅ Stricter validation (checks bytecode structure)
- ✅ Better error messages (opcode-level details)

### 3. `all_protocols` - Protocol Coverage
**Purpose**: Test all pickle protocols (0-5)  
**Validation**: Protocol-specific checks (PROTO opcode presence)  
**Speed**: ~5000-10000 execs/sec  
**Use**: Ensuring all protocol versions work

```bash
cargo fuzz run all_protocols -- -max_total_time=3600
```

### 4. `mutate_pickle` - Mutation Stress Test
**Purpose**: Test mutation system with various configurations  
**Validation**: Basic structural checks  
**Speed**: ~3000-5000 execs/sec  
**Use**: Finding bugs in mutators

```bash
cargo fuzz run mutate_pickle -- -max_total_time=3600
```

### 5. `opcode_ranges` - Range Testing
**Purpose**: Test opcode count constraints  
**Validation**: Opcode count within specified range  
**Speed**: ~5000-10000 execs/sec  
**Use**: Ensuring range constraints are respected

```bash
cargo fuzz run opcode_ranges -- -max_total_time=3600
```

## Recommended Workflow

### Phase 1: Fast Discovery (1 hour)
```bash
# Run fast targets to find obvious bugs
cargo fuzz run generate_pickle -- -max_total_time=1800
cargo fuzz run all_protocols -- -max_total_time=1800
```

### Phase 2: Thorough Validation (30 minutes)
```bash
# Validate with Python
cargo fuzz run validate_with_pickletools -- -max_total_time=1800
```

### Phase 3: Stress Testing (1 hour)
```bash
# Test mutations and edge cases
cargo fuzz run mutate_pickle -- -max_total_time=1800
cargo fuzz run opcode_ranges -- -max_total_time=1800
```

## Corpus Management

### Generate Initial Corpus
```bash
# Use pickle-whip CLI to generate diverse samples
mkdir -p corpus/generate_pickle
cargo run --release -- --dir corpus/generate_pickle --samples 100
```

### Run with Corpus
```bash
cargo fuzz run generate_pickle corpus/generate_pickle
```

### Minimize Corpus
```bash
# Remove redundant test cases
cargo fuzz cmin generate_pickle
```

### Merge Corpora
```bash
# Combine multiple corpora
cargo fuzz run generate_pickle -- \
    -merge=1 \
    corpus/generate_pickle \
    corpus/all_protocols
```

## Handling Crashes

### Reproduce a Crash
```bash
# Run specific crashing input
cargo fuzz run generate_pickle fuzz/artifacts/generate_pickle/crash-abc123
```

### Minimize Crashing Input
```bash
# Find smallest input that triggers crash
cargo fuzz tmin generate_pickle fuzz/artifacts/generate_pickle/crash-abc123
```

### Debug with Assertions
```bash
# Build with debug assertions
cargo fuzz run --debug-assertions generate_pickle fuzz/artifacts/...
```

## Coverage Analysis

### Generate Coverage Report
```bash
# Run fuzzing with coverage instrumentation
cargo fuzz coverage generate_pickle

# Generate HTML report (requires llvm-cov)
cargo cov -- show \
    target/x86_64-unknown-linux-gnu/release/generate_pickle \
    --format=html \
    --instr-profile=fuzz/coverage/generate_pickle/coverage.profdata \
    --output-dir=fuzz/coverage/html
```

### View Coverage
```bash
open fuzz/coverage/html/index.html
```

## Performance Tuning

### Increase Speed
```bash
# Limit max input size
cargo fuzz run generate_pickle -- -max_len=1000

# Increase RSS limit
cargo fuzz run generate_pickle -- -rss_limit_mb=4096

# Use multiple workers
cargo fuzz run generate_pickle -- -workers=4
```

### Focus on Specific Areas
```bash
# Use dictionary for guided fuzzing
cargo fuzz run generate_pickle -- -dict=dictionaries/pickle.dict

# Focus on specific opcodes
cargo fuzz run generate_pickle -- -focus_function=emit_int
```

## Continuous Fuzzing

### Run Overnight
```bash
# 8-hour campaign
nohup cargo fuzz run generate_pickle -- \
    -max_total_time=28800 \
    -print_final_stats=1 \
    > fuzz_log.txt 2>&1 &
```

### Monitor Progress
```bash
# Watch fuzzing stats
tail -f fuzz_log.txt

# Check corpus growth
watch -n 60 'ls -lh corpus/generate_pickle | wc -l'
```

## Validation Comparison

### pickletools.dis() vs pickle.loads()

| Aspect | pickletools.dis() | pickle.loads() |
|--------|-------------------|----------------|
| **Safety** | ✅ No execution | ⚠️ Executes code |
| **Strictness** | ✅ Validates structure | ⚠️ Lenient |
| **Speed** | ✅ Fast parsing | ⚠️ Slower |
| **Errors** | ✅ Detailed | ⚠️ Generic |
| **Use Case** | Validation | Integration testing |

**Recommendation**: Use `pickletools.dis()` for fuzzing validation.

## Troubleshooting

### "Python3 not found"
```bash
# Install Python 3
brew install python3  # macOS
sudo apt install python3  # Linux
```

### "Slow execution"
- Use `generate_pickle` instead of `validate_with_pickletools` for speed
- Reduce `-max_len` to limit input size
- Increase `-rss_limit_mb` if running out of memory

### "No new coverage"
- Minimize corpus: `cargo fuzz cmin generate_pickle`
- Try different targets: `all_protocols`, `mutate_pickle`
- Use dictionary: `-dict=dictionaries/pickle.dict`

## Integration with CI

See `.github/workflows/fuzz.yml` for continuous fuzzing setup.

## Resources

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer docs](https://llvm.org/docs/LibFuzzer.html)
- [Python pickletools](https://docs.python.org/3/library/pickletools.html)
