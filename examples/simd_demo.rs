//! Example demonstrating SIMD-accelerated string parsing
//!
//! Run with:
//! cargo run --example simd_demo --features simd --release
//!
//! Compare with scalar version:
//! cargo run --example simd_demo --no-default-features --release

#[cfg(feature = "simd")]
use json_steroids::simd;

fn main() {
    #[cfg(feature = "simd")]
    {
        println!("🚀 SIMD acceleration is ENABLED");
        demonstrate_simd_operations();
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("🐌 SIMD acceleration is DISABLED (using scalar fallback)");
        println!("Enable with: cargo run --example simd_demo --features simd");
    }
}

#[cfg(feature = "simd")]
fn demonstrate_simd_operations() {
    println!("\n=== String Scanning Performance ===\n");

    // Test 1: Simple string without escapes
    let simple_json = br#"Hello, world! This is a long string without any escape sequences to test SIMD performance""#;

    let start = std::time::Instant::now();
    let result = simd::scan_string(simple_json, 0);
    let duration = start.elapsed();

    println!("Simple string (no escapes):");
    println!("  Length: {} bytes", simple_json.len());
    println!("  Position: {}", result.position);
    println!("  Has escapes: {}", result.has_escapes);
    println!("  Time: {:?}", duration);

    // Test 2: String with escape sequences
    let escaped_json = br#"Hello\nworld!\tThis\rhas\\many\"escape\x00sequences"#;

    let start = std::time::Instant::now();
    let result = simd::scan_string(escaped_json, 0);
    let duration = start.elapsed();

    println!("\nString with escapes:");
    println!("  Length: {} bytes", escaped_json.len());
    println!("  First special char at: {}", result.position);
    println!("  Has escapes: {}", result.has_escapes);
    println!("  Time: {:?}", duration);

    // Test 3: Whitespace skipping
    println!("\n=== Whitespace Skipping Performance ===\n");

    let whitespace_json = b"        \t\t\t\n\n\r\r        {\"key\": \"value\"}";

    let start = std::time::Instant::now();
    let pos = simd::skip_whitespace(whitespace_json, 0);
    let duration = start.elapsed();

    println!("Whitespace test:");
    println!("  Total length: {} bytes", whitespace_json.len());
    println!("  Whitespace chars: {}", pos);
    println!("  First non-ws char: '{}'", whitespace_json[pos] as char);
    println!("  Time: {:?}", duration);

    // Test 4: Escape detection for writing
    println!("\n=== Escape Detection Performance ===\n");

    let clean_string = b"This is a perfectly normal string with no special characters at all";

    let start = std::time::Instant::now();
    let pos = simd::find_escape_needed(clean_string, 0);
    let duration = start.elapsed();

    println!("Clean string (no escaping needed):");
    println!("  Length: {} bytes", clean_string.len());
    println!(
        "  First escape at: {} (end of string: {})",
        pos,
        pos == clean_string.len()
    );
    println!("  Time: {:?}", duration);

    let dirty_string = b"This string has \"quotes\" and \n newlines";

    let start = std::time::Instant::now();
    let pos = simd::find_escape_needed(dirty_string, 0);
    let duration = start.elapsed();

    println!("\nString needing escapes:");
    println!("  Length: {} bytes", dirty_string.len());
    println!(
        "  First escape at: {} (char: '{}')",
        pos, dirty_string[pos] as char
    );
    println!("  Time: {:?}", duration);

    // CPU feature detection info
    println!("\n=== CPU Features ===\n");

    #[cfg(target_arch = "x86_64")]
    {
        println!("Architecture: x86_64");
        println!("  SSE2: {}", is_x86_feature_detected!("sse2"));
        println!("  AVX2: {}", is_x86_feature_detected!("avx2"));
        println!("  AVX-512: {}", is_x86_feature_detected!("avx512f"));
    }

    #[cfg(target_arch = "aarch64")]
    {
        println!("Architecture: aarch64");
        println!("  NEON: available (built-in on all AArch64)");
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        println!(
            "Architecture: {} (using scalar fallback)",
            std::env::consts::ARCH
        );
    }

    // Benchmark with larger data
    println!("\n=== Large Data Benchmark ===\n");

    let large_string = "x".repeat(10_000);
    let large_bytes = large_string.as_bytes();

    let iterations = 10_000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = simd::scan_string(large_bytes, 0);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;
    let throughput = (large_bytes.len() as f64) / per_iteration.as_secs_f64() / 1_000_000_000.0;

    println!(
        "Scanning {} byte string {} times:",
        large_bytes.len(),
        iterations
    );
    println!("  Total time: {:?}", duration);
    println!("  Per iteration: {:?}", per_iteration);
    println!("  Throughput: {:.2} GB/s", throughput);
}
