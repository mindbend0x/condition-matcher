//! # Matcher Module (Legacy)
//!
//! This module provides backwards compatibility with the old Matcher struct.
//! For new code, use [`RuleMatcher`](crate::matchers::RuleMatcher) instead.
//!
//! ## Example
//!
//! ```rust
//! use condition_matcher::{RuleMatcher, MatcherMode, Condition, ConditionSelector, ConditionOperator, Matchable, MatchableDerive, Matcher};
//!
//! #[derive(MatchableDerive, PartialEq, Debug)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! let user = User { name: "Alice".to_string(), age: 30 };
//!
//! let mut matcher = RuleMatcher::new(MatcherMode::AND);
//! matcher.add_condition(Condition {
//!     selector: ConditionSelector::FieldValue("age", &30u32),
//!     operator: ConditionOperator::Equals,
//! });
//!
//! assert!(matcher.matches(&user));
//! ```

use std::{any::Any, fmt};

use crate::{
    MatchError, Matchable,
    condition::{Condition, ConditionMode, ConditionOperator, ConditionSelector, NestedCondition},
    result::{ConditionResult, MatchResult},
};

/// The legacy matcher struct (kept for backwards compatibility).
///
/// For new code, use [`RuleMatcher`](crate::matchers::RuleMatcher) instead.
///
/// ## Example
///
/// ```rust,ignore
/// use condition_matcher::{Matcher, MatcherMode, Condition, ConditionSelector, ConditionOperator};
///
/// let mut matcher: Matcher<&str> = Matcher::new(MatcherMode::AND);
/// matcher
///     .add_condition(Condition {
///         selector: ConditionSelector::Length(5),
///         operator: ConditionOperator::GreaterThanOrEqual,
///     })
///     .add_condition(Condition {
///         selector: ConditionSelector::Value("test"),
///         operator: ConditionOperator::NotEquals,
///     });
///
/// assert!(matcher.run(&"hello").unwrap());
/// ```
#[derive(Debug)]
pub struct Matcher<'a, T: Matchable> {
    pub mode: ConditionMode,
    pub conditions: Vec<Condition<'a, T>>,
}

