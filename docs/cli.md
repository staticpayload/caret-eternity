# Caret CLI Reference

The `caret` command-line interface is the primary way to interact with Caret graphs.

## Installation

See [Installation Guide](guide/installation.md) for installation instructions.

## Basic Usage

```bash
caret [OPTIONS] <SUBCOMMAND>
```

## Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Print help information |
| `--version` | `-V` | Print version information |
| `--verbose` | `-v` | Increase verbosity (can be used multiple times) |
| `--quiet` | `-q` | Decrease verbosity |
| `--config <PATH>` | `-c` | Use specified config file |

## Subcommands

### run

Execute a Caret graph.

```bash
caret run [OPTIONS] <GRAPH>
```

**Arguments:**
- `<GRAPH>` - Path to graph file or graph ID

**Options:**
- `--workers <N>` - Number of worker threads (default: auto)
- `--profile` - Enable performance profiling
- `--trace` - Enable trace logging
- `--duration <SECONDS>` - Run for specified duration then exit

**Examples:**
```bash
# Run a graph from file
caret run my_graph.json

# Run with 4 workers
caret run --workers 4 my_graph.json

# Run with profiling
caret run --profile my_graph.json

# Run for 60 seconds then exit
caret run --duration 60 my_graph.json
```

### build

Validate and compile a graph without executing.

```bash
caret build [OPTIONS] <GRAPH>
```

**Arguments:**
- `<GRAPH>` - Path to graph file

**Options:**
- `--output <PATH>` - Write compiled graph to file
- `--format <FORMAT>` - Output format (json, bincode)

**Examples:**
```bash
# Validate a graph
caret build my_graph.json

# Compile to binary format
caret build --output compiled.bin --format bincode my_graph.json
```

### validate

Check a graph for errors without building.

```bash
caret validate <GRAPH>
```

**Arguments:**
- `<GRAPH>` - Path to graph file

**Checks performed:**
- Syntax validation
- Node existence
- Port connectivity
- Cycle detection

**Exit codes:**
- `0` - Graph is valid
- `1` - Graph has errors

### info

Display information about a graph.

```bash
caret info [OPTIONS] <GRAPH>
```

**Arguments:**
- `<GRAPH>` - Path to graph file

**Options:**
- `--format <FORMAT>` - Output format (text, json)
- `--verbose` - Show detailed information

**Output includes:**
- Node count
- Edge count
- Input/output ports
- Topology information
- Memory requirements

**Examples:**
```bash
# Show graph info
caret info my_graph.json

# JSON output
caret info --format json my_graph.json
```

### distributed

Run a distributed Caret node.

```bash
caret distributed [OPTIONS]
```

**Options:**
- `--mode <MODE>` - Node mode: `coordinator` or `worker` (default: `coordinator`)
- `--bind <ADDR>` - Bind address (default: `0.0.0.0:9234`)
- `--connect <ADDR>` - Connect to coordinator (worker mode)
- `--discovery <MODE>` - Discovery mode: `mdns`, `static`, `none`
- `--node-id <ID>` - Custom node ID (default: auto-generated)

**Examples:**
```bash
# Start a coordinator
caret distributed --mode coordinator

# Start a worker
caret distributed --mode worker --connect 192.168.1.100:9234

# Start with custom bind address
caret distributed --mode coordinator --bind 0.0.0.0:8080
```

### plugin

Manage Caret plugins.

```bash
caret plugin <SUBCOMMAND>
```

#### plugin list

List installed plugins.

```bash
caret plugin list
```

#### plugin install

Install a plugin from a file.

```bash
caret plugin install <PLUGIN_FILE>
```

#### plugin remove

Remove an installed plugin.

```bash
caret plugin remove <PLUGIN_NAME>
```

#### plugin info

Show plugin information.

```bash
caret plugin info <PLUGIN_NAME>
```

### benchmark

Run performance benchmarks.

```bash
caret benchmark [OPTIONS] [NAME]
```

**Arguments:**
- `[NAME]` - Specific benchmark to run (default: all)

**Options:**
- `--iterations <N>` - Number of iterations
- `--output <PATH>` - Write results to file
- `--format <FORMAT>` - Output format (text, json, csv)

**Available benchmarks:**
- `scheduler` - Scheduler performance
- `executor` - Executor tick performance
- `buffer` - Buffer operations
- `queue` - Queue operations
- `serialization` - Graph serialization

**Examples:**
```bash
# Run all benchmarks
caret benchmark

# Run specific benchmark
caret benchmark scheduler

# Run with custom iterations
caret benchmark --iterations 100000 scheduler

# Export results
caret benchmark --output results.json --format json
```

### doctor

Check system configuration and dependencies.

```bash
caret doctor
```

**Checks:**
- Rust toolchain version
- CPU information
- Memory availability
- Network configuration
- Plugin paths

### completion

Generate shell completion scripts.

```bash
caret completion <SHELL>
```

**Shells:**
- `bash`
- `elvish`
- `fish`
- `powershell`
- `zsh`

**Examples:**
```bash
# Generate bash completions
caret completion bash > /etc/bash_completion.d/caret

# Generate zsh completions
caret completion zsh > ~/.zfunc/_caret
```

### config

Manage Caret configuration.

```bash
caret config <SUBCOMMAND>
```

#### config get

Get a configuration value.

```bash
caret config get <KEY>
```

#### config set

Set a configuration value.

```bash
caret config set <KEY> <VALUE>
```

#### config list

List all configuration.

```bash
caret config list
```

**Configuration keys:**
- `worker_threads` - Default worker thread count
- `buffer_size` - Default buffer size
- `log_level` - Default log level
- `plugin_path` - Plugin search path

### log

View and filter logs.

```bash
caret log [OPTIONS]
```

**Options:**
- `--follow` - Follow log output (like `tail -f`)
- `--since <TIME>` - Show logs since time
- `--level <LEVEL>` - Filter by log level
- `--node <ID>` - Filter by node ID

**Examples:**
```bash
# Follow logs
caret log --follow

# Show error logs only
caret log --level error

# Show logs for specific node
caret log --node node-123
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CARET_CONFIG` | Path to config file |
| `CARET_LOG_LEVEL` | Default log level |
| `CARET_PLUGIN_PATH` | Plugin search path |
| `CARET_WORKERS` | Default worker count |
| `CARET_BIND_ADDR` | Default bind address |
| `NO_COLOR` | Disable colored output |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid usage |
| 3 | Graph validation failed |
| 4 | Runtime execution failed |
| 5 | Plugin error |
| 6 | Network error |

## Examples

### Complete Workflow

```bash
# Validate a graph
caret validate my_graph.json

# Get graph information
caret info my_graph.json

# Run the graph
caret run my_graph.json

# Run with profiling
caret run --profile my_graph.json > results.txt
```

### Distributed Execution

```bash
# Terminal 1: Start coordinator
caret distributed --mode coordinator --bind 0.0.0.0:9234

# Terminal 2: Start worker 1
caret distributed --mode worker --connect localhost:9234

# Terminal 3: Start worker 2
caret distributed --mode worker --connect localhost:9234

# Terminal 4: Submit graph to coordinator
caret run --coordinator localhost:9234 my_graph.json
```

## See Also

- [User Guide](guide/)
- [Developer Guide](dev/)
- [Configuration](guide/configuration.md)
