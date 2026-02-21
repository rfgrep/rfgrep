#![allow(dead_code)]

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::x86_64::*;

/// Interface for SIMD search backends
pub trait SimdBackend: Send + Sync {
    /// Search for pattern in text, returning all start indices
    fn search(&self, text: &[u8]) -> Vec<usize>;
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// AVX2 implementation using 256-bit registers
pub struct Avx2Backend {
    pattern: Vec<u8>,
    first_byte: u8,
    last_byte: u8,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Avx2Backend {
    pub fn new(pattern: &[u8]) -> Self {
        Self {
            pattern: pattern.to_vec(),
            first_byte: pattern[0],
            last_byte: *pattern.last().unwrap(),
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl SimdBackend for Avx2Backend {
    fn search(&self, text: &[u8]) -> Vec<usize> {
        let mut matches = Vec::new();
        let pat_len = self.pattern.len();
        let text_len = text.len();

        if text_len < pat_len {
            return matches;
        }

        unsafe {
            // Broadcast first byte of pattern to 256-bit register
            let first = _mm256_set1_epi8(self.first_byte as i8);

            let mut i = 0;
            // Process 32 bytes at a time
            while i + 32 <= text_len {
                let chunk = _mm256_loadu_si256(text.as_ptr().add(i) as *const __m256i);

                // Compare chunk with first byte
                let eq_first = _mm256_cmpeq_epi8(chunk, first);

                // Create mask from comparison result
                let mask = _mm256_movemask_epi8(eq_first) as u32;

                if mask != 0 {
                    // Iterate over set bits
                    let mut temp_mask = mask;
                    while temp_mask != 0 {
                        let tz = temp_mask.trailing_zeros() as usize;
                        let potential_match_idx = i + tz;

                        // Check bounds and last byte before full compare
                        if potential_match_idx + pat_len <= text_len {
                            if text[potential_match_idx + pat_len - 1] == self.last_byte {
                                // Full comparison
                                if &text[potential_match_idx..potential_match_idx + pat_len]
                                    == self.pattern.as_slice()
                                {
                                    matches.push(potential_match_idx);
                                }
                            }
                        }

                        // Clear least significant bit
                        temp_mask &= temp_mask - 1;
                    }
                }

                i += 32;
            }

            // Fallback for remaining bytes
            while i <= text_len.saturating_sub(pat_len) {
                if text[i] == self.first_byte {
                    if &text[i..i + pat_len] == self.pattern.as_slice() {
                        matches.push(i);
                    }
                }
                i += 1;
            }
        }
        matches
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// SSE4.2 implementation using 128-bit registers
pub struct Sse42Backend {
    pattern: Vec<u8>,
    first_byte: u8,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Sse42Backend {
    pub fn new(pattern: &[u8]) -> Self {
        Self {
            pattern: pattern.to_vec(),
            first_byte: pattern[0],
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl SimdBackend for Sse42Backend {
    fn search(&self, text: &[u8]) -> Vec<usize> {
        let mut matches = Vec::new();
        let pat_len = self.pattern.len();
        let text_len = text.len();

        if text_len < pat_len {
            return matches;
        }

        unsafe {
            let first = _mm_set1_epi8(self.first_byte as i8);

            let mut i = 0;
            // Process 16 bytes at a time
            while i + 16 <= text_len {
                let chunk = _mm_loadu_si128(text.as_ptr().add(i) as *const __m128i);
                let eq = _mm_cmpeq_epi8(chunk, first);
                let mask = _mm_movemask_epi8(eq);

                if mask != 0 {
                    let mut temp_mask = mask as u32;
                    while temp_mask != 0 {
                        let tz = temp_mask.trailing_zeros() as usize;
                        let potential_match_idx = i + tz;

                        if potential_match_idx + pat_len <= text_len {
                            if &text[potential_match_idx..potential_match_idx + pat_len]
                                == self.pattern.as_slice()
                            {
                                matches.push(potential_match_idx);
                            }
                        }
                        temp_mask &= temp_mask - 1;
                    }
                }
                i += 16;
            }

            // Fallback
            while i <= text_len.saturating_sub(pat_len) {
                if text[i] == self.first_byte {
                    if &text[i..i + pat_len] == self.pattern.as_slice() {
                        matches.push(i);
                    }
                }
                i += 1;
            }
        }
        matches
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx512f"
))]
/// AVX-512 implementation using 512-bit registers
pub struct Avx512Backend {
    pattern: Vec<u8>,
    first_byte: u8,
    last_byte: u8,
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx512f"
))]
impl Avx512Backend {
    pub fn new(pattern: &[u8]) -> Self {
        Self {
            pattern: pattern.to_vec(),
            first_byte: pattern[0],
            last_byte: *pattern.last().unwrap(),
        }
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx512f"
))]
impl SimdBackend for Avx512Backend {
    fn search(&self, text: &[u8]) -> Vec<usize> {
        let mut matches = Vec::new();
        let pat_len = self.pattern.len();
        let text_len = text.len();

        if text_len < pat_len {
            return matches;
        }

        unsafe {
            let first = std::arch::x86_64::_mm512_set1_epi8(self.first_byte as i8);
            let mut i = 0;
            while i + 64 <= text_len {
                let chunk = std::arch::x86_64::_mm512_loadu_si512(
                    text.as_ptr().add(i) as *const std::arch::x86_64::__m512i
                );
                let eq_first = std::arch::x86_64::_mm512_cmpeq_epi8_mask(chunk, first);
                if eq_first != 0 {
                    for bit in 0..64 {
                        if (eq_first & (1u64 << bit)) != 0 {
                            let potential_match_idx = i + bit;
                            if potential_match_idx + pat_len <= text_len {
                                if text[potential_match_idx + pat_len - 1] == self.last_byte {
                                    if &text[potential_match_idx..potential_match_idx + pat_len]
                                        == self.pattern.as_slice()
                                    {
                                        matches.push(potential_match_idx);
                                    }
                                }
                            }
                        }
                    }
                }
                i += 64;
            }
            while i <= text_len.saturating_sub(pat_len) {
                if text[i] == self.first_byte {
                    if &text[i..i + pat_len] == self.pattern.as_slice() {
                        matches.push(i);
                    }
                }
                i += 1;
            }
        }
        matches
    }
}

/// Fallback backend using standard library
pub struct FallbackBackend {
    pattern: Vec<u8>,
}

impl FallbackBackend {
    pub fn new(pattern: &[u8]) -> Self {
        Self {
            pattern: pattern.to_vec(),
        }
    }
}

impl SimdBackend for FallbackBackend {
    fn search(&self, text: &[u8]) -> Vec<usize> {
        let mut matches = Vec::new();
        if self.pattern.is_empty() {
            return matches;
        }

        let pat_len = self.pattern.len();
        let text_len = text.len();
        if text_len < pat_len {
            return matches;
        }
        for i in 0..=text_len - pat_len {
            if &text[i..i + pat_len] == self.pattern.as_slice() {
                matches.push(i);
            }
        }
        matches
    }
}

/// Main Engine that selects best backend
pub struct SimdSearchEngine {
    backend: Box<dyn SimdBackend>,
}

impl SimdSearchEngine {
    pub fn new(pattern: &str) -> Self {
        let bytes = pattern.as_bytes();
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            #[cfg(target_feature = "avx512f")]
            {
                if is_x86_feature_detected!("avx512f") && !bytes.is_empty() {
                    return Self {
                        backend: Box::new(Avx512Backend::new(bytes)),
                    };
                }
            }
            if is_x86_feature_detected!("avx2") && !bytes.is_empty() {
                return Self {
                    backend: Box::new(Avx2Backend::new(bytes)),
                };
            } else if is_x86_feature_detected!("sse4.2") && !bytes.is_empty() {
                return Self {
                    backend: Box::new(Sse42Backend::new(bytes)),
                };
            }
        }
        Self {
            backend: Box::new(FallbackBackend::new(bytes)),
        }
    }

    pub fn search(&self, text: &str) -> Vec<usize> {
        self.backend.search(text.as_bytes())
    }
}
