# Streaming Example

Demonstrates continuous stream processing with Caret.

## What It Does

This example sets up a continuous data processing pipeline:
1. Reads from a continuous data source
2. Processes each value in real-time
3. Outputs results continuously

## Use Cases

- Real-time analytics
- Log processing
- Sensor data processing
- Event stream processing

## Running

```bash
# Run the streaming graph
caret run graph.json

# Run with duration limit
caret run --duration 60 graph.json
```

## Stopping

Press `Ctrl+C` to stop the streaming graph gracefully.

## Graph Features

- **Continuous Source**: Generates data indefinitely
- **Low Latency**: Processes data as it arrives
- **Backpressure Handling**: Manages flow control
- **Graceful Shutdown**: Handles SIGTERM/SIGINT

## Concepts Demonstrated

- **Stream Processing**: Continuous data flow
- **Real-time Processing**: Low-latency execution
- **Backpressure**: Flow control mechanisms
- **Graceful Shutdown**: Clean resource cleanup
