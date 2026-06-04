//! Benchmark comparing SIMD vs scalar implementations
//!
//! Run with:
//! cargo bench --bench simd_benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

#[cfg(feature = "simd")]
use json_steroids::simd;

#[cfg(feature = "simd")]
fn bench_string_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_scanning");

    // Test different string lengths
    for size in [16, 64, 256, 1024, 4096, 16384].iter() {
        let data = "x".repeat(*size);
        let bytes = data.as_bytes();

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            b.iter(|| simd::scan_string(black_box(bytes), black_box(0)));
        });

        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter(|| simd::fallback::scan_string_scalar(black_box(bytes), black_box(0)));
        });
    }

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_whitespace_skipping(c: &mut Criterion) {
    let mut group = c.benchmark_group("whitespace_skipping");

    // Test different amounts of whitespace
    for size in [16, 64, 256, 1024].iter() {
        let data = " ".repeat(*size);
        let bytes = data.as_bytes();

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            b.iter(|| simd::skip_whitespace(black_box(bytes), black_box(0)));
        });

        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter(|| simd::fallback::skip_whitespace_scalar(black_box(bytes), black_box(0)));
        });
    }

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_escape_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("escape_detection");

    // Test clean strings (no escapes needed)
    for size in [16, 64, 256, 1024, 4096].iter() {
        let data = "abcdefghijklmnop".repeat(*size / 16);
        let bytes = data.as_bytes();

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("simd_clean", size), size, |b, _| {
            b.iter(|| simd::find_escape_needed(black_box(bytes), black_box(0)));
        });

        group.bench_with_input(BenchmarkId::new("scalar_clean", size), size, |b, _| {
            b.iter(|| simd::fallback::find_escape_needed_scalar(black_box(bytes), black_box(0)));
        });
    }

    // Test strings with escapes at the end
    let dirty_data = format!("{}{}", "x".repeat(1000), "\"");
    let dirty_bytes = dirty_data.as_bytes();

    group.bench_function("simd_escape_at_end", |b| {
        b.iter(|| simd::find_escape_needed(black_box(dirty_bytes), black_box(0)));
    });

    group.bench_function("scalar_escape_at_end", |b| {
        b.iter(|| simd::fallback::find_escape_needed_scalar(black_box(dirty_bytes), black_box(0)));
    });

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_realistic_json_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_json");

    // Realistic JSON with mixed content
    let json = r#"{
        "name": "John Doe",
        "age": 30,
        "email": "john.doe@example.com",
        "address": {
            "street": "123 Main St",
            "city": "Anytown",
            "country": "USA"
        },
        "tags": ["developer", "rust", "json"],
        "active": true,
        "balance": 1234.56
    }"#;

    let bytes = json.as_bytes();

    // Benchmark scanning all strings in the JSON
    group.bench_function("scan_all_strings_simd", |b| {
        b.iter(|| {
            let mut pos = 0;
            let mut count = 0;
            while pos < bytes.len() {
                if bytes[pos] == b'"' {
                    let result = simd::scan_string(black_box(bytes), black_box(pos + 1));
                    pos = result.position + 1;
                    count += 1;
                } else {
                    pos += 1;
                }
            }
            black_box(count)
        });
    });

    group.bench_function("scan_all_strings_scalar", |b| {
        b.iter(|| {
            let mut pos = 0;
            let mut count = 0;
            while pos < bytes.len() {
                if bytes[pos] == b'"' {
                    let result =
                        simd::fallback::scan_string_scalar(black_box(bytes), black_box(pos + 1));
                    pos = result.position + 1;
                    count += 1;
                } else {
                    pos += 1;
                }
            }
            black_box(count)
        });
    });

    group.finish();
}

#[cfg(feature = "simd")]
criterion_group!(
    benches,
    bench_string_scanning,
    bench_whitespace_skipping,
    bench_escape_detection,
    bench_realistic_json_parse
);

#[cfg(not(feature = "simd"))]
criterion_group!(benches,);

criterion_main!(benches);