impl<'a, T: Matchable + 'static> Matcher<'a, T> {
    /// Create a new matcher with the specified mode
    pub fn new(mode: ConditionMode) -> Self {
        Self {
            mode,
            conditions: Vec::new(),
        }
    }

    /// Create a new matcher with AND mode
    pub fn and() -> Self {
        Self::new(ConditionMode::AND)
    }

    /// Create a new matcher with OR mode
    pub fn or() -> Self {
        Self::new(ConditionMode::OR)
    }

    /// Create a new matcher with XOR mode
    pub fn xor() -> Self {
        Self::new(ConditionMode::XOR)
    }

    /// Add a condition to this matcher
    pub fn add_condition(&mut self, condition: Condition<'a, T>) -> &mut Self {
        self.conditions.push(condition);
        self
    }

    /// Add multiple conditions at once
    pub fn add_conditions(
        &mut self,
        conditions: impl IntoIterator<Item = Condition<'a, T>>,
    ) -> &mut Self {
        self.conditions.extend(conditions);
        self
    }

    /// Run the matcher and return a simple boolean result.
    /// Returns Err if any condition evaluation fails critically.
    pub fn run(&self, value: &T) -> Result<bool, MatchError> {
        let result = self.run_detailed(value)?;
        Ok(result.matched)
    }

    /// Run the matcher and return a simple boolean result.
    /// Returns Err if any condition evaluation fails critically.
    pub fn run_batch(&self, values: impl Iterator<Item = &'a T>) -> Result<Vec<bool>, MatchError> {
        values.into_iter().map(|value| self.run(value)).collect()
    }

    /// Run the matcher and return detailed results for each condition
    pub fn run_detailed(&self, value: &T) -> Result<MatchResult, MatchError> {
        let mut condition_results = Vec::new();

        for condition in self.conditions.iter() {
            let result = self.evaluate_condition(condition, value);
            condition_results.push(result);
        }

        let matched = match self.mode {
            ConditionMode::AND => condition_results.iter().all(|r| r.passed),
            ConditionMode::OR => condition_results.iter().any(|r| r.passed),
            ConditionMode::XOR => condition_results.iter().filter(|r| r.passed).count() == 1,
        };

        Ok(MatchResult {
            matched,
            condition_results,
            mode: self.mode,
        })
    }

    /// Run the matcher and return detailed results for each condition
    pub fn run_detailed_batch(
        &self,
        values: impl Iterator<Item = &'a T>,
    ) -> Result<Vec<MatchResult>, MatchError> {
        values
            .into_iter()
            .map(|value| self.run_detailed(value))
            .collect()
    }

    fn evaluate_condition(&self, condition: &Condition<'a, T>, value: &T) -> ConditionResult {
        match &condition.selector {
            ConditionSelector::Length(expected_length) => {
                self.eval_length(value, *expected_length, &condition.operator)
            }
            ConditionSelector::Type(type_name) => {
                self.eval_type(value, type_name, &condition.operator)
            }
            ConditionSelector::Value(value_to_check) => {
                self.eval_value(value, value_to_check, &condition.operator)
            }
            ConditionSelector::FieldValue(field_name, expected_value) => {
                self.eval_field_value(value, field_name, *expected_value, &condition.operator)
            }
            ConditionSelector::FieldPath(path, expected_value) => {
                self.eval_field_path(value, path, *expected_value, &condition.operator)
            }
            ConditionSelector::Not(inner_condition) => {
                let mut result = self.evaluate_condition(inner_condition, value);
                result.passed = !result.passed;
                result.description = format!("NOT({})", result.description);
                result
            }
            ConditionSelector::Nested(nested_group) => self.eval_nested(value, nested_group),
        }
    }

    fn eval_nested(&self, value: &T, group: &NestedCondition<'a, T>) -> ConditionResult {
        let mut results = Vec::new();

        // Evaluate all rules at this level
        for condition in &group.rules {
            results.push(self.evaluate_condition(condition, value));
        }

        // Evaluate nested groups recursively
        for nested_group in &group.nested {
            results.push(self.eval_nested(value, nested_group));
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

    /// Evaluate a NestedCondition group against a value
    pub fn evaluate_nested(&self, value: &T, group: &NestedCondition<'a, T>) -> ConditionResult {
        self.eval_nested(value, group)
    }

    fn eval_length(
        &self,
        value: &T,
        expected: usize,
        operator: &ConditionOperator,
    ) -> ConditionResult {
        match value.get_length() {
            Some(actual) => {
                let passed = compare_numeric(actual, expected, operator);
                ConditionResult {
                    passed,
                    description: format!("length {:?} {}", operator, expected),
                    actual_value: Some(actual.to_string()),
                    expected_value: Some(expected.to_string()),
                    error: None,
                }
            }
            None => ConditionResult {
                passed: false,
                description: format!("length {:?} {}", operator, expected),
                actual_value: None,
                expected_value: Some(expected.to_string()),
                error: Some(MatchError::LengthNotSupported {
                    type_name: value.type_name().to_string(),
                }),
            },
        }
    }

    fn eval_type(
        &self,
        value: &T,
        expected_type: &str,
        operator: &ConditionOperator,
    ) -> ConditionResult {
        let actual_type = value.type_name();
        let passed = match operator {
            ConditionOperator::Equals => actual_type == expected_type,
            ConditionOperator::NotEquals => actual_type != expected_type,
            ConditionOperator::Contains => actual_type.contains(expected_type),
            _ => false,
        };

        ConditionResult {
            passed,
            description: format!("type {:?} {}", operator, expected_type),
            actual_value: Some(actual_type.to_string()),
            expected_value: Some(expected_type.to_string()),
            error: None,
        }
    }

    fn eval_value(&self, value: &T, expected: &T, operator: &ConditionOperator) -> ConditionResult {
        let passed = match operator {
            ConditionOperator::Equals => value == expected,
            ConditionOperator::NotEquals => value != expected,
            _ => false,
        };

        ConditionResult {
            passed,
            description: format!("value {:?}", operator),
            actual_value: None,
            expected_value: None,
            error: None,
        }
    }

    fn eval_field_value(
        &self,
        value: &T,
        field: &str,
        expected: &dyn Any,
        operator: &ConditionOperator,
    ) -> ConditionResult {
        match value.get_field(field) {
            Some(actual) => {
                let (passed, actual_str, expected_str) =
                    compare_any_values(actual, expected, operator);
                ConditionResult {
                    passed,
                    description: format!("field '{}' {:?}", field, operator),
                    actual_value: actual_str,
                    expected_value: expected_str,
                    error: None,
                }
            }
            None => ConditionResult {
                passed: false,
                description: format!("field '{}' {:?}", field, operator),
                actual_value: None,
                expected_value: None,
                error: Some(MatchError::FieldNotFound {
                    field: field.to_string(),
                    type_name: value.type_name().to_string(),
                }),
            },
        }
    }

    fn eval_field_path(
        &self,
        value: &T,
        path: &[&str],
        expected: &dyn Any,
        operator: &ConditionOperator,
    ) -> ConditionResult {
        if path.is_empty() {
            return ConditionResult {
                passed: false,
                description: "field path".to_string(),
                actual_value: None,
                expected_value: None,
                error: Some(MatchError::EmptyFieldPath),
            };
        }

        // Try to use get_field_path first
        if let Some(actual) = value.get_field_path(path) {
            let (passed, actual_str, expected_str) = compare_any_values(actual, expected, operator);
            return ConditionResult {
                passed,
                description: format!("field path '{:?}' {:?}", path, operator),
                actual_value: actual_str,
                expected_value: expected_str,
                error: None,
            };
        }

        // Fallback: try first field only (basic implementation)
        match value.get_field(path[0]) {
            Some(actual) if path.len() == 1 => {
                let (passed, actual_str, expected_str) =
                    compare_any_values(actual, expected, operator);
                ConditionResult {
                    passed,
                    description: format!("field path '{:?}' {:?}", path, operator),
                    actual_value: actual_str,
                    expected_value: expected_str,
                    error: None,
                }
            }
            _ => ConditionResult {
                passed: false,
                description: format!("field path '{:?}' {:?}", path, operator),
                actual_value: None,
                expected_value: None,
                error: Some(MatchError::NestedFieldNotFound {
                    path: path.iter().map(|s| s.to_string()).collect(),
                    failed_at: path[0].to_string(),
                }),
            },
        }
    }
}

// ============================================================================
// Comparison Functions
// ============================================================================

fn compare_numeric<N: PartialOrd>(actual: N, expected: N, operator: &ConditionOperator) -> bool {
    match operator {
        ConditionOperator::Equals => actual == expected,
        ConditionOperator::NotEquals => actual != expected,
        ConditionOperator::GreaterThan => actual > expected,
        ConditionOperator::LessThan => actual < expected,
        ConditionOperator::GreaterThanOrEqual => actual >= expected,
        ConditionOperator::LessThanOrEqual => actual <= expected,
        _ => false,
    }
}

fn compare_any_values(
    actual: &dyn Any,
    expected: &dyn Any,
    operator: &ConditionOperator,
) -> (bool, Option<String>, Option<String>) {
    // Integer types
    if let Some(result) = try_compare::<i8>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<i16>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<i32>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<i64>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<i128>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<isize>(actual, expected, operator) {
        return result;
    }

    // Unsigned integer types
    if let Some(result) = try_compare::<u8>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<u16>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<u32>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<u64>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<u128>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<usize>(actual, expected, operator) {
        return result;
    }

    // Float types
    if let Some(result) = try_compare::<f32>(actual, expected, operator) {
        return result;
    }
    if let Some(result) = try_compare::<f64>(actual, expected, operator) {
        return result;
    }

    // Boolean
    if let Some(result) = try_compare::<bool>(actual, expected, operator) {
        return result;
    }

    // String types with string operations
    if let Some(result) = try_compare_strings(actual, expected, operator) {
        return result;
    }

    // Char
    if let Some(result) = try_compare::<char>(actual, expected, operator) {
        return result;
    }

    // No match found
    (false, None, None)
}

fn try_compare<T: PartialOrd + PartialEq + fmt::Display + 'static>(
    actual: &dyn Any,
    expected: &dyn Any,
    operator: &ConditionOperator,
) -> Option<(bool, Option<String>, Option<String>)> {
    if let (Some(a), Some(e)) = (actual.downcast_ref::<T>(), expected.downcast_ref::<T>()) {
        let passed = match operator {
            ConditionOperator::Equals => a == e,
            ConditionOperator::NotEquals => a != e,
            ConditionOperator::GreaterThan => a > e,
            ConditionOperator::LessThan => a < e,
            ConditionOperator::GreaterThanOrEqual => a >= e,
            ConditionOperator::LessThanOrEqual => a <= e,
            _ => return None,
        };
        Some((passed, Some(a.to_string()), Some(e.to_string())))
    } else {
        None
    }
}

