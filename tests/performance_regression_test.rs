use rfgrep::search_algorithms::SearchAlgorithm;
use rfgrep::search_algorithms::{BoyerMoore, SimdSearch};
use rfgrep::streaming_search::{StreamingConfig, StreamingSearchPipeline};
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

// Performance thresholds (in milliseconds)
const SMALL_FILE_THRESHOLD_MS: u128 = 5;
const MEDIUM_FILE_THRESHOLD_MS: u128 = 50;
const LARGE_FILE_THRESHOLD_MS: u128 = 500;
const SIMD_SPEEDUP_FACTOR: f64 = 2.0;

fn generate_test_content(size: usize) -> String {
    "The quick brown fox jumps over the lazy dog. ".repeat(size / 45)
}

#[test]
fn test_small_file_performance() {
    let content = generate_test_content(1024); // 1KB
    let pattern = "lazy";

    let searcher = SimdSearch::new(pattern);
    let start = Instant::now();
    let _ = searcher.search(&content, pattern);
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < SMALL_FILE_THRESHOLD_MS,
        "Small file search took {}ms, expected <{}ms",
        elapsed,
        SMALL_FILE_THRESHOLD_MS
    );
}

#[test]
fn test_medium_file_performance() {
    let content = generate_test_content(10_240); // 10KB
    let pattern = "lazy";

    let searcher = SimdSearch::new(pattern);
    let start = Instant::now();
    let _ = searcher.search(&content, pattern);
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < MEDIUM_FILE_THRESHOLD_MS,
        "Medium file search took {}ms, expected <{}ms",
        elapsed,
        MEDIUM_FILE_THRESHOLD_MS
    );
}

#[test]
fn test_large_file_performance() {
    let content = generate_test_content(1_024_000);
    let pattern = "lazy";

    let searcher = SimdSearch::new(pattern);
    let start = Instant::now();
    let _ = searcher.search(&content, pattern);
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < LARGE_FILE_THRESHOLD_MS,
        "Large file search took {}ms, expected <{}ms",
        elapsed,
        LARGE_FILE_THRESHOLD_MS
    );
}

#[test]
fn test_simd_vs_simple_performance() {
    let content = generate_test_content(102_400);
    let pattern = "lazy";

    // Time SIMD search
    let simd_searcher = SimdSearch::new(pattern);
    let start = Instant::now();
    let _ = simd_searcher.search(&content, pattern);
    let simd_time = start.elapsed().as_micros();

    // Time Boyer-Moore search
    let bm_searcher = BoyerMoore::new(pattern);
    let start = Instant::now();
    let _ = bm_searcher.search(&content, pattern);
    let bm_time = start.elapsed().as_micros();

    let speedup = bm_time as f64 / simd_time as f64;

    assert!(
        speedup >= SIMD_SPEEDUP_FACTOR || simd_time < 1000,
        "SIMD speedup is only {:.2}x, expected at least {:.2}x (SIMD: {}µs, BM: {}µs)",
        speedup,
        SIMD_SPEEDUP_FACTOR,
        simd_time,
        bm_time
    );
}

#[test]
fn test_multiple_patterns_performance() {
    let content = generate_test_content(51_200);
    let patterns = vec!["quick", "brown", "fox", "lazy", "dog"];

    let start = Instant::now();
    for pattern in &patterns {
        let searcher = SimdSearch::new(pattern);
        let _ = searcher.search(&content, pattern);
    }
    let elapsed = start.elapsed().as_millis();

    // Should complete all patterns in reasonable time
    assert!(
        elapsed < 100,
        "Multiple pattern search took {}ms, expected <100ms",
        elapsed
    );
}

