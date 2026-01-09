# Transformations Example

Demonstrates various data transformation operations in Caret.

## What It Does

This example shows different transformation patterns:
1. Map: Transform each value
2. Filter: Select values matching criteria
3. FlatMap: One-to-many transformations
4. Reduce: Aggregate to single value

## Running

```bash
caret run graph.json
```

## Transformations

### Map
```
Input: 1, 2, 3, 4, 5
Expression: x * 2
Output: 2, 4, 6, 8, 10
```

### Filter
```
Input: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
Predicate: x > 5
Output: 6, 7, 8, 9, 10
```

### FlatMap
```
Input: [1, 2, 3]
Expression: expand(x)
Output: 1, 1, 2, 2, 3, 3
```

### Reduce
```
Input: 1, 2, 3, 4, 5
Function: sum
Output: 15
```

## Graph Structure

```
Input ──▶ Map ──▶ Filter ──▶ FlatMap ──▶ Reduce ──▶ Output
```

## Concepts Demonstrated

- **Map**: Element-wise transformation
- **Filter**: Element selection
- **FlatMap**: Element expansion
- **Reduce**: Aggregation
- **Composition**: Combining transformations
