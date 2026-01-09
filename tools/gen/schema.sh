#!/bin/bash
# Generate JSON schema for graph validation

set -e

OUTPUT="${1:-schema/graph-schema.json}"

echo "=== Generating JSON Schema ==="

mkdir -p "$(dirname "$OUTPUT")"

# Generate schema from graph structure
cat > "$OUTPUT" << 'EOF'
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Caret Graph Definition",
  "type": "object",
  "required": ["nodes", "edges"],
  "properties": {
    "version": {"type": "string"},
    "name": {"type": "string"},
    "description": {"type": "string"},
    "nodes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "type"],
        "properties": {
          "id": {"type": "string"},
          "type": {"type": "string"},
          "config": {"type": "object"},
          "metadata": {"type": "object"}
        }
      }
    },
    "edges": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["from", "to"],
        "properties": {
          "id": {"type": "string"},
          "from": {"type": "string"},
          "from_port": {"type": "string"},
          "to": {"type": "string"},
          "to_port": {"type": "string"}
        }
      }
    },
    "partitioning": {
      "type": "object",
      "properties": {
        "strategy": {"type": "string"},
        "assignments": {"type": "object"}
      }
    }
  }
}
EOF

echo "=== Schema Generated ==="
echo "Location: $OUTPUT"
