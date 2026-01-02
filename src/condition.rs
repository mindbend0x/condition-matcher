use std::any::Any;

use crate::{
    evaluators::{FieldEvaluator, LengthEvaluator, PathEvaluator, TypeEvaluator, ValueEvaluator},
    matchable::Matchable,
    result::ConditionResult,
    traits::Predicate,
};

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
/// { "field": "price", "operator": "greater_than_or_equal", "value": 100.0 }
/// ```
#[cfg(feature = "json_condition")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonCondition {
    /// The field to check (supports dotted paths like "user.age")
    pub field: String,
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
///         { "field": "price", "operator": "greater_than_or_equal", "value": 100.0 }
///     ],
///     "nested": [
///         { "logic": "OR", "rules": [...] }
///     ]
/// }
/// ```
#[cfg(feature = "json_condition")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonNestedCondition {
    /// How to combine conditions: AND, OR, XOR
    #[serde(alias = "logic", alias = "comparator", default)]
    pub mode: ConditionMode,
    /// Simple conditions at this level
    #[serde(default, alias = "conditions")]
    pub rules: Vec<JsonCondition>,
    /// Child groups (recursive)
    #[serde(default, alias = "nested_rules", alias = "nested_conditions")]
    pub nested: Vec<Box<JsonNestedCondition>>,
}

// ============================================================================
// Predicate Implementation for Condition
// ============================================================================

impl<'a, T: Matchable + 'static> Predicate<T> for Condition<'a, T> {
    fn test(&self, value: &T) -> bool {
        self.test_detailed(value).passed
    }

    fn test_detailed(&self, value: &T) -> ConditionResult {
        match &self.selector {
            ConditionSelector::Length(expected) => {
                LengthEvaluator::evaluate(value, *expected, &self.operator)
            }
            ConditionSelector::Type(type_name) => {
                TypeEvaluator::evaluate(value, type_name, &self.operator)
            }
            ConditionSelector::Value(expected) => {
                ValueEvaluator::evaluate(value, expected, &self.operator)
            }
            ConditionSelector::FieldValue(field, expected) => {
                FieldEvaluator::evaluate(value, field, *expected, &self.operator)
            }
            ConditionSelector::FieldPath(path, expected) => {
                PathEvaluator::evaluate(value, path, *expected, &self.operator)
            }
            ConditionSelector::Not(inner) => {
                let mut result = inner.test_detailed(value);
                result.passed = !result.passed;
                result.description = format!("NOT({})", result.description);
                result
            }
            ConditionSelector::Nested(group) => evaluate_nested(value, group),
        }
    }
}

/// Evaluate a nested condition group.
fn evaluate_nested<'a, T: Matchable + 'static>(
    value: &T,
    group: &NestedCondition<'a, T>,
) -> ConditionResult {
    let mut results = Vec::new();

    // Evaluate all rules at this level
    for condition in &group.rules {
        results.push(condition.test_detailed(value));
    }

    // Evaluate nested groups recursively
    for nested_group in &group.nested {
        results.push(evaluate_nested(value, nested_group));
    }

    let passed = match group.mode {
        ConditionMode::AND => results.iter().all(|r| r.passed),
        ConditionMode::OR => results.iter().any(|r| r.passed),
        ConditionMode::XOR => results.iter().filter(|r| r.passed).count() == 1,
    };

    ConditionResult {
        passed,
        description: format!(
            "{:?} group ({} rules, {} nested)",
            group.mode,
            group.rules.len(),
            group.nested.len()
        ),
        actual_value: None,
        expected_value: None,
        error: None,
    }
}
