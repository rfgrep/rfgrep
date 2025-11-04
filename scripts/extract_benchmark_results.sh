#!/bin/bash
# extract benchmark results from Criterion output and format for README

set -e

CRITERION_DIR="${1:-target/criterion}"
OUTPUT_FILE="${2:-benchmark_results.md}"

if [ ! -d "$CRITERION_DIR" ]; then
    echo "Error: Criterion directory not found at $CRITERION_DIR"
    exit 1
fi

# start generating the output
cat > "$OUTPUT_FILE" << 'EOF'
## Performance Benchmarks

Latest benchmark results (automatically updated):

EOF

# add system info
cat >> "$OUTPUT_FILE" << EOF
**System:** $(uname -s) $(uname -m)
**Date:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Commit:** ${GITHUB_SHA:-$(git rev-parse HEAD 2>/dev/null || echo "local")}

EOF

# function to extract time from estimates.json
extract_benchmark_time() {
    local bench_dir="$1"
    local estimates_file="$bench_dir/new/estimates.json"

    if [ -f "$estimates_file" ]; then
        local mean_ns=$(jq -r '.mean.point_estimate' "$estimates_file" 2>/dev/null || echo "0")

        if [ "$mean_ns" != "0" ] && [ "$mean_ns" != "null" ]; then
            if (( $(echo "$mean_ns < 1000" | bc -l) )); then
                echo "${mean_ns} ns"
            elif (( $(echo "$mean_ns < 1000000" | bc -l) )); then
                echo "$(echo "scale=2; $mean_ns / 1000" | bc) µs"
            elif (( $(echo "$mean_ns < 1000000000" | bc -l) )); then
                echo "$(echo "scale=2; $mean_ns / 1000000" | bc) ms"
            else
                echo "$(echo "scale=2; $mean_ns / 1000000000" | bc) s"
            fi
        else
            echo "N/A"
        fi
    else
        echo "N/A"
    fi
}

# process algorithm benchmarks
if [ -d "$CRITERION_DIR/algorithm" ]; then
    cat >> "$OUTPUT_FILE" << 'EOF'
### Search Algorithms

| Algorithm | Mean Time |
|-----------|-----------|
EOF

    for algo in boyer_moore regex simple; do
        if [ -d "$CRITERION_DIR/algorithm/$algo" ]; then
            time=$(extract_benchmark_time "$CRITERION_DIR/algorithm/$algo")
            echo "| ${algo//_/ } | $time |" >> "$OUTPUT_FILE"
        fi
    done
    echo "" >> "$OUTPUT_FILE"
fi

# process file size benchmarks
if [ -d "$CRITERION_DIR/file_size" ]; then
    cat >> "$OUTPUT_FILE" << 'EOF'
### File Size Performance

| File Size | Mean Time |
|-----------|-----------|
EOF

    for size in small medium large; do
        if [ -d "$CRITERION_DIR/file_size/$size" ]; then
            time=$(extract_benchmark_time "$CRITERION_DIR/file_size/$size")
            echo "| $size | $time |" >> "$OUTPUT_FILE"
        fi
    done
    echo "" >> "$OUTPUT_FILE"
fi

# process pattern benchmarks
if [ -d "$CRITERION_DIR/pattern" ]; then
    cat >> "$OUTPUT_FILE" << 'EOF'
### Pattern Complexity

| Pattern Type | Mean Time |
|--------------|-----------|
EOF

    for pattern in simple regex complex_regex; do
        if [ -d "$CRITERION_DIR/pattern/$pattern" ]; then
            time=$(extract_benchmark_time "$CRITERION_DIR/pattern/$pattern")
            echo "| ${pattern//_/ } | $time |" >> "$OUTPUT_FILE"
        fi
    done
    echo "" >> "$OUTPUT_FILE"
fi

# process memory usage benchmark
if [ -d "$CRITERION_DIR/memory_usage" ]; then
    cat >> "$OUTPUT_FILE" << 'EOF'
### Memory Usage

| Benchmark | Mean Time |
|-----------|-----------|
EOF

    time=$(extract_benchmark_time "$CRITERION_DIR/memory_usage")
    echo "| memory usage | $time |" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
fi

# add note about viewing detailed results
cat >> "$OUTPUT_FILE" << 'EOF'

> **note:** These benchmarks are run automatically on every push to the main branch,... for detailed results and visualizations, see the [benchmark reports](https://github.com/kh3rld/rfgrep/actions/workflows/bench.yml).
EOF

echo "Benchmark results extracted to $OUTPUT_FILE"
