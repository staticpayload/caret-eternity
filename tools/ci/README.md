# CI Tools

This directory contains scripts for continuous integration.

## Scripts

| Script | Description |
|--------|-------------|
| `run_tests.sh` | Run full test suite |
| `build.sh` | Build for all targets |
| `coverage.sh` | Generate coverage report |

## Usage

### Local CI Run

```bash
./tools/ci/run_tests.sh
```

### Build for All Platforms

```bash
./tools/ci/build.sh
```

### Generate Coverage

```bash
./tools/ci/coverage.sh
```

## GitHub Actions

These scripts are used by `.github/workflows/ci.yml`.
