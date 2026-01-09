#!/bin/bash
# Run Caret benchmarks

set -e

BENCH_DIR="target/criterion"
CARGO_BENCH="cargo bench"

echo "Running Caret benchmarks..."

# Parse arguments
VERBOSE=""
FILTER=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE="-- --verbose"
            shift
            ;;
        -f|--filter)
            FILTER="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [-v|--verbose] [-f|--filter BENCH]"
            exit 1
            ;;
    esac
done

# Run benchmarks
if [ -n "$FILTER" ]; then
    echo "Running benchmark: $FILTER"
    $CARGO_BENCH "$FILTER" $VERBOSE
else
    echo "Running all benchmarks..."
    $CARGO_BENCH $VERBOSE
fi

# Generate report
echo ""
echo "Benchmark reports saved to: $BENCH_DIR"
echo "Open with: firefox $BENCH_DIR/report/index.html"
