# Simple Graph Example

A basic data flow pipeline that demonstrates the fundamental concepts of Caret.

## What It Does

This example:
1. Generates a sequence of numbers (1, 2, 3, 4, 5)
2. Multiplies each number by 2
3. Filters for values greater than 5
4. Prints the results

## Graph Structure

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ Sequence │────▶│   Map    │────▶│  Filter  │────▶│ Console  │
│  1-5     │     │  x * 2   │     │  > 5     │     │  Print   │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
```

## Running

```bash
caret run graph.json
```

## Expected Output

```
6
8
10
```

## Concepts Demonstrated

- **Sequential Nodes**: Data flows from node to node
- **Transformation**: The Map node modifies values
- **Filtering**: The Filter node selects values
- **Output**: The Console node displays results
