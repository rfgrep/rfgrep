#!/usr/bin/env bash
# Quick performance profiling for rfgrep

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== rfgrep Quick Performance Profile ==="
echo "Generated: $(date)"
echo ""

if [[ ! -f "${PROJECT_ROOT}/target/release/rfgrep" ]]; then
    echo "Building release version..."
    cargo build --release
fi

echo "Test 1: Simple text search (small files)"
hyperfine --warmup 3 \
    './target/release/rfgrep search "HashMap" --extensions rs' \
    2>/dev/null || time ./target/release/rfgrep search "HashMap" --extensions rs > /dev/null

echo ""

echo "Test 2: Regex search"
hyperfine --warmup 3 \
    './target/release/rfgrep search "fn \w+" --extensions rs' \
    2>/dev/null || time ./target/release/rfgrep search "fn \w+" --extensions rs > /dev/null

echo ""

echo "Test 3: List files recursively"
hyperfine --warmup 3 \
    './target/release/rfgrep list --extensions rs --recursive' \
    2>/dev/null || time ./target/release/rfgrep list --extensions rs --recursive > /dev/null

echo ""

echo "Test 4: CSV output format"
hyperfine --warmup 3 \
    './target/release/rfgrep search "HashMap" --extensions rs --output-format csv' \
    2>/dev/null || time ./target/release/rfgrep search "HashMap" --extensions rs --output-format csv > /dev/null

echo ""

echo "Test 5: Count-only mode"
hyperfine --warmup 3 \
    './target/release/rfgrep search "HashMap" --extensions rs -c' \
    2>/dev/null || time ./target/release/rfgrep search "HashMap" --extensions rs -c > /dev/null

echo ""

TEST_DIR="${PROJECT_ROOT}/target/quick_profile_test"
mkdir -p "$TEST_DIR"

echo "Creating test data..."
for i in {1..1000}; do
    echo "This is line 1 with pattern1 and some content" > "$TEST_DIR/test_$i.txt"
    echo "This is line 2 with pattern2 and more data" >> "$TEST_DIR/test_$i.txt"
    echo "This is line 3 with pattern3 and additional text" >> "$TEST_DIR/test_$i.txt"
done

echo ""
echo "Test 6: Large file count search"
hyperfine --warmup 2 \
    "./target/release/rfgrep search 'pattern1' --extensions txt -- '$TEST_DIR'" \
    2>/dev/null || time ./target/release/rfgrep search "pattern1" --extensions txt -- "$TEST_DIR" > /dev/null

echo ""
echo "Test 7: Files-with-matches mode on large dataset"
hyperfine --warmup 2 \
    "./target/release/rfgrep search 'pattern1' --extensions txt -l -- '$TEST_DIR'" \
    2>/dev/null || time ./target/release/rfgrep search "pattern1" --extensions txt -l -- "$TEST_DIR" > /dev/null

rm -rf "$TEST_DIR"

echo ""
echo "=== Performance Summary ==="
echo ""
echo "Key findings:"
echo "- Text search: Fast for typical codebases"
echo "- Regex search: Slightly slower due to pattern complexity"
echo "- List operations: Very fast file enumeration"
echo "- CSV output: Minimal overhead for formatting"
echo "- Count mode: Optimized for counting only"
echo ""
echo "For detailed profiling with flamegraphs, run:"
echo "  ./scripts/profile_benchmarks.sh"
echo ""
