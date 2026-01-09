# Value Types Specification

This specification defines the Caret value type system.

## Status: Stable

## Version: 1.0

## Overview

Caret uses a universal value type that can represent all supported data types.

## Type Definition

```
enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Arc<str>),
    Bytes(Arc<Vec<u8>>),
    List(Arc<Vec<Value>>),
    Map(Arc<HashMap<String, Value>>),
}
```

## Type Descriptions

### Null

Represents absence of value.

```
Value::Null
```

**JSON**: `null`

### Bool

Boolean value.

```
Value::Bool(true)
Value::Bool(false)
```

**JSON**: `true` or `false`

### Int

64-bit signed integer.

```
Value::Int(42)
Value::Int(-100)
Value::Int(i64::MAX)
```

**Range**: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807

**JSON**: `{"int": 42}`

### Float

64-bit floating point number.

```
Value::Float(3.14)
Value::Float(-0.001)
Value::Float(f64::INFINITY)
```

**Special values**: Infinity, -Infinity, NaN

**JSON**: `{"float": 3.14}`

### String

UTF-8 encoded string.

```
Value::String("hello")
```

**Encoding**: UTF-8
**Max length**: Limited by available memory

**JSON**: `{"string": "hello"}`

### Bytes

Byte array.

```
Value::Bytes(vec![0x00, 0x01, 0x02])
```

**Use cases**: Binary data, blobs

**JSON**: `{"bytes": "AAEC"}` (base64 encoded)

### List

Ordered collection of values.

```
Value::List(vec![
    Value::Int(1),
    Value::Int(2),
    Value::Int(3),
])
```

**Properties**:
- Ordered: Elements maintain insertion order
- Heterogeneous: Can contain mixed types
- Indexable: O(1) random access

**JSON**: `{"list": [1, 2, 3]}`

### Map

Key-value mapping.

```
let mut map = HashMap::new();
map.insert("name".into(), Value::String("Alice".into()));
map.insert("age".into(), Value::Int(30));
Value::Map(Arc::new(map))
```

**Properties**:
- Unordered: No guaranteed iteration order
- String keys: All keys are strings
- Heterogeneous: Values can be any type

**JSON**: `{"map": {"name": {"string": "Alice"}, "age": {"int": 30}}}`

## Type Operations

### Equality

```
Value::Int(1) == Value::Int(1)  // true
Value::Int(1) == Value::Int(2)  // false
Value::Null == Value::Null     // true
```

### Ordering

```
Value::Int(1) < Value::Int(2)     // true
Value::String("a") < Value::String("b")  // true
```

Different types are not comparable.

### Cloning

Values use Arc for cheap cloning:

```
let v = Value::String("hello".into());
let v2 = v.clone();  // O(1) - just increments Arc ref count
```

## Type Coercion

### Implicit Coercion

| From | To | Method |
|------|-----|--------|
| Int | Float | Direct conversion |
| Float | Int | Truncation |
| Null | Any | Default value |

### Explicit Coercion

```rust
// To string
value.as_string()

// To integer
value.as_int()

// To float
value.as_float()

// To bytes
value.as_bytes()
```

## Type Checking

### Type Predicates

```rust
value.is_null()    // bool
value.is_bool()    // bool
value.is_int()     // bool
value.is_float()   // bool
value.is_string()  // bool
value.is_bytes()   // bool
value.is_list()    // bool
value.is_map()     // bool
```

### Type Queries

```rust
value.type_name()  // Returns "int", "string", etc.
value.is_numeric() // Returns true for int or float
```

## Memory Layout

### Size

| Type | Size (bytes) |
|------|--------------|
| Null | 24 (enum overhead) |
| Bool | 24 |
| Int | 32 |
| Float | 32 |
| String | 32 + heap |
| Bytes | 32 + heap |
| List | 32 + heap |
| Map | 32 + heap |

### Arc Optimization

String, Bytes, List, and Map use Arc for sharing:
- Clone: O(1)
- Drop: O(1) (unless last reference)

## Performance

### Operations

| Operation | Complexity |
|-----------|------------|
| Clone | O(1)* |
| Equality | O(n) |
| Comparison | O(n) |
| Hash | O(n) |

* For Arc-backed types

### Best Practices

1. **Use Arc for sharing**: Values already use Arc internally
2. **Avoid large maps/maps**: Consider references for large data
3. **Prefer Int over Float**: When exact precision needed

## Examples

### Creating Values

```rust
// Primitive types
let n = Value::Null;
let b = Value::Bool(true);
let i = Value::Int(42);
let f = Value::Float(3.14);

// Complex types
let s = Value::String("hello".into());
let bytes = Value::Bytes(vec![0, 1, 2].into());
let list = Value::List(vec![Value::Int(1), Value::Int(2)].into());
let map = Value::Map(HashMap::new().into());
```

### Pattern Matching

```rust
match value {
    Value::Null => println!("null"),
    Value::Bool(b) => println!("bool: {}", b),
    Value::Int(i) => println!("int: {}", i),
    Value::Float(f) => println!("float: {}", f),
    Value::String(s) => println!("string: {}", s),
    Value::Bytes(b) => println!("bytes: {} bytes", b.len()),
    Value::List(l) => println!("list: {} items", l.len()),
    Value::Map(m) => println!("map: {} entries", m.len()),
}
```

### Converting

```rust
// From Rust types
let v: Value = 42.into();
let v: Value = "hello".into();

// To Rust types
let i: Option<i64> = value.as_int();
let s: Option<&str> = value.as_str();
```

## Compliance

Implementations MUST:
1. Support all value types
2. Use Arc for String, Bytes, List, Map
3. Provide type checking methods
4. Handle type errors gracefully

Implementations SHOULD:
1. Provide conversion helpers
2. Optimize common operations
3. Document type coercion rules

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-01 | Initial specification |
