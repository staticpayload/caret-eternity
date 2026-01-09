# Code Generation Tools

This directory contains scripts for generating various artifacts.

## Scripts

| Script | Description |
|--------|-------------|
| `bindings.sh` | Generate language bindings |
| `docs.sh` | Generate documentation |
| `schema.sh` | Generate JSON schemas |

## Usage

### Generate Bindings

```bash
./tools/gen/bindings.sh
```

### Generate Documentation

```bash
./tools/gen/docs.sh
```

### Generate JSON Schema

```bash
./tools/gen/schema.sh
```

## Requirements

- `maturin`: For Python bindings
- `npm`: For Node.js bindings
- `cargo`: For documentation generation
