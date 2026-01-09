# Getting Started with Caret

This guide will help you get up and running with Caret quickly.

## What You'll Learn

- What Caret is and what it can do
- How to install Caret
- How to run your first graph
- Basic concepts and terminology

## What is Caret?

Caret is a high-performance stream processing engine that:

- **Processes data flows** as directed graphs
- **Executes efficiently** using Rust
- **Scales horizontally** across machines
- **Extends dynamically** with plugins

### Common Use Cases

- **Real-time Analytics**: Process streaming data with low latency
- **IoT Edge Computing**: Run data processing on edge devices
- **Data Pipelines**: Transform and route data between systems
- **Desktop Automation**: Automate workflows on your computer

## Installation

### Prerequisites

Caret requires:
- **Linux**, **macOS**, or **Windows**
- **x86_64** or **ARM64** architecture

### Quick Install

#### Using install script (Linux/macOS):

```bash
curl -sSL https://install.caret.sh | sh
```

#### Using Homebrew (macOS):

```bash
brew install caret
```

#### Download Binary:

Visit [releases](https://github.com/staticpayload/caret-eternity/releases) and download the binary for your platform.

#### Build from Source:

```bash
# Clone repository
git clone https://github.com/staticpayload/caret-eternity.git
cd caret-eternity

# Build and install
cargo install --path .
```

See [Installation](installation.md) for detailed instructions.

## Verify Installation

```bash
$ caret --version
caret 0.1.0

$ caret --help
Caret - Stream Processing Engine

USAGE:
    caret [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
    -v, --verbose    Increase verbosity

SUBCOMMANDS:
    run          Execute a graph
    build        Compile a graph
    validate     Check a graph for errors
    info         Display graph information
    distributed  Run distributed node
    ...
```

## Your First Graph

Let's create a simple graph that generates numbers and doubles them.

### 1. Create the Graph Definition

Create a file called `first_graph.json`:

```json
{
  "nodes": [
    {
      "id": "source",
      "type": "generator",
      "config": {
        "interval_ms": 100,
        "values": [1, 2, 3, 4, 5]
      }
    },
    {
      "id": "double",
      "type": "map",
      "config": {
        "expression": "x * 2"
      }
    },
    {
      "id": "print",
      "type": "console",
      "config": {}
    }
  ],
  "edges": [
    {
      "from": "source",
      "to": "double"
    },
    {
      "from": "double",
      "to": "print"
    }
  ]
}
```

### 2. Run the Graph

```bash
caret run first_graph.json
```

You should see output like:

```
2
4
6
8
10
```

### 3. Understanding What Happened

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│ Source  │────▶│  Double │────▶│  Print  │
│ 1,2,3.. │     │  x*2    │     │ Console │
└─────────┘     └─────────┘     └─────────┘
```

1. **Source node** generated numbers: 1, 2, 3, 4, 5
2. **Map node** doubled each value: 2, 4, 6, 8, 10
3. **Console node** printed the results

## Core Concepts

### Nodes

Nodes are the building blocks of Caret graphs. Each node:

- Has a unique ID
- Has a type that defines its behavior
- Has zero or more input ports
- Has zero or more output ports
- Contains configuration options

### Edges

Edges connect nodes together:

- Connect an output port to an input port
- Define the flow of data through the graph
- Can be one-to-one or one-to-many

### Ports

Ports are the connection points for edges:

- **Input ports**: Receive data from other nodes
- **Output ports**: Send data to other nodes
- **Typed**: Each port expects specific data types

### Scheduler

The scheduler determines which nodes to execute:

- Checks which nodes have available data
- Executes ready nodes
- Manages backpressure

## Next Steps

1. **Learn More**: Read [Your First Graph](first-graph.md) for detailed graph creation
2. **Explore Examples**: Check out the [examples/](../../examples/) directory
3. **Go Distributed**: Learn about [Distributed Execution](distributed.md)
4. **Extend Caret**: Read about [Plugins](plugins.md)

## Common Issues

### "caret: command not found"

Make sure Caret is installed and in your PATH:

```bash
# Check if caret is in PATH
which caret

# If not, add to PATH (example for macOS/Linux)
export PATH="$PATH:$HOME/.cargo/bin"
```

### "Failed to load graph"

Check that:
- The JSON file is valid
- All node types are registered
- Required plugins are installed

### Graph runs but produces no output

Check that:
- Edges are properly connected
- Node configuration is correct
- Use `--verbose` flag for more information

## Getting Help

- **Documentation**: [docs/](../)
- **Issues**: [GitHub Issues](https://github.com/staticpayload/caret-eternity/issues)
- **Discussions**: [GitHub Discussions](https://github.com/staticpayload/caret-eternity/discussions)

Happy streaming!
