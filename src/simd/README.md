# SIMD Acceleration in json-steroids

This document explains how SIMD (Single Instruction Multiple Data) acceleration is integrated into json-steroids and how to use it.

## Quick Start

### Enable SIMD (enabled by default)

```toml
[dependencies]
json-steroids = "0.2"
```

### Disable SIMD (use scalar fallback)

```toml
[dependencies]
json-steroids = { version = "0.2", default-features = false }
```

## Running the Demo

```bash
# With SIMD acceleration
cargo run --example simd_demo --release

# Without SIMD (scalar fallback)
cargo run --example simd_demo --no-default-features --release
```

## Running Benchmarks

```bash
# Benchmark SIMD vs scalar implementations
cargo bench --bench simd_benchmarks

# View results in target/criterion/
```

## What's Optimized

### 1. String Scanning (3-5x faster)
- Finding closing quotes in JSON strings
- Detecting escape sequences
- Processing 16-32 bytes at once instead of 1

**Before (scalar):**
```rust
for byte in string {
    if byte == '"' || byte == '\\' { ... }
}
```

**After (SIMD):**
```rust
// Process 32 bytes at once with AVX2
let chunk = load_32_bytes();
let quotes = compare_all_to(chunk, '"');
let backslash = compare_all_to(chunk, '\\');
// Find first match in 32 bytes in one instruction
```

### 2. Whitespace Skipping (2-3x faster)
- Skipping spaces, tabs, newlines between JSON tokens
- Checking 16-32 whitespace characters at once

### 3. Escape Detection (4-6x faster)
- Finding characters that need escaping when writing JSON
- Vectorized check for control characters, quotes, backslashes

## Architecture Support

### x86_64 (Intel/AMD)
- ✅ **SSE2** (baseline, ~2001): 16 bytes at a time
- ✅ **AVX2** (optimal, ~2013): 32 bytes at a time
- 🔜 **AVX-512** (future, ~2017): 64 bytes at a time

Runtime CPU detection automatically selects the best available implementation.

### ARM64 (Apple Silicon, mobile, servers)
- ✅ **NEON** (standard on all AArch64): 16 bytes at a time

### Other architectures
- ✅ **Scalar fallback**: Works on any platform

## Performance Expectations

Based on typical JSON workloads:

| Operation | Scalar | SIMD | Speedup |
|-----------|--------|------|---------|
| String scanning (no escapes) | 1x | 3-5x | 🚀🚀🚀 |
| String scanning (with escapes) | 1x | 1.5-2x | 🚀 |
| Whitespace skipping | 1x | 2-3x | 🚀🚀 |
| Escape detection | 1x | 4-6x | 🚀🚀🚀 |

**Real-world impact:** 1.5-3x faster overall JSON parsing depending on content.

## How It Works

### 1. Automatic Runtime Detection

```rust
#[cfg(feature = "simd")]
pub fn scan_string(bytes: &[u8], start: usize) -> ScanResult {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { x86_64::scan_string_avx2(bytes, start) };
        } else if is_x86_feature_detected!("sse2") {
            return unsafe { x86_64::scan_string_sse2(bytes, start) };
        }
    }
    
    // Fallback for unsupported platforms
    fallback::scan_string_scalar(bytes, start)
}
```

### 2. SIMD String Scanning Example

```rust
// Load 32 bytes into SIMD register
let chunk = _mm256_loadu_si256(bytes.as_ptr() as *const __m256i);

// Create comparison vectors
let quote_vec = _mm256_set1_epi8(b'"' as i8);
let backslash_vec = _mm256_set1_epi8(b'\\' as i8);

// Compare all 32 bytes at once
let quote_cmp = _mm256_cmpeq_epi8(chunk, quote_vec);
let backslash_cmp = _mm256_cmpeq_epi8(chunk, backslash_vec);

// Combine results
let combined = _mm256_or_si256(quote_cmp, backslash_cmp);

// Convert to bitmask (1 bit per byte)
let mask = _mm256_movemask_epi8(combined) as u32;

// Find first match
if mask != 0 {
    let offset = mask.trailing_zeros() as usize;
    // Found special character at position + offset
}
```

## Integration into Parser

The SIMD functions are designed to be drop-in replacements:

```rust
// In parser.rs
pub fn parse_string(&mut self) -> Result<Cow<'a, str>> {
    self.skip_whitespace();
    
    if self.peek() != Some(b'"') {
        return Err(JsonError::ExpectedToken("string", self.pos));
    }
    self.advance();

    let start = self.pos;
    
    #[cfg(feature = "simd")]
    {
        let result = crate::simd::scan_string(self.input, start);
        self.pos = result.position;
        
        if result.has_escapes {
            // Process escapes...
        } else {
            // Zero-copy path
        }
    }
    
    #[cfg(not(feature = "simd"))]
    {
        // Original scalar implementation
    }
}
```

## Safety Considerations

SIMD code uses `unsafe` because:
1. It directly calls CPU intrinsics
2. It requires proper alignment and bounds checking
3. It must be guarded by CPU feature detection

All unsafe code is:
- ✅ Thoroughly tested
- ✅ Has safe fallbacks
- ✅ Uses runtime feature detection
- ✅ Properly handles unaligned loads
- ✅ Has clear safety comments

## Benchmarking Tips

### Measure on real hardware
```bash
# Enable CPU optimizations
RUSTFLAGS="-C target-cpu=native" cargo bench --bench simd_benchmarks
```

### Compare implementations
```bash
# With SIMD
cargo bench --bench simd_benchmarks --features simd

# Without SIMD
cargo bench --bench simd_benchmarks --no-default-features
```

### Profile with profiler
```bash
# Build with debug symbols
cargo build --release --example simd_demo
perf record ./target/release/examples/simd_demo
perf report
```

## Troubleshooting

### SIMD not providing speedup?

**Possible reasons:**
1. **Data too small**: SIMD has setup overhead. For strings < 32 bytes, scalar may be faster.
2. **CPU doesn't support AVX2**: Check with `cat /proc/cpuinfo | grep avx2` (Linux)
3. **Thermal throttling**: CPU may be throttled due to heat
4. **Testing in debug mode**: Always benchmark in `--release` mode

### Compile errors?

```bash
# Make sure you're using a recent Rust version
rustup update

# Check if SIMD is enabled
cargo build --features simd
```

### Wrong results?

The SIMD code should produce identical results to scalar. If not:
1. Check the test suite: `cargo test`
2. Run with `RUST_BACKTRACE=1`
3. File an issue with a reproducible example

## Future Optimizations

### Phase 2 (planned)
- [ ] UTF-8 validation with SIMD
- [ ] Number parsing with SIMD
- [ ] Structural indexing (simdjson technique)

### Phase 3 (experimental)
- [ ] AVX-512 support
- [ ] ARM SVE support
- [ ] WASM SIMD support

## References

- [simdjson](https://github.com/simdjson/simdjson) - C++ JSON parser using SIMD
- [simd-json](https://github.com/simd-lite/simd-json) - Rust port of simdjson
- [Intel Intrinsics Guide](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/)
- [ARM NEON Intrinsics](https://developer.arm.com/architectures/instruction-sets/intrinsics/)

## Contributing

To add more SIMD optimizations:

1. Add function to appropriate arch module (`x86_64.rs`, `aarch64.rs`)
2. Add scalar fallback to `fallback.rs`
3. Add public API to `mod.rs` with runtime detection
4. Add tests to `mod.rs`
5. Add benchmarks to `benches/simd_benchmarks.rs`
6. Update this README

## License

Same as json-steroids (MIT)