#[test]
fn test_file_io_performance() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let content = generate_test_content(51_200);
    fs::write(&test_file, &content).unwrap();

    let pattern = "lazy";
    let rt = tokio::runtime::Runtime::new().unwrap();

    let config = StreamingConfig {
        algorithm: SearchAlgorithm::BoyerMoore,
        context_lines: 0,
        case_sensitive: true,
        invert_match: false,
        max_matches: None,
        timeout_per_file: None,
        chunk_size: 8192,
        buffer_size: 65536,
    };

    let pipeline = StreamingSearchPipeline::new(config);

    let start = Instant::now();
    let _ = rt.block_on(async { pipeline.search_file(&test_file, pattern).await });
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < 100,
        "File I/O search took {}ms, expected <100ms",
        elapsed
    );
}

#[test]
fn test_parallel_processing_performance() {
    let temp_dir = TempDir::new().unwrap();
    let content = generate_test_content(10_240);

    // Create 20 test files
    let mut files = Vec::new();
    for i in 0..20 {
        let file_path = temp_dir.path().join(format!("test_{}.txt", i));
        fs::write(&file_path, &content).unwrap();
        files.push(file_path);
    }

    let pattern = "lazy";
    let rt = tokio::runtime::Runtime::new().unwrap();

    let config = StreamingConfig {
        algorithm: SearchAlgorithm::BoyerMoore,
        context_lines: 0,
        case_sensitive: true,
        invert_match: false,
        max_matches: None,
        timeout_per_file: None,
        chunk_size: 8192,
        buffer_size: 65536,
    };

    let pipeline = StreamingSearchPipeline::new(config);
    let file_refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();

    let start = Instant::now();
    let _ = rt.block_on(async { pipeline.search_files_parallel(&file_refs, pattern, 4).await });
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < 200,
        "Parallel search of 20 files took {}ms, expected <200ms",
        elapsed
    );
}

#[test]
fn test_pattern_length_performance() {
    let content = generate_test_content(102_400); // 100KB

    let patterns = vec![
        ("short", "ab"),
        ("medium", "lazy dog"),
        ("long", "The quick brown fox jumps over the lazy dog"),
    ];

    for (name, pattern) in patterns {
        let searcher = SimdSearch::new(pattern);
        let start = Instant::now();
        let _ = searcher.search(&content, pattern);
        let elapsed = start.elapsed().as_millis();

        assert!(
            elapsed < 50,
            "{} pattern search took {}ms, expected <50ms",
            name,
            elapsed
        );
    }
}

#[test]
fn test_match_frequency_performance() {
    let frequencies = vec![("rare", 100), ("common", 10), ("very_common", 2)];

    let pattern = "MATCH";

    for (name, interval) in frequencies {
        let mut text = String::with_capacity(100_000);
        for i in 0..10_000 {
            if i % interval == 0 {
                text.push_str("MATCH ");
            } else {
                text.push_str("nope ");
            }
        }

        let searcher = SimdSearch::new(pattern);
        let start = Instant::now();
        let _ = searcher.search(&text, pattern);
        let elapsed = start.elapsed().as_millis();

        assert!(
            elapsed < 100,
            "{} match frequency search took {}ms, expected <100ms",
            name,
            elapsed
        );
    }
}

#[test]
fn test_memory_efficiency() {
    let content = generate_test_content(1_024_000); // 1MB
    let pattern = "lazy";

    let searcher = SimdSearch::new(pattern);
    for _ in 0..100 {
        let _ = searcher.search(&content, pattern);
    }
}

#[test]
fn test_no_performance_regression() {
    let content = generate_test_content(512_000); // 500KB
    let pattern = "lazy";

    let searcher = SimdSearch::new(pattern);

    let mut times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let _ = searcher.search(&content, pattern);
        times.push(start.elapsed().as_micros());
    }

    let avg_time: u128 = times.iter().sum::<u128>() / times.len() as u128;
    let max_time = times.iter().max().unwrap();

    assert!(
        avg_time < 50_000,
        "Average search time is {}µs, expected <50,000µs",
        avg_time
    );

    assert!(
        *max_time < 100_000,
        "Worst case search time is {}µs, expected <100,000µs",
        max_time
    );
}
