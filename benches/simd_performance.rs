use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rfgrep::search_algorithms::{BoyerMoore, SimdSearch};
use std::fs;
use std::hint::black_box;
use tempfile::TempDir;

fn benchmark_simd_vs_boyermoore(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD vs Boyer-Moore");

    let sizes = vec![
        ("1KB", 1_024),
        ("10KB", 10_240),
        ("100KB", 102_400),
        ("1MB", 1_048_576),
    ];

    for (name, size) in sizes {
        let text = "The quick brown fox jumps over the lazy dog. ".repeat(size / 45);
        let pattern = "lazy";

        group.bench_with_input(
            BenchmarkId::new("SIMD", name),
            &(&text, pattern),
            |b, (text, pattern)| {
                let searcher = SimdSearch::new(pattern);
                b.iter(|| {
                    let results = searcher.search(text, pattern);
                    black_box(results);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Boyer-Moore", name),
            &(&text, pattern),
            |b, (text, pattern)| {
                let searcher = BoyerMoore::new(pattern);
                b.iter(|| {
                    let results = searcher.search(text, pattern);
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_pattern_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("Pattern Length Impact");

    let text = "Lorem ipsum dolor sit amet consectetur adipiscing elit ".repeat(1000);
    let patterns = vec![
        ("short-2", "it"),
        ("short-4", "amet"),
        ("medium-8", "adipiscing"),
        ("long-16", "consectetur elit"),
    ];

    for (name, pattern) in patterns {
        group.bench_with_input(
            BenchmarkId::new("SIMD", name),
            &(&text, pattern),
            |b, (text, pattern)| {
                let searcher = SimdSearch::new(pattern);
                b.iter(|| {
                    let results = searcher.search(text, pattern);
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group(":)");

    let log_content = r#"[2025-11-15 10:23:45] INFO: Application started
[2025-11-15 10:23:46] ERROR: Connection timeout
[2025-11-15 10:23:47] WARN: Retry attempt 1
[2025-11-15 10:23:48] ERROR: Connection timeout
[2025-11-15 10:23:49] INFO: Connection established
"#
    .repeat(1000);

    group.bench_function("log-ERROR-search", |b| {
        let searcher = SimdSearch::new("ERROR");
        b.iter(|| {
            let results = searcher.search(&log_content, "ERROR");
            black_box(results);
        });
    });

    // Simulate code searching
    let code_content = r#"
fn example_function() {
    let result = HashMap::new();
    println!("HashMap created");
}
"#
    .repeat(500);

    group.bench_function("code-HashMap-search", |b| {
        let searcher = SimdSearch::new("HashMap");
        b.iter(|| {
            let results = searcher.search(&code_content, "HashMap");
            black_box(results);
        });
    });

    group.finish();
}

fn benchmark_match_frequency(c: &mut Criterion) {
    let mut group = c.benchmark_group("Match Frequency");

    let text_size = 100_000;

    // Rare matches (1%)
    let mut rare_text = String::with_capacity(text_size);
    for i in 0..text_size / 100 {
        rare_text.push_str(if i % 100 == 0 { "MATCH " } else { "nope " });
    }

    // Common matches (10%)
    let mut common_text = String::with_capacity(text_size);
    for i in 0..text_size / 100 {
        common_text.push_str(if i % 10 == 0 { "MATCH " } else { "nope " });
    }

    // Very common matches (50%)
    let mut very_common_text = String::with_capacity(text_size);
    for i in 0..text_size / 100 {
        very_common_text.push_str(if i % 2 == 0 { "MATCH " } else { "nope " });
    }

    let pattern = "MATCH";

    group.bench_function("rare-1%", |b| {
        let searcher = SimdSearch::new(pattern);
        b.iter(|| {
            let results = searcher.search(&rare_text, pattern);
            black_box(results);
        });
    });

    group.bench_function("common-10%", |b| {
        let searcher = SimdSearch::new(pattern);
        b.iter(|| {
            let results = searcher.search(&common_text, pattern);
            black_box(results);
        });
    });

    group.bench_function("very-common-50%", |b| {
        let searcher = SimdSearch::new(pattern);
        b.iter(|| {
            let results = searcher.search(&very_common_text, pattern);
            black_box(results);
        });
    });

    group.finish();
}

fn benchmark_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("File Operations");

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create test file
    let content =
        "This is a test file with multiple occurrences of the pattern word. ".repeat(10000);
    fs::write(&test_file, &content).unwrap();

    group.bench_function("read-and-search", |b| {
        let searcher = SimdSearch::new("pattern");
        b.iter(|| {
            let text = fs::read_to_string(&test_file).unwrap();
            let results = searcher.search(&text, "pattern");
            black_box(results);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_simd_vs_boyermoore,
    benchmark_pattern_lengths,
    benchmark_real_world_scenarios,
    benchmark_match_frequency,
    benchmark_file_operations
);
criterion_main!(benches);
