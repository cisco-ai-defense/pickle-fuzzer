# Fuzzing pickle-whip with cargo-fuzz

This directory contains fuzz targets for testing pickle-whip's generation logic using libFuzzer via cargo-fuzz.

## Quick Start

```bash
# Install cargo-fuzz (one-time setup)
cargo install cargo-fuzz

# List all fuzz targets
cargo fuzz list

# Run fast protocol fuzzing
cargo fuzz run all_protocols

# Run with Python validation (slower but more thorough)
cargo fuzz run validate_with_python
```

## Fuzz Targets

### 1. `all_protocols` - Fast Protocol Coverage
**Purpose**: Test all pickle protocols (0-5) with core generation logic  
**Validation**: Protocol-specific checks (PROTO opcode presence, structural validation)  
**Speed**: ~5000-10000 execs/sec  
**Use**: Primary fast fuzzing target for finding crashes and protocol bugs

```bash
cargo fuzz run all_protocols -- -max_total_time=3600
```

**What it tests:**
- All 6 protocol versions (V0-V5)
- Protocol-specific opcode constraints
- Stack and memo simulation
- Opcode emission logic

### 2. `validate_with_python` - Thorough Python Validation
**Purpose**: Comprehensive validation with Python's `pickletools.genops()`  
**Validation**: Full structural validation via Python interpreter (uses same logic as `scripts/validate-pickles.py`)  
**Speed**: ~100-500 execs/sec (subprocess overhead)  
**Use**: Ensuring generated pickles are structurally valid and parseable by Python

```bash
cargo fuzz run validate_with_python -- -max_total_time=1800
```

**What it tests:**
- All protocols (V0-V5)
- Opcode range configuration (min/max opcodes)
- Mutation system with all mutators
- Mutation rate configuration
- Python compatibility via `pickletools.genops()` validation

**Note**: This target spawns Python subprocesses to validate each generated pickle using the same validation logic as `scripts/validate-pickles.py`.

## Recommended Workflow

### Phase 1: Fast Discovery (1-2 hours)
```bash
# Run fast protocol fuzzing to find crashes
cargo fuzz run all_protocols -- -max_total_time=7200
```

### Phase 2: Thorough Validation (30-60 minutes)
```bash
# Validate with Python for semantic correctness
cargo fuzz run validate_with_python -- -max_total_time=3600
```

### Quick Smoke Test (5 minutes)
```bash
# Quick sanity check before commits
cargo fuzz run all_protocols -- -max_total_time=300
```

## Corpus Management

### Generate Initial Corpus
```bash
# Use pickle-whip CLI to generate diverse samples
mkdir -p corpus/all_protocols
cargo run --release -- --dir corpus/all_protocols --samples 100
```

### Run with Corpus
```bash
cargo fuzz run all_protocols corpus/all_protocols
```

### Minimize Corpus
```bash
# Remove redundant test cases
cargo fuzz cmin all_protocols
```

### Merge Corpora
```bash
# Combine multiple corpora
cargo fuzz run all_protocols -- \
    -merge=1 \
    corpus/all_protocols \
    corpus/validate_with_python
```

## Handling Crashes

### Reproduce a Crash
```bash
# Run specific crashing input
cargo fuzz run all_protocols fuzz/artifacts/all_protocols/crash-abc123
```

### Minimize Crashing Input
```bash
# Find smallest input that triggers crash
cargo fuzz tmin all_protocols fuzz/artifacts/all_protocols/crash-abc123
```

### Debug with Assertions
```bash
# Build with debug assertions
cargo fuzz run --debug-assertions all_protocols fuzz/artifacts/...
```

## Coverage Analysis

### Generate Coverage Report
```bash
# Run fuzzing with coverage instrumentation
cargo fuzz coverage all_protocols

# Generate HTML report (requires llvm-cov)
cargo cov -- show \
    target/x86_64-unknown-linux-gnu/release/all_protocols \
    --format=html \
    --instr-profile=fuzz/coverage/all_protocols/coverage.profdata \
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
cargo fuzz run all_protocols -- -max_len=1000

# Increase RSS limit
cargo fuzz run all_protocols -- -rss_limit_mb=4096

# Use multiple workers
cargo fuzz run all_protocols -- -workers=4
```

### Focus on Specific Areas
```bash
# Use dictionary for guided fuzzing
cargo fuzz run all_protocols -- -dict=dictionaries/pickle.dict

# Focus on specific opcodes
cargo fuzz run all_protocols -- -focus_function=emit_int
```

## Continuous Fuzzing

### Run Overnight
```bash
# 8-hour campaign
nohup cargo fuzz run all_protocols -- \
    -max_total_time=28800 \
    -print_final_stats=1 \
    > fuzz_log.txt 2>&1 &
```

### Monitor Progress
```bash
# Watch fuzzing stats
tail -f fuzz_log.txt

# Check corpus growth
watch -n 60 'ls -lh corpus/all_protocols | wc -l'
```

## Target Comparison

| Aspect | all_protocols | validate_with_python |
|--------|---------------|----------------------|
| **Speed** | ⚡ Fast (5-10K exec/s) | 🐌 Slow (100-500 exec/s) |
| **Coverage** | All protocols | All protocols + mutations |
| **Validation** | Structural | Semantic (Python) |
| **Use Case** | Fast discovery | Thorough validation |
| **Best For** | Finding crashes | Ensuring correctness |

**Recommendation**: Start with `all_protocols` for speed, then validate with `validate_with_python`.

## Troubleshooting

### "Python3 not found"
```bash
# Install Python 3
brew install python3  # macOS
sudo apt install python3  # Linux
```

### "Slow execution"
- Use `all_protocols` instead of `validate_with_python` for speed
- Reduce `-max_len` to limit input size
- Increase `-rss_limit_mb` if running out of memory

### "No new coverage"
- Minimize corpus: `cargo fuzz cmin all_protocols`
- Switch targets: try `validate_with_python` for different code paths
- Use dictionary: `-dict=dictionaries/pickle.dict`

## Integration with CI

The project includes automated fuzzing via GitHub Actions (`.github/workflows/fuzz.yml`):

### Automatic Runs

- **Pull Requests & Pushes**: 5-minute fast fuzzing with `all_protocols`
- **Daily (2 AM UTC)**: 30-minute thorough fuzzing with `validate_with_python`
- **Daily**: Corpus minimization to remove redundant test cases

### Manual Runs

Trigger custom fuzzing via GitHub Actions UI:

1. Go to **Actions** → **Fuzz Testing** → **Run workflow**
2. Configure:
   - **Target**: `all_protocols` or `validate_with_python`
   - **Duration**: Seconds to run (default: 600)

### Crash Handling

If fuzzing finds crashes:
- ❌ Workflow fails with error
- 📦 Crash artifacts uploaded for download
- 🔍 Review artifacts to reproduce locally:
  ```bash
  # Download crash artifact from GitHub Actions
  cargo fuzz run all_protocols path/to/crash-file
  ```

### Corpus Management

- Corpus is cached between runs for faster fuzzing
- Daily minimization removes redundant test cases
- Corpus statistics reported in workflow summary

## Resources

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer docs](https://llvm.org/docs/LibFuzzer.html)
- [Python pickletools](https://docs.python.org/3/library/pickletools.html)