fn try_compare_strings(
    actual: &dyn Any,
    expected: &dyn Any,
    operator: &ConditionOperator,
) -> Option<(bool, Option<String>, Option<String>)> {
    // Get the actual string
    let actual_str: Option<&str> = actual
        .downcast_ref::<String>()
        .map(|s| s.as_str())
        .or_else(|| actual.downcast_ref::<&str>().copied());

    // Get the expected string
    let expected_str: Option<&str> = expected
        .downcast_ref::<String>()
        .map(|s| s.as_str())
        .or_else(|| expected.downcast_ref::<&str>().copied());

    match (actual_str, expected_str) {
        (Some(a), Some(e)) => {
            let passed = match operator {
                ConditionOperator::Equals => a == e,
                ConditionOperator::NotEquals => a != e,
                ConditionOperator::Contains => a.contains(e),
                ConditionOperator::NotContains => !a.contains(e),
                ConditionOperator::StartsWith => a.starts_with(e),
                ConditionOperator::EndsWith => a.ends_with(e),
                ConditionOperator::GreaterThan => a > e,
                ConditionOperator::LessThan => a < e,
                ConditionOperator::GreaterThanOrEqual => a >= e,
                ConditionOperator::LessThanOrEqual => a <= e,
                ConditionOperator::IsEmpty => a.is_empty(),
                ConditionOperator::IsNotEmpty => !a.is_empty(),
                #[cfg(feature = "regex")]
                ConditionOperator::Regex => regex::Regex::new(e)
                    .map(|re| re.is_match(a))
                    .unwrap_or(false),
                #[cfg(not(feature = "regex"))]
                ConditionOperator::Regex => false,
                _ => return None,
            };
            Some((passed, Some(a.to_string()), Some(e.to_string())))
        }
        _ => None,
    }
}

