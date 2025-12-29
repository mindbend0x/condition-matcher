use std::any::Any;

/// Operators for comparing values in conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
}

/// A single condition to evaluate
#[derive(Debug)]
pub struct Condition<'a, T> {
    pub operator: ConditionOperator,
    pub selector: ConditionSelector<'a, T>,
}
