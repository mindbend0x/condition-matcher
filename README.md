# Condition Matcher

A flexible and type-safe condition matching library for Rust with automatic struct field access.

## Features

- **Automatic struct matching** with derive macro
- Multiple matching modes (AND, OR, XOR)
- Support for various condition types (value, length, type, field)
- **Numeric comparisons** on fields (>, <, >=, <=)
- **String operations** (contains, starts_with, ends_with)
- **Regex matching** (optional feature)
- **NOT operator** for negating conditions
- **Optional field handling** (Option<T> support)
- **Detailed match results** with error information
- **Builder pattern** for ergonomic API
- **Serde support** (optional feature)
- Zero-cost abstractions with compile-time type safety

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
condition-matcher = "0.1.0"

# Optional features
condition-matcher = { version = "0.1.0", features = ["serde", "regex"] }
# Or all features
condition-matcher = { version = "0.1.0", features = ["full"] }
```

## Quick Start

```rust
use condition_matcher::{
    Matcher, MatcherMode, Condition, ConditionSelector, ConditionOperator,
    Matchable, MatchableDerive
};

// Simply derive Matchable to get automatic field access!
#[derive(MatchableDerive, PartialEq, Debug)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

let user = User {
    name: "Alice".to_string(),
    age: 30,
    email: Some("alice@example.com".to_string()),
};

// Create a matcher with AND mode
let mut matcher = Matcher::new(MatcherMode::AND);
matcher
    .add_condition(Condition {
        selector: ConditionSelector::FieldValue("age", &18u32),
        operator: ConditionOperator::GreaterThanOrEqual,
    })
    .add_condition(Condition {
        selector: ConditionSelector::FieldValue("name", &"lic"),
        operator: ConditionOperator::Contains,
    });

assert!(matcher.run(&user).unwrap());
```

## Builder API

For a more ergonomic experience, use the builder pattern:

```rust
use condition_matcher::{MatcherBuilder, MatcherMode};

let matcher = MatcherBuilder::<&str>::new()
    .mode(MatcherMode::AND)
    .length_gte(4)
    .value_not_equals("bad")
    .build();

assert!(matcher.run(&"good").unwrap());
```

Or use the field condition builder:

```rust
use condition_matcher::{field, Matcher, MatcherMode, Matchable, MatchableDerive};

#[derive(MatchableDerive, PartialEq)]
struct Product {
    price: f64,
}

let condition = field::<Product>("price").gte(&50.0f64);
let mut matcher = Matcher::new(MatcherMode::AND);
matcher.add_condition(condition);
```

## Matching Modes

### AND Mode
All conditions must match:

```rust
let mut matcher = Matcher::new(MatcherMode::AND);
```

### OR Mode
At least one condition must match:

```rust
let mut matcher = Matcher::new(MatcherMode::OR);
```

### XOR Mode
Exactly one condition must match:

```rust
let mut matcher = Matcher::new(MatcherMode::XOR);
```

## Condition Types

### Value Matching
```rust
Condition {
    selector: ConditionSelector::Value("Alice"),
    operator: ConditionOperator::Equals,
}
```

### Length Matching
```rust
Condition {
    selector: ConditionSelector::Length(5),
    operator: ConditionOperator::GreaterThanOrEqual,
}
```

### Field Value Matching
```rust
Condition {
    selector: ConditionSelector::FieldValue("age", &18u32),
    operator: ConditionOperator::GreaterThanOrEqual,
}
```

### NOT Operator
```rust
let inner = Condition {
    selector: ConditionSelector::FieldValue("active", &true),
    operator: ConditionOperator::Equals,
};

Condition {
    selector: ConditionSelector::Not(Box::new(inner)),
    operator: ConditionOperator::Equals,
}
```

## Supported Operators

| Operator | Description | Works With |
|----------|-------------|------------|
| `Equals` | Exact equality | All types |
| `NotEquals` | Inequality | All types |
| `GreaterThan` | Greater than | Numeric types, strings |
| `LessThan` | Less than | Numeric types, strings |
| `GreaterThanOrEqual` | Greater or equal | Numeric types, strings |
| `LessThanOrEqual` | Less or equal | Numeric types, strings |
| `Contains` | Contains substring | Strings |
| `NotContains` | Does not contain | Strings |
| `StartsWith` | Starts with prefix | Strings |
| `EndsWith` | Ends with suffix | Strings |
| `Regex` | Matches regex pattern | Strings (requires `regex` feature) |
| `IsEmpty` | Check if empty | Strings, collections |
| `IsNotEmpty` | Check if not empty | Strings, collections |
| `IsNone` | Check if Option is None | Option types |
| `IsSome` | Check if Option is Some | Option types |

## Supported Types

The matcher automatically supports comparison for:

- **Integers**: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- **Unsigned**: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- **Floats**: `f32`, `f64`
- **Other**: `bool`, `char`, `String`, `&str`

## Detailed Results

Get detailed information about why a match succeeded or failed:

```rust
let result = matcher.run_detailed(&user).unwrap();

println!("Match: {}", result.is_match());
println!("Passed: {}", result.passed_conditions().len());
println!("Failed: {}", result.failed_conditions().len());

for condition in result.condition_results {
    println!("  {} - {}", 
        if condition.passed { "PASS" } else { "FAIL" },
        condition.description
    );
    if let Some(error) = condition.error {
        println!("    Error: {}", error);
    }
}
```

## Error Handling

The library provides detailed error information:

```rust
use condition_matcher::MatchError;

match matcher.run(&value) {
    Ok(true) => println!("Matched!"),
    Ok(false) => println!("No match"),
    Err(MatchError::FieldNotFound { field, type_name }) => {
        println!("Field '{}' not found on type '{}'", field, type_name);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Optional Features

### Serde Support

Enable serialization/deserialization of operators and modes:

```toml
condition-matcher = { version = "0.1.0", features = ["serde"] }
```

### Regex Support

Enable regex pattern matching:

```toml
condition-matcher = { version = "0.1.0", features = ["regex"] }
```

```rust
Condition {
    selector: ConditionSelector::FieldValue("email", &r"^[a-z]+@[a-z]+\.[a-z]+$"),
    operator: ConditionOperator::Regex,
}
```

### All Features

```toml
condition-matcher = { version = "0.1.0", features = ["full"] }
```

## Custom Types

To make your custom type matchable, simply derive `Matchable`:

```rust
#[derive(MatchableDerive, PartialEq)]
struct MyStruct {
    field1: i32,
    field2: String,
    optional_field: Option<String>,
}
```

The derive macro automatically:
- Implements field access for all named fields
- Handles `Option<T>` fields by unwrapping when present
- Returns `None` for missing optional fields

## Examples

Run the examples to see the library in action:

```bash
cargo run --example basic_usage
cargo run --example advanced_filtering
```

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
