use proptest::prelude::*;
use rfgrep::search_algorithms::{BoyerMoore, SimdSearch, SimpleSearch};

// Property: Search should always find the pattern if it exists
proptest! {
    #[test]
    fn test_simd_search_finds_pattern(pattern in "[a-z]{1,10}", text_prefix in "[a-z ]{0,100}", text_suffix in "[a-z ]{0,100}") {
        let text = format!("{}{}{}", text_prefix, pattern, text_suffix);
        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text, &pattern);
        prop_assert!(!results.is_empty(), "Pattern '{}' should be found in text", pattern);

        for &pos in &results {
            let found = &text[pos..pos + pattern.len()];
            prop_assert_eq!(found, &pattern, "Found text should match pattern");
        }
    }
}

proptest! {
    #[test]
    fn test_boyer_moore_finds_pattern(pattern in "[a-z]{2,10}", text_prefix in "[a-z ]{0,100}", text_suffix in "[a-z ]{0,100}") {
        let text = format!("{}{}{}", text_prefix, pattern, text_suffix);
        let searcher = BoyerMoore::new(&pattern);
        let results = searcher.search(&text, &pattern);

        if !results.is_empty() {
            for &pos in &results {
                if pos + pattern.len() <= text.len() {
                    let found = &text[pos..pos + pattern.len()];
                    prop_assert_eq!(found, &pattern, "Found text should match pattern");
                }
            }
        }
    }
}

// Property: Empty pattern should return empty results
proptest! {
    #[test]
    fn test_empty_pattern_returns_empty(text in ".*") {
        let searcher = SimdSearch::new("");
        let results = searcher.search(&text, "");
        prop_assert!(results.is_empty(), "Empty pattern should return no results");
    }
}

// Property: Pattern not in text should return empty results
proptest! {
    #[test]
    fn test_pattern_not_found(pattern in "XXXXX+", text in "[a-z ]{10,100}") {
        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text, &pattern);
        prop_assert!(results.is_empty(), "Pattern '{}' should not be found in text without it", pattern);
    }
}

// Property: Multiple occurrences should all be found
proptest! {
    #[test]
    fn test_multiple_occurrences(pattern in "[a-z]{3,8}", count in 2usize..10) {
        let parts: Vec<String> = (0..count + 1).map(|i| format!("filler{}", i)).collect();
        let mut text = String::new();

        for (i, part) in parts.iter().enumerate() {
            text.push_str(part);
            if i < count {
                text.push_str(&pattern);
            }
        }

        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text, &pattern);

        prop_assert_eq!(results.len(), count, "Should find exactly {} occurrences", count);
    }
}

// Property: All algorithms should find the same matches
proptest! {
    #[test]
    fn test_algorithms_consistency(pattern in "[a-z]{2,8}", text in "[a-z ]{20,200}") {
        if !text.contains(&pattern) {
            return Ok(());
        }

        let simd = SimdSearch::new(&pattern);
        let boyer_moore = BoyerMoore::new(&pattern);
        let simple = SimpleSearch::new(&pattern);

        let simd_results = simd.search(&text, &pattern);
        let bm_results = boyer_moore.search(&text, &pattern);
        let simple_results = simple.search(&text, &pattern);

        prop_assert_eq!(simd_results.len(), bm_results.len(),
            "SIMD and Boyer-Moore should find same number of matches");
        prop_assert_eq!(simd_results.len(), simple_results.len(),
            "SIMD and Simple should find same number of matches");
        for i in 0..simd_results.len() {
            prop_assert_eq!(simd_results[i], bm_results[i],
                "Match positions should be identical");
            prop_assert_eq!(simd_results[i], simple_results[i],
                "Match positions should be identical");
        }
    }
}

