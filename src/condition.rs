use std::any::Any;

#[cfg(feature = "json_condition")]
use std::borrow::Cow;

/// Mode for combining multiple conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(any(feature = "serde", feature = "json_condition"), derive(serde::Serialize, serde::Deserialize))]
pub enum ConditionMode {
    /// All conditions must match
    #[default]
    AND,
    /// At least one condition must match
    OR,
    /// Exactly one condition must match
    XOR,
}

/// Operators for comparing values in conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(any(feature = "serde", feature = "json_condition"), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(feature = "serde", feature = "json_condition"), serde(rename_all = "snake_case"))]
pub enum ConditionOperator {
    /// Exact equality check
    Equals,
    /// Inequality check
    NotEquals,
    /// Greater than comparison (numeric types)
    GreaterThan,
    /// Less than comparison (numeric types)
    LessThan,
    /// Greater than or equal comparison (numeric types)
    GreaterThanOrEqual,
    /// Less than or equal comparison (numeric types)
    LessThanOrEqual,
    /// String contains substring
    Contains,
    /// String does not contain substring
    NotContains,
    /// String starts with prefix
    StartsWith,
    /// String ends with suffix
    EndsWith,
    /// Value matches regex pattern
    Regex,
    /// Check if value is None/null
    IsNone,
    /// Check if value is Some/present
    IsSome,
    /// Check if collection is empty
    IsEmpty,
    /// Check if collection is not empty
    IsNotEmpty,
}

// ============================================================================
// Core condition types (always available, uses &dyn Any)
// ============================================================================

/// Selectors for targeting what to check in a condition
#[derive(Debug)]
pub enum ConditionSelector<'a, T> {
    /// Check the length of a string or collection
    Length(usize),
    /// Check the type name
    Type(String),
    /// Compare against a specific value
    Value(T),
    /// Check a field value by name
    FieldValue(&'a str, &'a dyn Any),
    /// Check a nested field path (e.g., ["address", "city"])
    FieldPath(&'a [&'a str], &'a dyn Any),
    /// Negate a condition (inverts the result)
    Not(Box<Condition<'a, T>>),
    /// A nested group of conditions
    Nested(Box<NestedCondition<'a, T>>),
}

/// A single condition to evaluate
#[derive(Debug)]
pub struct Condition<'a, T> {
    pub operator: ConditionOperator,
    pub selector: ConditionSelector<'a, T>,
}

/// A group of conditions combined with a logic mode
#[derive(Debug)]
pub struct NestedCondition<'a, T> {
    /// How to combine conditions: AND, OR, XOR
    pub mode: ConditionMode,
    /// Simple conditions at this level
    pub rules: Vec<Condition<'a, T>>,
    /// Child groups (recursive)
    pub nested: Vec<Box<NestedCondition<'a, T>>>,
}

// ============================================================================
// JSON-serializable condition types (only with json_condition feature)
// These are separate types that can be deserialized from JSON and converted
// to the core types for evaluation.
// ============================================================================

/// A JSON-serializable condition for field comparisons.
/// 
/// Deserializes from JSON like:
/// ```json
/// { "field": "price", "operator": "gte", "value": 100.0 }
/// ```
#[cfg(feature = "json_condition")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonCondition<'a> {
    /// The field to check (supports dotted paths like "user.age")
    #[serde(borrow)]
    pub field: Cow<'a, str>,
    /// The comparison operator
    pub operator: ConditionOperator,
    /// The value to compare against
    pub value: serde_json::Value,
}

/// A JSON-serializable group of conditions with nested support.
/// 
/// Deserializes from JSON like:
/// ```json
/// {
///     "logic": "AND",
///     "rules": [
///         { "field": "price", "operator": "gte", "value": 100.0 }
///     ],
///     "nested": [
///         { "logic": "OR", "rules": [...] }
///     ]
/// }
/// ```
#[cfg(feature = "json_condition")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonNestedCondition<'a> {
    /// How to combine conditions: AND, OR, XOR
    #[serde(alias = "logic", alias = "comparator")]
    pub mode: ConditionMode,
    /// Simple conditions at this level
    #[serde(default, borrow, alias = "conditions")]
    pub rules: Vec<JsonCondition<'a>>,
    /// Child groups (recursive)
    #[serde(default, alias = "nested_rules", alias = "nested_conditions")]
    pub nested: Vec<Box<JsonNestedCondition<'a>>>,
}
