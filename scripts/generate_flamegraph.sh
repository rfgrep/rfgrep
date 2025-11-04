#!/bin/bash
# generate a flamegraph for the benchmarks

# exit on error
set -e

# ensure flamegraph is installed
if ! command -v flamegraph &> /dev/null
then
    echo "flamegraph could not be found, please install it with 'cargo install flamegraph'"
    exit 1
fi

# create results directory if it doesn't exist
mkdir -p ./results

# build the benchmark binary first
echo "building benchmark binary..."
cargo build --release --bench compare

# run benchmarks with flamegraph profiling
echo "running benchmarks with profiling..."
flamegraph -o ./results/benchmark_flamegraph.svg -- ./target/release/deps/compare-* --bench

echo "flamegraph generated at ./results/benchmark_flamegraph.svg"
