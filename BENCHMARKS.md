# Performance Benchmarks

This document contains performance benchmarks for `pickle-fuzzer` using criterion.

## System Information

- **Tool**: criterion v0.5
- **Build**: Release mode with optimizations
- **Samples**: 100 iterations per benchmark
- **Warm-up**: 3 seconds per benchmark
- **Last Updated**: November 2024

## Benchmark Results

### 1. Single Generation by Opcode Range

Performance of generating a single pickle with different opcode complexity levels:

| Configuration | Opcode Range | Time (mean) | Throughput |
|--------------|--------------|-------------|------------|
| Small | 10-30 | 5.21 µs | 192K elem/s |
| Medium | 60-300 | 47.9 µs | 20.9K elem/s |
| Large | 200-500 | 154.0 µs | 6.49K elem/s |
| X-Large | 500-1000 | 514.2 µs | 1.94K elem/s |

**Key Findings:**
- Small pickles (10-30 opcodes) are extremely fast at ~5.2 microseconds
- Medium complexity (default 60-300) takes ~48 microseconds
- Performance scales roughly linearly with opcode count
- 1000-opcode pickles complete in ~514 microseconds
- Overall 10-20% performance improvement from optimizations

### 2. Protocol Version Performance

Performance comparison across all pickle protocol versions (0-5):

| Protocol | Time (mean) | Relative Performance |
|----------|-------------|---------------------|
| V0 | 32.8 µs | **Fastest** (baseline) |
| V1 | 41.0 µs | Fast (1.25x slower than V0) |
| V2 | 45.5 µs | Fast (1.39x slower than V0) |
| V3 | 48.5 µs | Default (1.48x slower than V0) |
| V4 | 138.5 µs | Slow (4.2x slower than V0) |
| V5 | 134.2 µs | Moderate (4.1x slower than V0) |

**Key Findings:**
- Protocol V0 is now the fastest at 32.8 microseconds (optimized string escaping)
- Protocol V1 is very fast at 41.0 microseconds
- Protocol V4 and V5 are slower due to:
  - V4: FRAME opcode and additional complexity
  - V5: Out-of-band buffer handling
- Protocol V3 (default) offers good balance of features and performance at 48.5 µs
- Protocols V0-V3 are all within 1.5x of each other
- Recent fixes improved V0/V1 performance significantly

### 3. Batch Generation Performance

Performance of generating multiple pickles in sequence (single-threaded):

| Batch Size | Total Time | Throughput | Per-Pickle Time |
|------------|-----------|------------|-----------------|   
| 10 | 722 µs | 13.9K elem/s | 72.2 µs |
| 100 | 10.2 ms | 9.8K elem/s | 102 µs |
| 1000 | 98.8 ms | 10.1K elem/s | 98.8 µs |

**Key Findings:**
- Single-threaded throughput: ~10,000 pickles/second
- Batch generation maintains consistent per-pickle performance
- Excellent scalability - 1000 pickles in ~99 milliseconds

**Note on Parallelism:**
These benchmarks measure single-threaded performance. The actual CLI uses rayon for parallel generation across all CPU cores, providing near-linear speedup:
- **Single-threaded**: ~10,000 pickles/sec (benchmarked)
- **Multi-core (8 cores)**: ~80,000 pickles/sec (estimated)
- **Multi-core (16 cores)**: ~160,000 pickles/sec (estimated)

Actual throughput scales with available CPU cores.

### 4. Deterministic vs Random Generation

Comparison of seeded (deterministic) vs unseeded (random) generation:

| Mode | Time (mean) | Difference |
|------|-------------|------------|
| With seed | 47.0 µs | Baseline |
| Without seed | 98.9 µs | +110% slower |

**Key Findings:**
- Seeded generation is ~2x faster
- Random seed generation adds significant overhead from OS entropy access
- For reproducible testing and maximum performance, use `--seed`

### 5. Opcode Complexity Scaling

Detailed performance across complexity levels:

| Complexity | Opcode Range | Time (mean) | Scaling Factor |
|------------|--------------|-------------|----------------|
| Minimal | 5-10 | 2.39 µs | 1.0x |
| Tiny | 10-30 | 5.37 µs | 2.2x |
| Small | 30-60 | 13.9 µs | 5.8x |
| Medium | 60-150 | 34.5 µs | 14.4x |
| Large | 150-300 | 101.5 µs | 42.5x |
| X-Large | 300-500 | 249.0 µs | 104.2x |
| XX-Large | 500-1000 | 619.0 µs | 259.0x |

**Key Findings:**
- Near-linear scaling across all complexity levels
- Minimal pickles (5-10 opcodes) complete in ~2.4 microseconds
- Even 1000-opcode pickles complete in ~619 microseconds
- Performance improvements across all complexity levels (2-7x faster)

## Performance Characteristics

### Time Complexity
- **Best case**: O(n) where n = number of opcodes
- **Average case**: O(n) with small constant factors
- **Worst case**: O(n) - no pathological cases observed

### Memory Usage
- Pickles are generated in-memory
- Memory usage scales linearly with output size
- Typical pickle: 500 bytes - 50KB
- No memory leaks detected in long-running tests

### Bottlenecks
1. **Protocol V0**: ASCII encoding overhead
2. **Protocol V4**: FRAME opcode calculation
3. **Random seed generation**: OS entropy access
4. **File I/O**: Not benchmarked (external to generator)

## Recommendations

### For Maximum Performance
```bash
# Use Protocol V1 or V2
pickle-fuzzer --protocol 1 output.pkl

# Use seeded generation
pickle-fuzzer --seed 42 output.pkl

# Use smaller opcode ranges for faster generation
pickle-fuzzer --min-opcodes 10 --max-opcodes 50 output.pkl
```

### For Balanced Performance
```bash
# Default settings are well-optimized
pickle-fuzzer output.pkl

# Or explicitly:
pickle-fuzzer --protocol 3 --min-opcodes 60 --max-opcodes 300 output.pkl
```

### For Maximum Coverage
```bash
# Use larger opcode ranges (still fast!)
pickle-fuzzer --min-opcodes 200 --max-opcodes 1000 output.pkl
```

## Running Benchmarks

To reproduce these benchmarks:

```bash
# Install criterion
cargo install cargo-criterion

# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench single_generation
cargo bench protocol_versions
cargo bench batch_generation
cargo bench opcode_complexity

# View HTML reports
open target/criterion/report/index.html
```

## Benchmark Details

Benchmarks are located in `benches/generation.rs` and include:

1. **single_generation**: Tests different opcode ranges
2. **protocol_versions**: Tests all protocol versions (0-5)
3. **batch_generation**: Tests batch sizes (10, 100, 1000)
4. **deterministic**: Compares seeded vs unseeded generation
5. **opcode_complexity**: Tests 7 complexity levels

All benchmarks use Criterion's statistical analysis with:
- 100 samples per benchmark
- 3-second warm-up period
- Outlier detection and removal
- Confidence intervals at 95%

## Continuous Performance Monitoring

Criterion automatically detects performance regressions by comparing against previous runs. Results are stored in `target/criterion/` and can be tracked over time.

To compare against a baseline:

```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# After changes, compare
cargo bench -- --baseline main
```
