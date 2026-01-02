# Condition Matcher

**Why?** Rust's `match` statement is arguably one of the most powerful and flexible features of the language. It allows you to match against a value and execute different code blocks based on the value. However, if you want to match against user-defined rules, you'd need to write a lot of boilerplate code.

**What?** This library provides a flexible and type-safe condition matching library for Rust with automatic struct field access. It allows you to create matchers that can be used to match against values and execute different code blocks based on the value.

## Features

- **Automatic struct matching** with derive macro
- Multiple matching modes with support for nested conditions
- Support for various condition types (value, length, type, field)
- **Numeric comparisons** on fields (>, <, >=, <=)
- **String operations** (contains, starts_with, ends_with)
- **Regex matching** (optional feature)
- **Optional field handling** (Option<T> support)
- **Detailed match results** with error information
- **Builder pattern** for ergonomic API
- **Serde support** (optional feature)
- **Parallel processing** (optional feature)
- Zero-cost abstractions with compile-time type safety


## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
condition-matcher = "0.2.0"

# Optional features
condition-matcher = { version = "0.2.0", features = ["serde", "regex", "json_condition", "parallel"] }
# Or all features
condition-matcher = { version = "0.2.0", features = ["full"] }
```

## Quick Start

There are two main concepts in this library:
- `Matcher`: A matcher is a collection of conditions that are evaluated against some data.
- `Matchable`: A matchable is a type (data) that conditions can be evaluated against.

The `Matcher` trait is implemented by `RuleMatcher` and `JsonMatcher`. The primary difference is that a `JsonMatcher` is constructed from a JSON input value.

A `Matcher` can be created in a few different ways:

- By using the `MatcherBuilder` to construct the ruleset in Rust code. This creates a `RuleMatcher`.
- By using a JSON string or `serde_json::Value` to construct a `JsonMatcher`.

A `Matchable` can be a basic type, like a string or number, or a complex type, like a struct. To make a complex type matchable, you can derive the `Matchable` trait.

```rust
#[derive(MatchableDerive, PartialEq, Debug)]
struct Product {
    id: i32,
    name: String,
    price: f64,
    in_stock: bool,
    quantity: u32,
}
```

It is also possible to implement the `Matchable` trait for your own types and more complex use-cases, like a database cache layer. More details can be found in the examples.


### Basic Usage

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

assert!(matcher.matches(&user));
```


### Builder API

For a more ergonomic experience, use the builder pattern:

```rust
use condition_matcher::{MatcherBuilder, MatcherMode};

let matcher = MatcherBuilder::<&str>::new()
    .mode(MatcherMode::AND)
    .length_gte(4)
    .value_not_equals("bad")
    .build();

assert!(matcher.matches(&"good"));
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

### JSON Condition Usage

Your rules can be stored in a database or a config file. To enable using stored rules, you can create a `JsonMatcher` from a JSON string or `serde_json::Value`.

Here's a quick example of how a JSON value can be used to create a `JsonMatcher`:

```rust
let conditions = r#"{
    "mode": "OR",
    "nested": [
        {
            "mode": "AND",
            "rules": [
                {"field": "price", "operator": "greater_than", "value": 500}
            ]
        },
        {
            "mode": "AND",
            "rules": [
                {"field": "price", "operator": "less_than", "value": 100},
                {"field": "in_stock", "operator": "equals", "value": false}
            ]
        }
    ]
}"#;
let matcher = JsonMatcher::from_json(conditions).unwrap();
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

> Note: More matching modes will be included in future versions. For example `XOR` to match exactly one condition.

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

> Field value matching has a built-in implementation for all basic types within a struct. Complex types need to implement the `get_field` and `get_field_path` methods
from the `Matchable` trait to handle arbitrary field access.

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

### Nested Conditions

Nested conditions are supported by the `NestedCondition` struct.

```rust
let nested = NestedCondition {
    mode: ConditionMode::AND,
    rules: vec![
        Condition {
            selector: ConditionSelector::FieldValue("price", &100.0f64),
            operator: ConditionOperator::GreaterThanOrEqual,
        },
    ],
    nested: vec![
        NestedCondition {
            mode: ConditionMode::OR,
            rules: vec![
                Condition {
                    selector: ConditionSelector::FieldValue("quantity", &10u32),
                    operator: ConditionOperator::LessThanOrEqual,
                },
                Condition {
                    selector: ConditionSelector::FieldValue("is_new", &true),
                    operator: ConditionOperator::GreaterThanOrEqual,
                },
            ],
            nested: vec![],
        },
    ],
};
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
cargo run --example json_condition
cargo run --example parallel_processing
cargo run --example parallel_cache_watchers
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit an issue for a feature request or bug report, or a Pull Request if you'd like to add something.
