//! x86_64 SIMD implementations using SSE2 and AVX2
//!
//! SSE2: 128-bit vectors (16 bytes at a time)
//! AVX2: 256-bit vectors (32 bytes at a time)

use super::ScanResult;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SSE2 implementation of string scanning (16 bytes at a time)
#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn scan_string_sse2(bytes: &[u8], start: usize) -> ScanResult {
    let mut pos = start;
    let mut has_escapes = false;
    let len = bytes.len();

    // Create comparison vectors
    let quote_vec = _mm_set1_epi8(b'"' as i8);
    let backslash_vec = _mm_set1_epi8(b'\\' as i8);

    // Process 16 bytes at a time
    while pos + 16 <= len {
        let chunk = _mm_loadu_si128(bytes.as_ptr().add(pos) as *const __m128i);

        // Compare with quote and backslash
        let quote_cmp = _mm_cmpeq_epi8(chunk, quote_vec);
        let backslash_cmp = _mm_cmpeq_epi8(chunk, backslash_vec);

        // Combine results
        let combined = _mm_or_si128(quote_cmp, backslash_cmp);
        let mask = _mm_movemask_epi8(combined);

        if mask != 0 {
            // Found a special character
            let offset = mask.trailing_zeros() as usize;
            let found_pos = pos + offset;
            let found_byte = *bytes.get_unchecked(found_pos);

            if found_byte == b'\\' {
                has_escapes = true;
                // Continue scanning after the escape sequence
                pos = found_pos + 2;
                continue;
            } else {
                // Found quote
                return ScanResult {
                    position: found_pos,
                    has_escapes,
                };
            }
        }

        pos += 16;
    }

    // Process remaining bytes with scalar fallback
    while pos < len {
        let byte = *bytes.get_unchecked(pos);
        match byte {
            b'"' => {
                return ScanResult {
                    position: pos,
                    has_escapes,
                };
            }
            b'\\' => {
                has_escapes = true;
                pos += 2;
                continue;
            }
            _ => pos += 1,
        }
    }

    ScanResult {
        position: pos,
        has_escapes,
    }
}

/// AVX2 implementation of string scanning (32 bytes at a time)
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn scan_string_avx2(bytes: &[u8], start: usize) -> ScanResult {
    let mut pos = start;
    let mut has_escapes = false;
    let len = bytes.len();

    // Create comparison vectors
    let quote_vec = _mm256_set1_epi8(b'"' as i8);
    let backslash_vec = _mm256_set1_epi8(b'\\' as i8);

    // Process 32 bytes at a time
    while pos + 32 <= len {
        let chunk = _mm256_loadu_si256(bytes.as_ptr().add(pos) as *const __m256i);

        // Compare with quote and backslash
        let quote_cmp = _mm256_cmpeq_epi8(chunk, quote_vec);
        let backslash_cmp = _mm256_cmpeq_epi8(chunk, backslash_vec);

        // Combine results
        let combined = _mm256_or_si256(quote_cmp, backslash_cmp);
        let mask = _mm256_movemask_epi8(combined) as u32;

        if mask != 0 {
            // Found a special character
            let offset = mask.trailing_zeros() as usize;
            let found_pos = pos + offset;
            let found_byte = *bytes.get_unchecked(found_pos);

            if found_byte == b'\\' {
                has_escapes = true;
                // Continue scanning after the escape sequence
                pos = found_pos + 2;
                continue;
            } else {
                // Found quote
                return ScanResult {
                    position: found_pos,
                    has_escapes,
                };
            }
        }

        pos += 32;
    }

    // Process remaining bytes with SSE2 or scalar fallback
    if pos + 16 <= len {
        return scan_string_sse2(bytes, pos);
    }

    // Scalar fallback for last few bytes
    while pos < len {
        let byte = *bytes.get_unchecked(pos);
        match byte {
            b'"' => {
                return ScanResult {
                    position: pos,
                    has_escapes,
                };
            }
            b'\\' => {
                has_escapes = true;
                pos += 2;
                continue;
            }
            _ => pos += 1,
        }
    }

    ScanResult {
        position: pos,
        has_escapes,
    }
}

/// SSE2 implementation of whitespace skipping
#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn skip_whitespace_sse2(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;
    let len = bytes.len();

    // Create comparison vectors for each whitespace character
    let space_vec = _mm_set1_epi8(b' ' as i8);
    let tab_vec = _mm_set1_epi8(b'\t' as i8);
    let newline_vec = _mm_set1_epi8(b'\n' as i8);
    let cr_vec = _mm_set1_epi8(b'\r' as i8);

    // Process 16 bytes at a time
    while pos + 16 <= len {
        let chunk = _mm_loadu_si128(bytes.as_ptr().add(pos) as *const __m128i);

        // Check if all bytes are whitespace
        let space_cmp = _mm_cmpeq_epi8(chunk, space_vec);
        let tab_cmp = _mm_cmpeq_epi8(chunk, tab_vec);
        let newline_cmp = _mm_cmpeq_epi8(chunk, newline_vec);
        let cr_cmp = _mm_cmpeq_epi8(chunk, cr_vec);

        // Combine all whitespace matches
        let ws1 = _mm_or_si128(space_cmp, tab_cmp);
        let ws2 = _mm_or_si128(newline_cmp, cr_cmp);
        let all_ws = _mm_or_si128(ws1, ws2);

        let mask = _mm_movemask_epi8(all_ws);

        if mask != 0xFFFF {
            // Not all bytes are whitespace
            // Find first non-whitespace
            let inverted = !mask as u16;
            let offset = inverted.trailing_zeros() as usize;
            return pos + offset;
        }

        pos += 16;
    }

    // Scalar fallback
    while pos < len {
        let byte = *bytes.get_unchecked(pos);
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => pos += 1,
            _ => break,
        }
    }

    pos
}