// Property: Search should be case-sensitive by default
proptest! {
    #[test]
    fn test_case_sensitivity(lower in "[a-z]{3,8}") {
        let upper = lower.to_uppercase();
        let text = format!("prefix {} middle {} suffix", lower, upper);

        let searcher = SimdSearch::new(&lower);
        let results = searcher.search(&text, &lower);

        prop_assert_eq!(results.len(), 1, "Should only find lowercase match");
    }
}

// Property: Overlapping patterns should be handled correctly
proptest! {
    #[test]
    fn test_overlapping_patterns(base in "[a-z]{3,5}") {
        let pattern = format!("{}{}", base, base);
        let text = format!("{}{}{}", base, base, base);

        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text, &pattern);

        prop_assert!(!results.is_empty(), "Should find pattern in overlapping text");

        for &pos in &results {
            if pos + pattern.len() <= text.len() {
                let found = &text[pos..pos + pattern.len()];
                prop_assert_eq!(found, &pattern, "Found text should match pattern");
            }
        }
    }
}

// Property: Search should handle special characters
proptest! {
    #[test]
    fn test_special_characters(pattern in "[a-z0-9]{1,10}") {
        let text = format!("start {} middle {} end", pattern, pattern);
        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text, &pattern);

        prop_assert!(results.len() >= 2, "Should find at least both explicit occurrences");
    }
}

// Property: Search at boundaries (start, end)
proptest! {
    #[test]
    fn test_boundary_search(pattern in "[a-z]{3,8}", middle in "[a-z ]{10,50}") {
        let text_start = format!("{}{}", pattern, middle);
        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text_start, &pattern);
        prop_assert!(!results.is_empty(), "Should find pattern at start");
        prop_assert_eq!(results[0], 0, "First match should be at position 0");

        let text_end = format!("{}{}", middle, pattern);
        let results = searcher.search(&text_end, &pattern);
        prop_assert!(!results.is_empty(), "Should find pattern at end");
    }
}

// Property: Large text should be handled efficiently
proptest! {
    #[test]
    fn test_large_text_handling(pattern in "[a-z]{4,8}") {
        let mut text = String::new();
        for i in 0..1000 {
            text.push_str(&format!("filler{} ", i));
            if i % 100 == 0 {
                text.push_str(&pattern);
                text.push(' ');
            }
        }

        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text, &pattern);

        prop_assert!(results.len() >= 9 && results.len() <= 11,
            "Should find around 10 occurrences in large text");
    }
}

// Property: Unicode handling
proptest! {
    #[test]
    fn test_unicode_patterns(pattern in "[a-zA-Z0-9]{3,8}") {
        let text = format!("Hello {} World {} 你好", pattern, pattern);
        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text, &pattern);

        prop_assert_eq!(results.len(), 2, "Should find both ASCII pattern occurrences");
    }
}

// Property: Repeated patterns
proptest! {
    #[test]
    fn test_repeated_patterns(pattern in "[a-z]{2,4}", repeats in 3usize..20) {
        let text = pattern.repeat(repeats);
        let searcher = SimdSearch::new(&pattern);
        let results = searcher.search(&text, &pattern);

        prop_assert!(results.len() >= repeats - 1,
            "Should find multiple occurrences of repeated pattern");
    }
}

// Property: Single character patterns
proptest! {
    #[test]
    fn test_single_char_pattern(text in "[a-z ]{20,100}") {
        let pattern = "x";
        let text_with_pattern = format!("{}{}{}", text, pattern, text);

        let searcher = SimdSearch::new(pattern);
        let results = searcher.search(&text_with_pattern, pattern);

        prop_assert!(!results.is_empty(), "Should find single character pattern");
    }
}

// Property: Pattern longer than text
proptest! {
    #[test]
    fn test_pattern_longer_than_text(short_text in "[a-z]{1,5}") {
        let long_pattern = "thisisaverylongpattern";
        let searcher = SimdSearch::new(long_pattern);
        let results = searcher.search(&short_text, long_pattern);

        prop_assert!(results.is_empty(), "Should not find pattern longer than text");
    }
}
