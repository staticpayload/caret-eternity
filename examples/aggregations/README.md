# Aggregations Example

Demonstrates windowing and aggregation operations in Caret.

## What It Does

This example shows different windowing strategies:
1. Tumbling window: Fixed-size, non-overlapping
2. Sliding window: Fixed-size, overlapping
3. Session window: Activity-based

## Running

```bash
caret run tumbling.json
caret run sliding.json
caret run session.json
```

## Window Types

### Tumbling Window

```
Time:  0----1----2----3----4----5----6
       |----|----|----|----|----|
       W1   W2   W3   W4   W5
```

Each window is 1 second, non-overlapping.

### Sliding Window

```
Time:  0----1----2----3----4----5
       |----|----|----|----|
        W1   W2   W3   W4
```

Window is 1 second, slides every 0.5 seconds.

### Session Window

```
Activity:  *  *  *        *  *    *  *  *
Session:  |---|        |--|     |-----|
          S1         S2      S3
```

Session created by activity gap.

## Aggregation Functions

- **sum**: Sum of all values
- **avg**: Average of values
- **min**: Minimum value
- **max**: Maximum value
- **count**: Count of values

## Concepts Demonstrated

- **Windowing**: Time-based grouping
- **Aggregation**: Summarizing data
- **Watermarks**: Handling late data
- **Triggers**: When to emit results