// ============================================================================
// JSON Value Comparison (when json_condition feature is enabled)
// ============================================================================

#[cfg(feature = "json_condition")]
fn extract_as_f64(actual: &dyn Any) -> Option<f64> {
    // Try various numeric types
    if let Some(v) = actual.downcast_ref::<f64>() {
        return Some(*v);
    }
    if let Some(v) = actual.downcast_ref::<f32>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<i64>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<i32>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<i16>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<i8>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<u64>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<u32>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<u16>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<u8>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<isize>() {
        return Some(*v as f64);
    }
    if let Some(v) = actual.downcast_ref::<usize>() {
        return Some(*v as f64);
    }
    None
}

#[cfg(feature = "json_condition")]
fn compare_json_to_any(
    actual: &dyn Any,
    expected: &serde_json::Value,
    operator: &ConditionOperator,
) -> (bool, Option<String>, Option<String>) {
    // Numeric comparison
    if let Some(exp_f64) = expected.as_f64() {
        if let Some(act_f64) = extract_as_f64(actual) {
            let passed = match operator {
                ConditionOperator::Equals => (act_f64 - exp_f64).abs() < f64::EPSILON,
                ConditionOperator::NotEquals => (act_f64 - exp_f64).abs() >= f64::EPSILON,
                ConditionOperator::GreaterThan => act_f64 > exp_f64,
                ConditionOperator::LessThan => act_f64 < exp_f64,
                ConditionOperator::GreaterThanOrEqual => act_f64 >= exp_f64,
                ConditionOperator::LessThanOrEqual => act_f64 <= exp_f64,
                _ => false,
            };
            return (passed, Some(act_f64.to_string()), Some(exp_f64.to_string()));
        }
    }

    // String comparison
    if let Some(exp_str) = expected.as_str() {
        let act_str: Option<&str> = actual
            .downcast_ref::<String>()
            .map(|s| s.as_str())
            .or_else(|| actual.downcast_ref::<&str>().copied());

        if let Some(a) = act_str {
            let passed = match operator {
                ConditionOperator::Equals => a == exp_str,
                ConditionOperator::NotEquals => a != exp_str,
                ConditionOperator::Contains => a.contains(exp_str),
                ConditionOperator::NotContains => !a.contains(exp_str),
                ConditionOperator::StartsWith => a.starts_with(exp_str),
                ConditionOperator::EndsWith => a.ends_with(exp_str),
                ConditionOperator::GreaterThan => a > exp_str,
                ConditionOperator::LessThan => a < exp_str,
                ConditionOperator::GreaterThanOrEqual => a >= exp_str,
                ConditionOperator::LessThanOrEqual => a <= exp_str,
                ConditionOperator::IsEmpty => a.is_empty(),
                ConditionOperator::IsNotEmpty => !a.is_empty(),
                #[cfg(feature = "regex")]
                ConditionOperator::Regex => regex::Regex::new(exp_str)
                    .map(|re| re.is_match(a))
                    .unwrap_or(false),
                #[cfg(not(feature = "regex"))]
                ConditionOperator::Regex => false,
                _ => false,
            };
            return (passed, Some(a.to_string()), Some(exp_str.to_string()));
        }
    }

    // Boolean comparison
    if let Some(exp_bool) = expected.as_bool() {
        if let Some(act_bool) = actual.downcast_ref::<bool>() {
            let passed = match operator {
                ConditionOperator::Equals => *act_bool == exp_bool,
                ConditionOperator::NotEquals => *act_bool != exp_bool,
                _ => false,
            };
            return (
                passed,
                Some(act_bool.to_string()),
                Some(exp_bool.to_string()),
            );
        }
    }

    (false, None, None)
}

