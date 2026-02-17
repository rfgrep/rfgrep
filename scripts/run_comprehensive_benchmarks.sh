#!/bin/bash
set -eo pipefail

# Comprehensive benchmarking script for rfgrep
# This script runs various types of benchmarks and generates detailed reports

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_DIR/results/$(date +%Y-%m-%d_%H-%M-%S)"
BENCH_DATA_DIR="$PROJECT_DIR/bench_data"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo")
    fi
    
    if ! command -v hyperfine >/dev/null 2>&1; then
        missing_deps+=("hyperfine")
    fi
    
    if ! command -v valgrind >/dev/null 2>&1; then
        missing_deps+=("valgrind")
    fi
    
    if ! command -v strace >/dev/null 2>&1; then
        missing_deps+=("strace")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Install missing dependencies and try again"
        exit 1
    fi
    
    log_success "All dependencies found"
}

# Build the project
build_project() {
    log_info "Building rfgrep in release mode..."
    
    cd "$PROJECT_DIR"
    
    if ! cargo build --release; then
        log_error "Failed to build rfgrep"
        exit 1
    fi
    
    if [ ! -x "./target/release/rfgrep" ]; then
        log_error "rfgrep binary not found after build"
        exit 1
    fi
    
    log_success "Build completed successfully"
}

# Generate test data
generate_test_data() {
    log_info "Generating comprehensive test data..."
    
    mkdir -p "$BENCH_DATA_DIR"
    cd "$BENCH_DATA_DIR"
    
    log_info "Creating small files (1KB-10KB)..."
    for i in {1..1000}; do
        size=$((RANDOM % 9000 + 1000))
        head -c "$size" /dev/urandom > "small_$i.txt"
    done
    
    log_info "Creating medium files (100KB-1MB)..."
    for i in {1..100}; do
        size=$((RANDOM % 900000 + 100000))
        head -c "$size" /dev/urandom > "medium_$i.txt"
    done
    
    log_info "Creating large files (1MB-10MB)..."
    for i in {1..20}; do
        size=$((RANDOM % 9000000 + 1000000))
        head -c "$size" /dev/urandom > "large_$i.txt"
    done
    
    log_info "Creating source code files..."
    for i in {1..200}; do
        cat > "code_$i.rs" << EOF
// Test Rust file $i
fn main() {
    println!("Hello, world!");
    let x = $i;
    let y = x * 2;
    println!("The answer is: {}", y);
}

struct TestStruct$i {
    field1: i32,
    field2: String,
}

impl TestStruct$i {
    fn new() -> Self {
        Self {
            field1: 0,
            field2: String::new(),
        }
    }
}
EOF
    done
    
    log_info "Creating binary files..."
    for i in {1..50}; do
        size=$((RANDOM % 100000 + 10000))
        head -c "$size" /dev/urandom > "binary_$i.bin"
    done
    
    log_success "Test data generation completed"
}

# Run criterion benchmarks
run_criterion_benchmarks() {
    log_info "Running Criterion benchmarks..."
    
    cd "$PROJECT_DIR"
    mkdir -p "$RESULTS_DIR/criterion"
    
    if cargo bench -- --output-format html --output "$RESULTS_DIR/criterion/report.html"; then
        log_success "Criterion benchmarks completed"
    else
        log_warning "Criterion benchmarks failed, but continuing..."
    fi
}

# Run hyperfine benchmarks
run_hyperfine_benchmarks() {
    log_info "Running Hyperfine benchmarks..."
    
    cd "$PROJECT_DIR"
    mkdir -p "$RESULTS_DIR/hyperfine"
    
    log_info "Running basic search benchmarks..."
    hyperfine \
        --warmup 3 \
        --runs 10 \
        --export-json "$RESULTS_DIR/hyperfine/basic_search.json" \
        --export-markdown "$RESULTS_DIR/hyperfine/basic_search.md" \
        "./target/release/rfgrep '$BENCH_DATA_DIR' search 'pattern'" \
        "grep -r 'pattern' '$BENCH_DATA_DIR'" \
        "rg 'pattern' '$BENCH_DATA_DIR'" \
        "fd -X grep 'pattern' '$BENCH_DATA_DIR'" || log_warning "Basic search benchmarks failed"
    
    log_info "Running algorithm comparison benchmarks..."
    hyperfine \
        --warmup 2 \
        --runs 5 \
        --export-json "$RESULTS_DIR/hyperfine/algorithms.json" \
        --export-markdown "$RESULTS_DIR/hyperfine/algorithms.md" \
        "./target/release/rfgrep '$BENCH_DATA_DIR' search 'pattern' --algorithm boyer-moore" \
        "./target/release/rfgrep '$BENCH_DATA_DIR' search 'pattern' --algorithm regex" \
        "./target/release/rfgrep '$BENCH_DATA_DIR' search 'pattern' --algorithm simple" || log_warning "Algorithm benchmarks failed"
    
    log_info "Running file type filtering benchmarks..."
    hyperfine \
        --warmup 2 \
        --runs 5 \
        --export-json "$RESULTS_DIR/hyperfine/file_types.json" \
        --export-markdown "$RESULTS_DIR/hyperfine/file_types.md" \
        "./target/release/rfgrep '$BENCH_DATA_DIR' search 'pattern' --extensions txt" \
        "./target/release/rfgrep '$BENCH_DATA_DIR' search 'pattern' --extensions rs" \
        "rg 'pattern' -g '*.txt' '$BENCH_DATA_DIR'" \
        "rg 'pattern' -g '*.rs' '$BENCH_DATA_DIR'" || log_warning "File type benchmarks failed"
    
    log_success "Hyperfine benchmarks completed"
}

