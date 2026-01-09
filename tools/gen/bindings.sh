#!/bin/bash
# Generate language bindings

set -e

echo "=== Generating Language Bindings ==="

# Generate Python bindings using PyO3
echo "Generating Python bindings..."
cd bindings/python
maturin develop || echo "Skipping Python bindings (maturin not installed)"
cd ../..

# Generate Node.js bindings using napi-rs
echo "Generating Node.js bindings..."
cd bindings/node
npm run build || echo "Skipping Node.js bindings (npm not installed)"
cd ../..

# Generate C header
echo "Generating C header..."
cargo run --bin caret_gen_c_header || echo "Skipping C header generation"

echo "=== Bindings Generation Complete ==="
