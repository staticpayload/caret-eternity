#!/bin/bash
# Clean build artifacts and temporary files

set -e

echo "Cleaning Caret workspace..."

# Standard cargo clean
cargo clean

# Additional cleanup
rm -rf \
    .cargo/ \
    *.db \
    *.log \
    *.profraw \
    *.profdata \
    flamegraph.svg \
    caret-*.tar.gz

# Clean example builds
find examples -name "target" -type d -exec rm -rf {} + 2>/dev/null || true

# Clean backup files
find . -name "*.bak" -delete
find . -name "*~" -delete
find . -name "*.swp" -delete
find . -name "*.swo" -delete

echo "Clean complete!"
