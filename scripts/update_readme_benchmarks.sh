#!/bin/bash
# Update README.md with latest benchmark results

set -e

README_FILE="${1:-README.md}"
BENCHMARK_RESULTS="${2:-benchmark_results.md}"

if [ ! -f "$README_FILE" ]; then
    echo "Error: README file not found at $README_FILE"
    exit 1
fi

if [ ! -f "$BENCHMARK_RESULTS" ]; then
    echo "Error: Benchmark results file not found at $BENCHMARK_RESULTS"
    exit 1
fi

TEMP_FILE=$(mktemp)

START_MARKER="<!-- BENCHMARK_RESULTS_START -->"
END_MARKER="<!-- BENCHMARK_RESULTS_END -->"

if ! grep -q "$START_MARKER" "$README_FILE"; then
    echo "Error: Start marker not found in README. Please add '$START_MARKER' where you want benchmarks to appear."
    exit 1
fi

if ! grep -q "$END_MARKER" "$README_FILE"; then
    echo "Error: End marker not found in README. Please add '$END_MARKER' after the start marker."
    exit 1
fi

awk "/$START_MARKER/{exit} {print}" "$README_FILE" > "$TEMP_FILE"

echo "$START_MARKER" >> "$TEMP_FILE"

cat "$BENCHMARK_RESULTS" >> "$TEMP_FILE"

echo "$END_MARKER" >> "$TEMP_FILE"

awk "found{print} /$END_MARKER/{found=1}" "$README_FILE" >> "$TEMP_FILE"

mv "$TEMP_FILE" "$README_FILE"

echo "README updated successfully with latest benchmark results"
