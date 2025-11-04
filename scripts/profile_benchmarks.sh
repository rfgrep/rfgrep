#!/usr/bin/env bash
# benchmark profiling script for rfgrep,..
# this script runs benchmarks with profiling enabled and generates flamegraphs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROFILE_DIR="${PROJECT_ROOT}/target/profiling"

# colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# check if running on Linux (perf is Linux-specific)
check_system() {
    if [[ "$(uname)" != "Linux" ]]; then
        log_warn "this script is optimized for Linux with perf support"
        log_warn "profiling on $(uname) may have limited features"
    fi
}

# check for required tools
check_tools() {
    local missing_tools=()

    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi

    local has_profiler=false
    if command -v perf &> /dev/null; then
        has_profiler=true
        log_info "found perf profiler"
    fi

    if command -v cargo-flamegraph &> /dev/null; then
        has_profiler=true
        log_info "found cargo-flamegraph"
    fi

    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log_error "missing required tools: ${missing_tools[*]}"
        exit 1
    fi

    if ! $has_profiler; then
        log_warn "no profiling tools found. Install with:"
        log_warn "  cargo install flamegraph"
        log_warn "  sudo apt install linux-perf  # or your distro's perf package"
    fi
}

# setup profiling directory
setup_profile_dir() {
    mkdir -p "${PROFILE_DIR}"
    log_info "profile output directory: ${PROFILE_DIR}"
}

# run benchmarks with criterion
run_criterion_benchmarks() {
    log_info "running criterion benchmarks..."
    cd "${PROJECT_ROOT}"
    cargo bench --bench compare
    log_info "criterion benchmarks complete. Results in target/criterion/"
}

# run benchmarks with perf profiling
run_perf_profile() {
    if ! command -v perf &> /dev/null; then
        log_warn "perf not found, skipping perf profiling"
        return
    fi

    log_info "running benchmarks with perf profiling..."
    cd "${PROJECT_ROOT}"

    RUSTFLAGS="-C force-frame-pointers=yes -C debug-assertions=off" \
        cargo build --release --bench compare

    local perf_data="${PROFILE_DIR}/perf.data"
    log_info "recording perf data to ${perf_data}"

    perf record -F 99 -g -o "${perf_data}" \
        ./target/release/deps/compare-* --bench 2>/dev/null || \
        log_warn "perf record failed or no samples collected"

    if [[ -f "${perf_data}" ]]; then
        log_info "generating perf report..."
        perf report -i "${perf_data}" --stdio > "${PROFILE_DIR}/perf_report.txt" 2>/dev/null || \
            log_warn "perf report generation failed"

        if command -v stackcollapse-perf.pl &> /dev/null && command -v flamegraph.pl &> /dev/null; then
            log_info "generating flamegraph from perf data..."
            perf script -i "${perf_data}" | \
                stackcollapse-perf.pl | \
                flamegraph.pl > "${PROFILE_DIR}/flamegraph_perf.svg"
            log_info "flamegraph saved to ${PROFILE_DIR}/flamegraph_perf.svg"
        fi
    fi
}

# run benchmarks with cargo-flamegraph
run_cargo_flamegraph() {
    if ! command -v cargo-flamegraph &> /dev/null; then
        log_warn "cargo-flamegraph not found, skipping"
        log_warn "install with: cargo install flamegraph"
        return
    fi

    log_info "running benchmarks with cargo-flamegraph..."
    cd "${PROJECT_ROOT}"

    cargo flamegraph --bench compare --output "${PROFILE_DIR}/flamegraph_cargo.svg" -- --bench

    if [[ -f "${PROFILE_DIR}/flamegraph_cargo.svg" ]]; then
        log_info "flamegraph saved to ${PROFILE_DIR}/flamegraph_cargo.svg"
    fi
}

# Run custom profiling benchmark
run_custom_profile() {
    log_info "running custom profiling benchmark..."
    cd "${PROJECT_ROOT}"

    local test_dir="${PROJECT_ROOT}/target/bench_profile_data"
    mkdir -p "${test_dir}"

    for i in {1..100}; do
        echo "this is test file $i with pattern1 and some content." > "${test_dir}/test_$i.txt"
    done

    log_info "running profiled search on test data..."

    if command -v perf &> /dev/null; then
        perf record -F 99 -g -o "${PROFILE_DIR}/search_perf.data" \
            ./target/release/rfgrep search "pattern1" "${test_dir}" --extensions txt

        if [[ -f "${PROFILE_DIR}/search_perf.data" ]]; then
            perf report -i "${PROFILE_DIR}/search_perf.data" --stdio > \
                "${PROFILE_DIR}/search_perf_report.txt"
            log_info "Search perf report saved to ${PROFILE_DIR}/search_perf_report.txt"
        fi
    else
        log_info "running search with timing..."
        time ./target/release/rfgrep search "pattern1" "${test_dir}" --extensions txt > /dev/null
    fi

    rm -rf "${test_dir}"
}

# Generate summary report
generate_summary() {
    log_info "generating profiling summary..."

    local summary_file="${PROFILE_DIR}/PROFILING_SUMMARY.md"
    cat > "${summary_file}" <<EOF
# rfgrep Benchmark Profiling Summary

Generated: $(date)

## Available Reports

EOF

    if [[ -f "${PROFILE_DIR}/perf_report.txt" ]]; then
        echo "- Perf Report: \`perf_report.txt\`" >> "${summary_file}"
    fi

    if [[ -f "${PROFILE_DIR}/flamegraph_perf.svg" ]]; then
        echo "- Perf Flamegraph: \`flamegraph_perf.svg\`" >> "${summary_file}"
    fi

    if [[ -f "${PROFILE_DIR}/flamegraph_cargo.svg" ]]; then
        echo "- Cargo Flamegraph: \`flamegraph_cargo.svg\`" >> "${summary_file}"
    fi

    if [[ -f "${PROFILE_DIR}/search_perf_report.txt" ]]; then
        echo "- Search Perf Report: \`search_perf_report.txt\`" >> "${summary_file}"
    fi

    cat >> "${summary_file}" <<EOF

## Criterion Benchmark Results

See detailed results in \`target/criterion/\` directory.

To view HTML reports:
\`\`\`bash
firefox target/criterion/report/index.html
\`\`\`

## How to Analyze Flamegraphs

1. Open the SVG files in a web browser
2. Click on any function to zoom in
3. Wider blocks = more CPU time
4. Look for:
   - Wide blocks at the bottom (hot functions)
   - Tall stacks (deep call chains)
   - Surprising function calls

## Common Optimization Targets

Based on typical grep workloads:
- File I/O operations
- Regex compilation and matching
- Memory allocations
- String operations
- Thread synchronization

## Next Steps

1. Review flamegraphs for hot spots
2. Check perf reports for cache misses
3. Profile specific slow functions
4. Run benchmarks before/after changes
EOF

    log_info "Summary saved to ${summary_file}"
}

# Main execution
main() {
    log_info "Starting rfgrep benchmark profiling"

    check_system
    check_tools
    setup_profile_dir

    run_criterion_benchmarks
    run_perf_profile
    run_cargo_flamegraph
    run_custom_profile

    generate_summary

    log_info "Profiling complete!"
    log_info "View results in: ${PROFILE_DIR}"

    log_info "\nGenerated files:"
    ls -lh "${PROFILE_DIR}"
}

# Run main
main "$@"
