//! Scalar fallback implementations for SIMD operations
//!
//! These implementations are used when SIMD is not available or for
//! remaining bytes after SIMD processing.

use super::ScanResult;

/// Scalar implementation of string scanning
#[inline]
pub fn scan_string_scalar(bytes: &[u8], start: usize) -> ScanResult {
    let mut pos = start;
    let mut has_escapes = false;

    while pos < bytes.len() {
        let byte = unsafe { *bytes.get_unchecked(pos) };
        match byte {
            b'"' => {
                return ScanResult {
                    position: pos,
                    has_escapes,
                };
            }
            b'\\' => {
                has_escapes = true;
                pos += 2; // Skip escape sequence
                continue;
            }
            _ => {
                pos += 1;
            }
        }
    }

    ScanResult {
        position: pos,
        has_escapes,
    }
}

/// Scalar implementation of whitespace skipping
#[inline]
pub fn skip_whitespace_scalar(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;

    while pos < bytes.len() {
        let byte = unsafe { *bytes.get_unchecked(pos) };
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => pos += 1,
            _ => break,
        }
    }

    pos
}

/// Scalar implementation of escape detection
#[inline]
pub fn find_escape_needed_scalar(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;

    while pos < bytes.len() {
        let byte = unsafe { *bytes.get_unchecked(pos) };

        // Need to escape: control characters, quote, backslash
        if byte < 0x20 || byte == b'"' || byte == b'\\' {
            return pos;
        }

        pos += 1;
    }

    pos
}
