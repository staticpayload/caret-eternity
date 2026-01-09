#!/bin/bash
# Watch for changes and run tests

if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

echo "=== Watching for Changes ==="
cargo watch -x 'test --workspace'
