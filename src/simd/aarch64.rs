//! ARM NEON implementations for AArch64
//!
//! NEON: 128-bit vectors (16 bytes at a time)

use super::ScanResult;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// NEON implementation of string scanning (16 bytes at a time)
#[target_feature(enable = "neon")]
#[inline]
pub unsafe fn scan_string_neon(bytes: &[u8], start: usize) -> ScanResult {
    let mut pos = start;
    let mut has_escapes = false;
    let len = bytes.len();

    // Create comparison vectors
    let quote_vec = vdupq_n_u8(b'"');
    let backslash_vec = vdupq_n_u8(b'\\');

    // Process 16 bytes at a time
    while pos + 16 <= len {
        let chunk = vld1q_u8(bytes.as_ptr().add(pos));

        // Compare with quote and backslash
        let quote_cmp = vceqq_u8(chunk, quote_vec);
        let backslash_cmp = vceqq_u8(chunk, backslash_vec);

        // Combine results
        let combined = vorrq_u8(quote_cmp, backslash_cmp);

        // Convert to scalar mask
        // NEON doesn't have a direct movemask, so we need to extract bits
        let mask = neon_movemask(combined);

        if mask != 0 {
            let offset = mask.trailing_zeros() as usize;
            let found_pos = pos + offset;
            let found_byte = *bytes.get_unchecked(found_pos);

            if found_byte == b'\\' {
                has_escapes = true;
                pos = found_pos + 2;
                continue;
            } else {
                return ScanResult {
                    position: found_pos,
                    has_escapes,
                };
            }
        }

        pos += 16;
    }

    // Scalar fallback
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

/// NEON implementation of whitespace skipping
#[target_feature(enable = "neon")]
#[inline]
pub unsafe fn skip_whitespace_neon(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;
    let len = bytes.len();

    // Create comparison vectors
    let space_vec = vdupq_n_u8(b' ');
    let tab_vec = vdupq_n_u8(b'\t');
    let newline_vec = vdupq_n_u8(b'\n');
    let cr_vec = vdupq_n_u8(b'\r');

    while pos + 16 <= len {
        let chunk = vld1q_u8(bytes.as_ptr().add(pos));

        // Compare with all whitespace characters
        let space_cmp = vceqq_u8(chunk, space_vec);
        let tab_cmp = vceqq_u8(chunk, tab_vec);
        let newline_cmp = vceqq_u8(chunk, newline_vec);
        let cr_cmp = vceqq_u8(chunk, cr_vec);

        // Combine all matches
        let ws1 = vorrq_u8(space_cmp, tab_cmp);
        let ws2 = vorrq_u8(newline_cmp, cr_cmp);
        let all_ws = vorrq_u8(ws1, ws2);

        let mask = neon_movemask(all_ws);

        if mask != 0xFFFF {
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

/// NEON implementation of escape detection
#[target_feature(enable = "neon")]
#[inline]
pub unsafe fn find_escape_needed_neon(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;
    let len = bytes.len();

    let control_max = vdupq_n_u8(0x1F);
    let quote_vec = vdupq_n_u8(b'"');
    let backslash_vec = vdupq_n_u8(b'\\');

    while pos + 16 <= len {
        let chunk = vld1q_u8(bytes.as_ptr().add(pos));

        // Check for control characters (< 0x20)
        let control_cmp = vcltq_u8(chunk, control_max);
        // Check for quote and backslash
        let quote_cmp = vceqq_u8(chunk, quote_vec);
        let backslash_cmp = vceqq_u8(chunk, backslash_vec);

        // Combine all checks
        let special1 = vorrq_u8(control_cmp, quote_cmp);
        let special = vorrq_u8(special1, backslash_cmp);

        let mask = neon_movemask(special);

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

/// Helper function to emulate x86's _mm_movemask_epi8 on NEON
/// Extracts the high bit of each byte and packs into a u16
#[inline]
unsafe fn neon_movemask(input: uint8x16_t) -> u16 {
    // Extract high bits by shifting
    // This is a simplified version - production code would be more optimized
    let mut mask = 0u16;
    let bytes: [u8; 16] = std::mem::transmute(input);

    for (i, &byte) in bytes.iter().enumerate() {
        if byte != 0 {
            mask |= 1 << i;
        }
    }

    mask
}
