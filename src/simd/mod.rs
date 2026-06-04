//! SIMD-accelerated operations for JSON parsing and writing
//!
//! This module provides SIMD-optimized implementations for:
//! - String scanning (finding quotes, escapes, control characters)
//! - Whitespace skipping
//! - String escaping detection
//!
//! The implementation automatically selects the best available SIMD level
//! at runtime (AVX2 > SSE2 > Scalar fallback).

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "aarch64")]
mod aarch64;

// Re-export fallback for benchmarking and testing
pub mod fallback;

/// Result of scanning for special characters in a string
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanResult {
    /// Position of the first special character, or length if none found
    pub position: usize,
    /// Whether any escape sequences were found
    pub has_escapes: bool,
}

/// Scan a byte slice for string terminator (") or escape sequences (\)
/// Returns the position of the first quote or escape, and whether escapes were found
#[inline]
pub fn scan_string(bytes: &[u8], start: usize) -> ScanResult {
    #[cfg(target_arch = "x86_64")]
    {
        // Runtime CPU feature detection
        if is_x86_feature_detected!("avx2") {
            return unsafe { x86_64::scan_string_avx2(bytes, start) };
        } else if is_x86_feature_detected!("sse2") {
            return unsafe { x86_64::scan_string_sse2(bytes, start) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        return unsafe { aarch64::scan_string_neon(bytes, start) };
    }

    // Fallback for other architectures or when SIMD is not available
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        fallback::scan_string_scalar(bytes, start)
    }

    #[cfg(target_arch = "x86_64")]
    {
        fallback::scan_string_scalar(bytes, start)
    }
}

/// Skip whitespace characters (space, tab, newline, carriage return)
/// Returns the position of the first non-whitespace character
#[inline]
pub fn skip_whitespace(bytes: &[u8], start: usize) -> usize {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { x86_64::skip_whitespace_avx2(bytes, start) };
        } else if is_x86_feature_detected!("sse2") {
            return unsafe { x86_64::skip_whitespace_sse2(bytes, start) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        return unsafe { aarch64::skip_whitespace_neon(bytes, start) };
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        fallback::skip_whitespace_scalar(bytes, start)
    }

    #[cfg(target_arch = "x86_64")]
    {
        fallback::skip_whitespace_scalar(bytes, start)
    }
}

/// Find the first byte that needs escaping in a string
/// Returns the position of the first such byte, or length if none found
#[inline]
pub fn find_escape_needed(bytes: &[u8], start: usize) -> usize {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { x86_64::find_escape_needed_avx2(bytes, start) };
        } else if is_x86_feature_detected!("sse2") {
            return unsafe { x86_64::find_escape_needed_sse2(bytes, start) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        return unsafe { aarch64::find_escape_needed_neon(bytes, start) };
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        fallback::find_escape_needed_scalar(bytes, start)
    }

    #[cfg(target_arch = "x86_64")]
    {
        fallback::find_escape_needed_scalar(bytes, start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_string_no_escapes() {
        let input = b"hello world\"remaining";
        let result = scan_string(input, 0);
        assert_eq!(result.position, 11);
        assert_eq!(result.has_escapes, false);
    }

    #[test]
    fn test_scan_string_with_escapes() {
        let input = b"hello\\nworld\"remaining";
        let result = scan_string(input, 0);
        assert_eq!(result.position, 12); // Position of closing quote
        assert_eq!(result.has_escapes, true);
    }

    #[test]
    fn test_scan_string_quote_only() {
        let input = b"\"";
        let result = scan_string(input, 0);
        assert_eq!(result.position, 0);
        assert_eq!(result.has_escapes, false);
    }

    #[test]
    fn test_skip_whitespace() {
        let input = b"   \t\n\r  hello";
        let pos = skip_whitespace(input, 0);
        assert_eq!(pos, 8);
        assert_eq!(input[pos], b'h');
    }

    #[test]
    fn test_skip_whitespace_none() {
        let input = b"hello world";
        let pos = skip_whitespace(input, 0);
        assert_eq!(pos, 0);
    }

    #[test]
    fn test_find_escape_needed() {
        let input = b"hello world";
        let pos = find_escape_needed(input, 0);
        assert_eq!(pos, input.len());
    }

    #[test]
    fn test_find_escape_needed_quote() {
        let input = b"hello\"world";
        let pos = find_escape_needed(input, 0);
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_find_escape_needed_control() {
        let input = b"hello\nworld";
        let pos = find_escape_needed(input, 0);
        assert_eq!(pos, 5);
    }
}