// ============================================================================
// JSON Condition Evaluation (when json_condition feature is enabled)
// ============================================================================

#[cfg(feature = "json_condition")]
use crate::condition::{JsonCondition, JsonNestedCondition};
#[cfg(feature = "json_condition")]
use crate::result::{JsonConditionResult, JsonEvalResult};

/// Evaluate a JsonNestedCondition against a Matchable context.
///
/// This function allows you to deserialize conditions from JSON and evaluate
/// them against any type that implements Matchable.
///
/// # Example
///
/// ```ignore
/// use condition_matcher::{evaluate_json_condition, JsonNestedCondition, Matchable};
///
/// let json = r#"{"logic": "AND", "rules": [{"field": "age", "operator": "gte", "value": 18}]}"#;
/// let group: JsonNestedCondition = serde_json::from_str(json)?;
/// let result = evaluate_json_condition(&my_struct, &group);
/// ```
#[cfg(feature = "json_condition")]
pub fn evaluate_json_condition<M: Matchable>(
    context: &M,
    group: &JsonNestedCondition,
) -> JsonEvalResult {
    let mut details = Vec::new();
    eval_json_nested_recursive(context, group, &mut details)
}

#[cfg(feature = "json_condition")]
fn eval_json_nested_recursive<M: Matchable>(
    context: &M,
    group: &JsonNestedCondition,
    details: &mut Vec<JsonConditionResult>,
) -> JsonEvalResult {
    let mut flags = Vec::new();

    // Evaluate all rules at this level
    for rule in &group.rules {
        let result = eval_json_rule(context, rule);
        flags.push(result.passed);
        details.push(result);
    }

    // Evaluate nested groups recursively
    for nested_group in &group.nested {
        let nested_result = eval_json_nested_recursive(context, nested_group, details);
        flags.push(nested_result.matched);
    }

    let matched = match group.mode {
        ConditionMode::AND => flags.iter().all(|&f| f),
        ConditionMode::OR => flags.iter().any(|&f| f),
        ConditionMode::XOR => flags.iter().filter(|&&f| f).count() == 1,
    };

    JsonEvalResult {
        matched,
        details: details.clone(),
    }
}

#[cfg(feature = "json_condition")]
fn eval_json_rule<M: Matchable>(context: &M, rule: &JsonCondition) -> JsonConditionResult {
    let field = &rule.field;

    // Support dotted paths like "user.age" by splitting on '.'
    let path_segments: Vec<&str> = field.split('.').collect();

    // Try to resolve the field value
    let actual_value = if path_segments.len() == 1 {
        context.get_field(field)
    } else {
        context.get_field_path(&path_segments)
    };

    match actual_value {
        Some(actual) => {
            let (passed, actual_str, _expected_str) =
                compare_json_to_any(actual, &rule.value, &rule.operator);
            JsonConditionResult {
                passed,
                field: field.clone(),
                operator: rule.operator,
                expected: rule.value.clone(),
                actual: actual_str
                    .clone()
                    .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
                    .or_else(|| {
                        actual_str.and_then(|s| s.parse::<f64>().ok().map(serde_json::Value::from))
                    }),
                error: None,
            }
        }
        None => JsonConditionResult {
            passed: false,
            field: field.clone(),
            operator: rule.operator,
            expected: rule.value.clone(),
            actual: None,
            error: Some(format!("Field '{}' not found", field)),
        },
    }
}