/// AVX2 implementation of whitespace skipping
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn skip_whitespace_avx2(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;
    let len = bytes.len();

    // Create comparison vectors
    let space_vec = _mm256_set1_epi8(b' ' as i8);
    let tab_vec = _mm256_set1_epi8(b'\t' as i8);
    let newline_vec = _mm256_set1_epi8(b'\n' as i8);
    let cr_vec = _mm256_set1_epi8(b'\r' as i8);

    // Process 32 bytes at a time
    while pos + 32 <= len {
        let chunk = _mm256_loadu_si256(bytes.as_ptr().add(pos) as *const __m256i);

        // Check if all bytes are whitespace
        let space_cmp = _mm256_cmpeq_epi8(chunk, space_vec);
        let tab_cmp = _mm256_cmpeq_epi8(chunk, tab_vec);
        let newline_cmp = _mm256_cmpeq_epi8(chunk, newline_vec);
        let cr_cmp = _mm256_cmpeq_epi8(chunk, cr_vec);

        let ws1 = _mm256_or_si256(space_cmp, tab_cmp);
        let ws2 = _mm256_or_si256(newline_cmp, cr_cmp);
        let all_ws = _mm256_or_si256(ws1, ws2);

        let mask = _mm256_movemask_epi8(all_ws) as u32;

        if mask != 0xFFFFFFFF {
            // Not all bytes are whitespace
            let inverted = !mask;
            let offset = inverted.trailing_zeros() as usize;
            return pos + offset;
        }

        pos += 32;
    }

    // SSE2 fallback for remaining bytes
    if pos + 16 <= len {
        return skip_whitespace_sse2(bytes, pos);
    }

    // Scalar fallback
    while pos < len {
        let byte = *bytes.get_unchecked(pos);
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => pos += 1,
            _ => break,
        }
    }

    pos
}

/// SSE2 implementation of escape detection
#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn find_escape_needed_sse2(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;
    let len = bytes.len();

    let control_max = _mm_set1_epi8(0x1F as i8);
    let quote_vec = _mm_set1_epi8(b'"' as i8);
    let backslash_vec = _mm_set1_epi8(b'\\' as i8);

    while pos + 16 <= len {
        let chunk = _mm_loadu_si128(bytes.as_ptr().add(pos) as *const __m128i);

        // Check for control characters (< 0x20)
        let control_cmp = _mm_cmplt_epi8(chunk, control_max);
        // Check for quote and backslash
        let quote_cmp = _mm_cmpeq_epi8(chunk, quote_vec);
        let backslash_cmp = _mm_cmpeq_epi8(chunk, backslash_vec);

        // Combine all checks
        let special1 = _mm_or_si128(control_cmp, quote_cmp);
        let special = _mm_or_si128(special1, backslash_cmp);

        let mask = _mm_movemask_epi8(special);

        if mask != 0 {
            let offset = mask.trailing_zeros() as usize;
            return pos + offset;
        }

        pos += 16;
    }

    // Scalar fallback
    while pos < len {
        let byte = *bytes.get_unchecked(pos);
        if byte < 0x20 || byte == b'"' || byte == b'\\' {
            return pos;
        }
        pos += 1;
    }

    pos
}

/// AVX2 implementation of escape detection
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn find_escape_needed_avx2(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;
    let len = bytes.len();

    let control_max = _mm256_set1_epi8(0x1F as i8);
    let quote_vec = _mm256_set1_epi8(b'"' as i8);
    let backslash_vec = _mm256_set1_epi8(b'\\' as i8);

    while pos + 32 <= len {
        let chunk = _mm256_loadu_si256(bytes.as_ptr().add(pos) as *const __m256i);

        // Check for control characters (< 0x20)
        let control_cmp = _mm256_cmpgt_epi8(control_max, chunk);
        // Check for quote and backslash
        let quote_cmp = _mm256_cmpeq_epi8(chunk, quote_vec);
        let backslash_cmp = _mm256_cmpeq_epi8(chunk, backslash_vec);

        // Combine all checks
        let special1 = _mm256_or_si256(control_cmp, quote_cmp);
        let special = _mm256_or_si256(special1, backslash_cmp);

        let mask = _mm256_movemask_epi8(special) as u32;

        if mask != 0 {
            let offset = mask.trailing_zeros() as usize;
            return pos + offset;
        }

        pos += 32;
    }

    // SSE2 fallback
    if pos + 16 <= len {
        return find_escape_needed_sse2(bytes, pos);
    }

    // Scalar fallback
    while pos < len {
        let byte = *bytes.get_unchecked(pos);
        if byte < 0x20 || byte == b'"' || byte == b'\\' {
            return pos;
        }
        pos += 1;
    }

    pos
}