# Run memory profiling
run_memory_profiling() {
    log_info "Running memory profiling..."
    
    cd "$PROJECT_DIR"
    mkdir -p "$RESULTS_DIR/memory"
    
    log_info "Running Valgrind massif profiling..."
    if valgrind --tool=massif --stacks=yes --massif-out-file="$RESULTS_DIR/memory/massif.out" \
        ./target/release/rfgrep "$BENCH_DATA_DIR" search "pattern" >/dev/null 2>&1; then
        ms_print "$RESULTS_DIR/memory/massif.out" > "$RESULTS_DIR/memory/massif_report.txt"
        log_success "Valgrind profiling completed"
    else
        log_warning "Valgrind profiling failed"
    fi
    
    log_info "Running memory usage monitoring..."
    if command -v /usr/bin/time >/dev/null 2>&1; then
        /usr/bin/time -v ./target/release/rfgrep "$BENCH_DATA_DIR" search "pattern" >/dev/null 2>&1 2> "$RESULTS_DIR/memory/time_output.txt" || true
    fi
}

# Run I/O profiling
run_io_profiling() {
    log_info "Running I/O profiling..."
    
    cd "$PROJECT_DIR"
    mkdir -p "$RESULTS_DIR/io"
    
    log_info "Running strace profiling..."
    if strace -c -f -o "$RESULTS_DIR/io/strace.txt" \
        ./target/release/rfgrep "$BENCH_DATA_DIR" search "pattern" >/dev/null 2>&1; then
        log_success "Strace profiling completed"
    else
        log_warning "Strace profiling failed"
    fi
}

# Run performance tests
run_performance_tests() {
    log_info "Running performance tests..."
    
    cd "$PROJECT_DIR"
    mkdir -p "$RESULTS_DIR/performance"
    
    if cargo test --release test_performance_harness -- --nocapture > "$RESULTS_DIR/performance/harness_output.txt" 2>&1; then
        log_success "Performance tests completed"
    else
        log_warning "Performance tests failed"
    fi
}

# Generate summary report
generate_summary_report() {
    log_info "Generating summary report..."
    
    cd "$PROJECT_DIR"
    
    cat > "$RESULTS_DIR/SUMMARY.md" << EOF
# rfgrep Benchmark Results

**Date:** $(date)
**System:** $(uname -a)
**CPU:** $(lscpu | grep "Model name" | cut -d: -f2 | xargs)
**Memory:** $(free -h | grep "Mem:" | awk '{print $2}')
**Rust Version:** $(rustc --version)

## Benchmark Categories

1. **Criterion Benchmarks** - Micro-benchmarks for individual components
2. **Hyperfine Benchmarks** - Command-line tool comparisons
3. **Memory Profiling** - Memory usage analysis
4. **I/O Profiling** - System call analysis
5. **Performance Tests** - Custom performance harness tests

## Results

- Criterion results: \`criterion/\`
- Hyperfine results: \`hyperfine/\`
- Memory profiling: \`memory/\`
- I/O profiling: \`io/\`
- Performance tests: \`performance/\`

## Quick Stats

- Test data size: $(du -sh "$BENCH_DATA_DIR" 2>/dev/null | cut -f1 || echo "Unknown")
- Total files: $(find "$BENCH_DATA_DIR" -type f | wc -l 2>/dev/null || echo "Unknown")
- Results directory: \`$RESULTS_DIR\`

EOF

    log_success "Summary report generated"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up temporary files..."
    rm -rf "$BENCH_DATA_DIR"  
    log_success "Cleanup completed"
}

# Main function
main() {
    log_info "Starting comprehensive rfgrep benchmarking..."
    log_info "Results will be saved to: $RESULTS_DIR"
    
    mkdir -p "$RESULTS_DIR"
    
    check_dependencies
    build_project
    generate_test_data
    run_criterion_benchmarks
    run_hyperfine_benchmarks
    run_memory_profiling
    run_io_profiling
    run_performance_tests
    generate_summary_report
    cleanup
    
    log_success "All benchmarks completed successfully!"
    log_info "Results available in: $RESULTS_DIR"
    log_info "View summary: cat $RESULTS_DIR/SUMMARY.md"
}

main "$@"
