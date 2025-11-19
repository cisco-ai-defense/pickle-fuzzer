// SPDX-License-Identifier: Apache-2.0
//
// Copyright 2025 Cisco Systems, Inc. and its affiliates
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pickle_whip::{Generator, Version};

fn bench_single_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_generation");

    // Benchmark different opcode ranges
    let configs = vec![
        ("small_10-30", 10, 30),
        ("medium_60-300", 60, 300),
        ("large_200-500", 200, 500),
        ("xlarge_500-1000", 500, 1000),
    ];

    for (name, min, max) in configs {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("opcodes", name),
            &(min, max),
            |b, &(min, max)| {
                b.iter(|| {
                    let mut gen = Generator::new(Version::V3)
                        .with_seed(42)
                        .with_opcode_range(min, max);
                    black_box(gen.generate().unwrap())
                });
            },
        );
    }

    group.finish();
}

fn bench_protocol_versions(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_versions");

    for version_num in 0..=5 {
        let version = Version::try_from(version_num).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(version_num),
            &version,
            |b, &version| {
                b.iter(|| {
                    let mut gen = Generator::new(version).with_seed(42);
                    black_box(gen.generate().unwrap())
                });
            },
        );
    }

    group.finish();
}

fn bench_batch_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_generation");

    let batch_sizes = vec![10, 100, 1000];

    for size in batch_sizes {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    let mut gen = Generator::new(Version::V3).with_seed(42 + i as u64);
                    black_box(gen.generate().unwrap());
                }
            });
        });
    }

    group.finish();
}

fn bench_deterministic_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("deterministic");

    group.bench_function("with_seed", |b| {
        b.iter(|| {
            let mut gen = Generator::new(Version::V3).with_seed(42);
            black_box(gen.generate().unwrap())
        });
    });

    group.bench_function("without_seed", |b| {
        b.iter(|| {
            let mut gen = Generator::new(Version::V3);
            black_box(gen.generate().unwrap())
        });
    });

    group.finish();
}

fn bench_opcode_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("opcode_complexity");

    // Test different complexity levels
    let configs = vec![
        ("minimal", 5, 10),
        ("tiny", 10, 30),
        ("small", 30, 60),
        ("medium", 60, 150),
        ("large", 150, 300),
        ("xlarge", 300, 500),
        ("xxlarge", 500, 1000),
    ];

    for (name, min, max) in configs {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(min, max),
            |b, &(min, max)| {
                b.iter(|| {
                    let mut gen = Generator::new(Version::V3)
                        .with_seed(42)
                        .with_opcode_range(min, max);
                    black_box(gen.generate().unwrap())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_generation,
    bench_protocol_versions,
    bench_batch_generation,
    bench_deterministic_generation,
    bench_opcode_complexity
);
criterion_main!(benches);
