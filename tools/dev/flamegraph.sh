#!/bin/bash
# Generate flamegraph for performance analysis

if ! command -v cargo-flamegraph &> /dev/null; then
    echo "Installing cargo-flamegraph..."
    cargo install flamegraph
fi

BINARY="${1:-caret}"
ARGS="${2:---run examples/simple-graph/graph.json}"

echo "=== Generating Flamegraph ==="
cargo flamegraph --bin "$BINARY" -- $ARGS

echo "=== Flamegraph Generated ==="
echo "Open: flamegraph.svg"
