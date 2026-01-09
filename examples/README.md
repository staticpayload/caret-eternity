# Caret Examples

This directory contains example graphs and code demonstrating Caret functionality.

## Examples

| Directory | Description |
|-----------|-------------|
| [simple-graph/](simple-graph/) | Basic data flow pipeline |
| [distributed/](distributed/) | Multi-node distributed execution |
| [custom-node/](custom-node/) | Creating custom nodes |
| [streaming/](streaming/) | Continuous stream processing |
| [transformations/](transformations/) | Data transformation examples |
| [aggregations/](aggregations/) | Windowing and aggregation |

## Running Examples

### Run with caret CLI

```bash
# Build caret
cargo build --release

# Run example
./target/release/caret run examples/simple-graph/graph.json
```

### Run with cargo

```bash
cargo run --bin caret -- run examples/simple-graph/graph.json
```

## Example Structure

Each example follows this structure:

```
example-name/
├── README.md           # Description
├── graph.json          # Graph definition
├── input/              # Sample input (optional)
└── expected/           # Expected output (optional)
```

## Creating New Examples

1. Create directory: `examples/my-example/`
2. Add `README.md` with description
3. Create `graph.json` with graph definition
4. Test the example
5. Update this `README.md`

## Troubleshooting

### Example won't run

1. Check graph syntax: `caret validate examples/my-example/graph.json`
2. Verify caret is built: `cargo build --release`
3. Check error output with `--verbose`

### Unexpected output

1. Review the example README
2. Check input data format
3. Verify node configuration

## Contributing

New examples welcome! Please:
- Keep examples simple and focused
- Include detailed documentation
- Test on multiple platforms when applicable
- Follow the example structure
