//! # Matcher Module
//! 
//! This module provides the core matching functionality for evaluating conditions
//! against values.
//! 
//! ## Example
//! 
//! ```rust
//! use condition_matcher::{Matcher, MatcherMode, Condition, ConditionSelector, ConditionOperator, Matchable, MatchableDerive};
//! 
//! #[derive(MatchableDerive, PartialEq, Debug)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//! 
//! let user = User { name: "Alice".to_string(), age: 30 };
//! 
//! let mut matcher = Matcher::new(MatcherMode::AND);
//! matcher.add_condition(Condition {
//!     selector: ConditionSelector::FieldValue("age", &30u32),
//!     operator: ConditionOperator::Equals,
//! });
//! 
//! assert!(matcher.run(&user).unwrap());
//! ```

use std::{any::Any, collections::HashMap, fmt};

use crate::condition::{Condition, ConditionOperator, ConditionSelector};

// Re-export the derive macro
pub use condition_matcher_derive::Matchable as MatchableDerive;

/// Errors that can occur during condition matching
#[derive(Debug, Clone, PartialEq)]
pub enum MatchError {
    /// The specified field was not found on the type
    FieldNotFound {
        field: String,
        type_name: String,
    },
    /// Type mismatch between expected and actual values
    TypeMismatch {
        field: String,
        expected: String,
        actual: String,
    },
    /// The operator is not supported for this type/context
    UnsupportedOperator {
        operator: String,
        context: String,
    },
    /// Length check is not supported for this type
    LengthNotSupported {
        type_name: String,
    },
    /// Regex compilation failed
    #[cfg(feature = "regex")]
    RegexError {
        pattern: String,
        message: String,
    },
    /// The field path is empty
    EmptyFieldPath,
    /// Nested field not found
    NestedFieldNotFound {
        path: Vec<String>,
        failed_at: String,
    },
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatchError::FieldNotFound { field, type_name } => {
                write!(f, "Field '{}' not found on type '{}'", field, type_name)
            }
            MatchError::TypeMismatch { field, expected, actual } => {
                write!(f, "Type mismatch for field '{}': expected '{}', got '{}'", field, expected, actual)
            }
            MatchError::UnsupportedOperator { operator, context } => {
                write!(f, "Operator '{}' not supported for {}", operator, context)
            }
            MatchError::LengthNotSupported { type_name } => {
                write!(f, "Length check not supported for type '{}'", type_name)
            }
            #[cfg(feature = "regex")]
            MatchError::RegexError { pattern, message } => {
                write!(f, "Invalid regex pattern '{}': {}", pattern, message)
            }
            MatchError::EmptyFieldPath => {
                write!(f, "Field path cannot be empty")
            }
            MatchError::NestedFieldNotFound { path, failed_at } => {
                write!(f, "Nested field not found at '{}' in path {:?}", failed_at, path)
            }
        }
    }
}

impl std::error::Error for MatchError {}

/// Result of a match operation with detailed information
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Whether the overall match succeeded
    pub matched: bool,
    /// Individual results for each condition
    pub condition_results: Vec<ConditionResult>,
    /// The matching mode used
    pub mode: MatcherMode,
}

impl MatchResult {
    /// Returns true if the match succeeded
    pub fn is_match(&self) -> bool {
        self.matched
    }
    
    /// Returns the conditions that passed
    pub fn passed_conditions(&self) -> Vec<&ConditionResult> {
        self.condition_results.iter().filter(|r| r.passed).collect()
    }
    
    /// Returns the conditions that failed
    pub fn failed_conditions(&self) -> Vec<&ConditionResult> {
        self.condition_results.iter().filter(|r| !r.passed).collect()
    }
}

/// Result of evaluating a single condition
#[derive(Debug, Clone)]
pub struct ConditionResult {
    /// Whether this condition passed
    pub passed: bool,
    /// Description of what was checked
    pub description: String,
    /// The actual value that was compared (as string for display)
    pub actual_value: Option<String>,
    /// The expected value (as string for display)
    pub expected_value: Option<String>,
    /// Error if evaluation failed
    pub error: Option<MatchError>,
}

/// Mode for combining multiple conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MatcherMode {
    /// All conditions must match
    #[default]
    AND,
    /// At least one condition must match
    OR,
    /// Exactly one condition must match
    XOR,
}

/// Trait for types that can be matched against conditions.
/// 
/// This trait allows different types to opt-in to specific matching capabilities.
/// For structs, you can use `#[derive(Matchable)]` to automatically implement field access.
/// 
/// ## Example
/// 
/// ```rust
/// use condition_matcher::{Matchable, MatchableDerive};
/// use std::any::Any;
/// 
/// #[derive(MatchableDerive, PartialEq)]
/// struct MyStruct {
///     value: i32,
///     name: String,
/// }
/// 
/// // The derive macro automatically implements get_field for all fields
/// ```
pub trait Matchable: PartialEq + Sized {
    /// Get the length of the value if supported (for strings, collections, etc.)
    fn get_length(&self) -> Option<usize> {
        None
    }
    
    /// Get a field value by name as a type-erased reference.
    /// Returns None if field access is not supported or field doesn't exist.
    fn get_field(&self, _field: &str) -> Option<&dyn Any> {
        None
    }
    
    /// Get a nested field value by path.
    /// Default implementation walks through get_field calls.
    fn get_field_path(&self, _path: &[&str]) -> Option<&dyn Any> {
        None
    }
    
    /// Get the type name as a string
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
    
    /// Check if the value is considered "empty" (for collections, strings, options)
    fn is_empty(&self) -> Option<bool> {
        self.get_length().map(|len| len == 0)
    }
    
    /// Check if this is a None/null value (for Option types)
    fn is_none(&self) -> bool {
        false
    }
}

/// The main matcher struct that holds conditions and evaluates them against values.
/// 
/// ## Example
/// 
/// ```rust
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
    pub mode: MatcherMode,
    pub conditions: Vec<Condition<'a, T>>,
}

impl<'a, T: Matchable + 'static> Matcher<'a, T> {
    /// Create a new matcher with the specified mode
    pub fn new(mode: MatcherMode) -> Self {
        Self {
            mode,
            conditions: Vec::new(),
        }
    }
    
    /// Create a new matcher with AND mode
    pub fn and() -> Self {
        Self::new(MatcherMode::AND)
    }
    
    /// Create a new matcher with OR mode
    pub fn or() -> Self {
        Self::new(MatcherMode::OR)
    }
    
    /// Create a new matcher with XOR mode
    pub fn xor() -> Self {
        Self::new(MatcherMode::XOR)
    }

    /// Add a condition to this matcher
    pub fn add_condition(&mut self, condition: Condition<'a, T>) -> &mut Self {
        self.conditions.push(condition);
        self
    }
    
    /// Add multiple conditions at once
    pub fn add_conditions(&mut self, conditions: impl IntoIterator<Item = Condition<'a, T>>) -> &mut Self {
        self.conditions.extend(conditions);
        self
    }

    /// Run the matcher and return a simple boolean result.
    /// Returns Err if any condition evaluation fails critically.
    pub fn run(&self, value: &T) -> Result<bool, MatchError> {
        let result = self.run_detailed(value)?;
        Ok(result.matched)
    }
    
    /// Run the matcher and return detailed results for each condition
    pub fn run_detailed(&self, value: &T) -> Result<MatchResult, MatchError> {
        let mut condition_results = Vec::new();
        
        for condition in self.conditions.iter() {
            let result = self.evaluate_condition(condition, value);
            condition_results.push(result);
        }

        let matched = match self.mode {
            MatcherMode::AND => condition_results.iter().all(|r| r.passed),
            MatcherMode::OR => condition_results.iter().any(|r| r.passed),
            MatcherMode::XOR => condition_results.iter().filter(|r| r.passed).count() == 1,
        };
        
        Ok(MatchResult {
            matched,
            condition_results,
            mode: self.mode,
        })
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
        }
    }
    
    fn eval_length(&self, value: &T, expected: usize, operator: &ConditionOperator) -> ConditionResult {
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
    
    fn eval_type(&self, value: &T, expected_type: &str, operator: &ConditionOperator) -> ConditionResult {
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
                let (passed, actual_str, expected_str) = compare_any_values(actual, expected, operator);
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
                let (passed, actual_str, expected_str) = compare_any_values(actual, expected, operator);
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
    if let Some(result) = try_compare::<i8>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<i16>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<i32>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<i64>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<i128>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<isize>(actual, expected, operator) { return result; }
    
    // Unsigned integer types
    if let Some(result) = try_compare::<u8>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<u16>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<u32>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<u64>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<u128>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<usize>(actual, expected, operator) { return result; }
    
    // Float types
    if let Some(result) = try_compare::<f32>(actual, expected, operator) { return result; }
    if let Some(result) = try_compare::<f64>(actual, expected, operator) { return result; }
    
    // Boolean
    if let Some(result) = try_compare::<bool>(actual, expected, operator) { return result; }
    
    // String types with string operations
    if let Some(result) = try_compare_strings(actual, expected, operator) { return result; }
    
    // Char
    if let Some(result) = try_compare::<char>(actual, expected, operator) { return result; }
    
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
    let actual_str: Option<&str> = actual.downcast_ref::<String>().map(|s| s.as_str())
        .or_else(|| actual.downcast_ref::<&str>().copied());
    
    // Get the expected string
    let expected_str: Option<&str> = expected.downcast_ref::<String>().map(|s| s.as_str())
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
                ConditionOperator::Regex => {
                    regex::Regex::new(e).map(|re| re.is_match(a)).unwrap_or(false)
                }
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
// Matchable Implementations for Common Types
// ============================================================================

impl Matchable for &str {
    fn get_length(&self) -> Option<usize> {
        Some(self.len())
    }
    
    fn is_empty(&self) -> Option<bool> {
        Some((*self).is_empty())
    }
}

impl Matchable for String {
    fn get_length(&self) -> Option<usize> {
        Some(self.len())
    }
    
    fn is_empty(&self) -> Option<bool> {
        Some(self.is_empty())
    }
}

impl<T: Matchable> Matchable for Vec<T> {
    fn get_length(&self) -> Option<usize> {
        Some(self.len())
    }
    
    fn is_empty(&self) -> Option<bool> {
        Some(self.is_empty())
    }
}

impl<K, V> Matchable for HashMap<K, V>
where
    K: std::borrow::Borrow<str> + std::hash::Hash + Eq,
    V: PartialEq + 'static,
{
    fn get_length(&self) -> Option<usize> {
        Some(self.len())
    }
    
    fn get_field(&self, field: &str) -> Option<&dyn Any> {
        self.get(field).map(|v| v as &dyn Any)
    }
    
    fn is_empty(&self) -> Option<bool> {
        Some(self.is_empty())
    }
}

impl<T: Matchable + 'static> Matchable for Option<T> {
    fn get_length(&self) -> Option<usize> {
        self.as_ref().and_then(|v| v.get_length())
    }
    
    fn get_field(&self, field: &str) -> Option<&dyn Any> {
        self.as_ref().and_then(|v| v.get_field(field))
    }
    
    fn is_none(&self) -> bool {
        self.is_none()
    }
    
    fn is_empty(&self) -> Option<bool> {
        Some(self.is_none())
    }
}

// Implement for primitive types
macro_rules! impl_matchable_primitive {
    ($($t:ty),*) => {
        $(
            impl Matchable for $t {}
        )*
    };
}

impl_matchable_primitive!(
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
    bool, char
);

// ============================================================================
// Builder API
// ============================================================================

/// A builder for creating matchers with a fluent API
/// 
/// ## Example
/// 
/// ```rust
/// use condition_matcher::{MatcherBuilder, MatcherMode, ConditionOperator};
/// 
/// let matcher = MatcherBuilder::<i32>::new()
///     .mode(MatcherMode::AND)
///     .value_equals(42)
///     .build();
/// 
/// assert!(matcher.run(&42).unwrap());
/// ```
pub struct MatcherBuilder<'a, T: Matchable> {
    mode: MatcherMode,
    conditions: Vec<Condition<'a, T>>,
}

impl<'a, T: Matchable + 'static> MatcherBuilder<'a, T> {
    /// Create a new builder with default AND mode
    pub fn new() -> Self {
        Self {
            mode: MatcherMode::AND,
            conditions: Vec::new(),
        }
    }
    
    /// Set the matching mode
    pub fn mode(mut self, mode: MatcherMode) -> Self {
        self.mode = mode;
        self
    }
    
    /// Add a condition that the value equals the expected value
    pub fn value_equals(mut self, expected: T) -> Self {
        self.conditions.push(Condition {
            selector: ConditionSelector::Value(expected),
            operator: ConditionOperator::Equals,
        });
        self
    }
    
    /// Add a condition that the value does not equal the expected value
    pub fn value_not_equals(mut self, expected: T) -> Self {
        self.conditions.push(Condition {
            selector: ConditionSelector::Value(expected),
            operator: ConditionOperator::NotEquals,
        });
        self
    }
    
    /// Add a length condition
    pub fn length(mut self, len: usize, operator: ConditionOperator) -> Self {
        self.conditions.push(Condition {
            selector: ConditionSelector::Length(len),
            operator,
        });
        self
    }
    
    /// Add a condition that length equals the expected value
    pub fn length_equals(self, len: usize) -> Self {
        self.length(len, ConditionOperator::Equals)
    }
    
    /// Add a condition that length is greater than or equal to the expected value
    pub fn length_gte(self, len: usize) -> Self {
        self.length(len, ConditionOperator::GreaterThanOrEqual)
    }
    
    /// Add a condition that length is less than or equal to the expected value
    pub fn length_lte(self, len: usize) -> Self {
        self.length(len, ConditionOperator::LessThanOrEqual)
    }
    
    /// Add a raw condition
    pub fn condition(mut self, condition: Condition<'a, T>) -> Self {
        self.conditions.push(condition);
        self
    }
    
    /// Build the matcher
    pub fn build(self) -> Matcher<'a, T> {
        Matcher {
            mode: self.mode,
            conditions: self.conditions,
        }
    }
}

impl<'a, T: Matchable + 'static> Default for MatcherBuilder<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for field-based conditions
/// 
/// ## Example
/// 
/// ```rust
/// use condition_matcher::{FieldConditionBuilder, Matchable, MatchableDerive, Matcher, MatcherMode};
/// 
/// #[derive(MatchableDerive, PartialEq)]
/// struct User {
///     age: u32,
/// }
/// 
/// let condition = FieldConditionBuilder::<User>::new("age").gte(&18u32);
/// 
/// let mut matcher = Matcher::new(MatcherMode::AND);
/// matcher.add_condition(condition);
/// 
/// let user = User { age: 25 };
/// assert!(matcher.run(&user).unwrap());
/// ```
pub struct FieldConditionBuilder<'a, T> {
    field: &'a str,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Matchable> FieldConditionBuilder<'a, T> {
    /// Create a new field condition builder for the given field
    pub fn new(field: &'a str) -> Self {
        Self {
            field,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Field equals value
    pub fn equals(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::Equals,
        }
    }
    
    /// Field not equals value
    pub fn not_equals(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::NotEquals,
        }
    }
    
    /// Field greater than value
    pub fn gt(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::GreaterThan,
        }
    }
    
    /// Field greater than or equal value
    pub fn gte(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::GreaterThanOrEqual,
        }
    }
    
    /// Field less than value
    pub fn lt(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::LessThan,
        }
    }
    
    /// Field less than or equal value
    pub fn lte(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::LessThanOrEqual,
        }
    }
    
    /// Field contains substring (for string fields)
    pub fn contains(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::Contains,
        }
    }
    
    /// Field starts with prefix (for string fields)
    pub fn starts_with(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::StartsWith,
        }
    }
    
    /// Field ends with suffix (for string fields)
    pub fn ends_with(self, value: &'a dyn Any) -> Condition<'a, T> {
        Condition {
            selector: ConditionSelector::FieldValue(self.field, value),
            operator: ConditionOperator::EndsWith,
        }
    }
}

/// Convenience function to create a field condition builder
pub fn field<'a, T: Matchable>(name: &'a str) -> FieldConditionBuilder<'a, T> {
    FieldConditionBuilder::new(name)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{ConditionOperator, ConditionSelector};

    #[test]
    fn test_matcher_and_mode() {
        let mut matcher: Matcher<&str> = Matcher::new(MatcherMode::AND);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(5),
                operator: ConditionOperator::GreaterThanOrEqual,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("something"),
                operator: ConditionOperator::NotEquals,
            });
        
        assert_eq!(matcher.run(&"test").unwrap(), false);
        assert_eq!(matcher.run(&"test12345").unwrap(), true);
        assert_eq!(matcher.run(&"something").unwrap(), false);
        assert_eq!(matcher.run(&"somethingelse").unwrap(), true);
    }

    #[test]
    fn test_matcher_or_mode() {
        let mut matcher: Matcher<&str> = Matcher::new(MatcherMode::OR);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("hello"),
                operator: ConditionOperator::Equals,
            });
        
        assert_eq!(matcher.run(&"test").unwrap(), true);
        assert_eq!(matcher.run(&"hello").unwrap(), true);
        assert_eq!(matcher.run(&"world").unwrap(), false);
    }

    #[test]
    fn test_matcher_xor_mode() {
        let mut matcher: Matcher<&str> = Matcher::new(MatcherMode::XOR);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("test"),
                operator: ConditionOperator::Equals,
            });
        
        assert_eq!(matcher.run(&"test").unwrap(), false);
        assert_eq!(matcher.run(&"hello").unwrap(), false);
        assert_eq!(matcher.run(&"abcd").unwrap(), true);
    }

    #[test]
    fn test_type_checking() {
        let mut matcher: Matcher<&str> = Matcher::new(MatcherMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::Type("&str".to_string()),
            operator: ConditionOperator::Equals,
        });
        
        assert_eq!(matcher.run(&"test").unwrap(), true);
    }

    #[test]
    fn test_field_checking() {
        use crate::matcher::MatchableDerive;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct TestStruct {
            a: i32,
            b: String,
        }

        let test_value = TestStruct {
            a: 1,
            b: "test".to_string(),
        };

        // Test equals
        let mut matcher: Matcher<TestStruct> = Matcher::new(MatcherMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("a", &1i32),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher.run(&test_value).unwrap(), true);

        // Test not equals
        let mut matcher2: Matcher<TestStruct> = Matcher::new(MatcherMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("a", &2i32),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher2.run(&test_value).unwrap(), false);

        // Test string field
        let mut matcher3: Matcher<TestStruct> = Matcher::new(MatcherMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("b", &"test"),
            operator: ConditionOperator::Equals,
        });
        assert_eq!(matcher3.run(&test_value).unwrap(), true);
    }

    #[test]
    fn test_numeric_comparisons_on_fields() {
        use crate::matcher::MatchableDerive;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Person {
            age: u32,
            score: f64,
        }

        let person = Person { age: 25, score: 85.5 };

        // Test greater than
        let mut matcher: Matcher<Person> = Matcher::new(MatcherMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("age", &18u32),
            operator: ConditionOperator::GreaterThan,
        });
        assert!(matcher.run(&person).unwrap());

        // Test less than or equal
        let mut matcher2: Matcher<Person> = Matcher::new(MatcherMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("age", &25u32),
            operator: ConditionOperator::LessThanOrEqual,
        });
        assert!(matcher2.run(&person).unwrap());

        // Test float comparison
        let mut matcher3: Matcher<Person> = Matcher::new(MatcherMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("score", &80.0f64),
            operator: ConditionOperator::GreaterThan,
        });
        assert!(matcher3.run(&person).unwrap());
    }

    #[test]
    fn test_string_operations() {
        use crate::matcher::MatchableDerive;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Email {
            address: String,
        }

        let email = Email {
            address: "user@example.com".to_string(),
        };

        // Test contains
        let mut matcher: Matcher<Email> = Matcher::new(MatcherMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"@example"),
            operator: ConditionOperator::Contains,
        });
        assert!(matcher.run(&email).unwrap());

        // Test starts with
        let mut matcher2: Matcher<Email> = Matcher::new(MatcherMode::AND);
        matcher2.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"user@"),
            operator: ConditionOperator::StartsWith,
        });
        assert!(matcher2.run(&email).unwrap());

        // Test ends with
        let mut matcher3: Matcher<Email> = Matcher::new(MatcherMode::AND);
        matcher3.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &".com"),
            operator: ConditionOperator::EndsWith,
        });
        assert!(matcher3.run(&email).unwrap());

        // Test not contains
        let mut matcher4: Matcher<Email> = Matcher::new(MatcherMode::AND);
        matcher4.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &"@gmail"),
            operator: ConditionOperator::NotContains,
        });
        assert!(matcher4.run(&email).unwrap());
    }

    #[test]
    fn test_detailed_results() {
        let mut matcher: Matcher<&str> = Matcher::new(MatcherMode::AND);
        matcher
            .add_condition(Condition {
                selector: ConditionSelector::Length(4),
                operator: ConditionOperator::Equals,
            })
            .add_condition(Condition {
                selector: ConditionSelector::Value("test"),
                operator: ConditionOperator::Equals,
            });
        
        let result = matcher.run_detailed(&"test").unwrap();
        assert!(result.is_match());
        assert_eq!(result.passed_conditions().len(), 2);
        assert_eq!(result.failed_conditions().len(), 0);
        
        let result2 = matcher.run_detailed(&"hello").unwrap();
        assert!(!result2.is_match());
        assert_eq!(result2.passed_conditions().len(), 0);
        assert_eq!(result2.failed_conditions().len(), 2);
    }

    #[test]
    fn test_builder_api() {
        let matcher = MatcherBuilder::<&str>::new()
            .mode(MatcherMode::AND)
            .length_gte(4)
            .value_not_equals("bad")
            .build();
        
        assert!(matcher.run(&"good").unwrap());
        assert!(!matcher.run(&"bad").unwrap());
        assert!(!matcher.run(&"hi").unwrap());
    }

    #[test]
    fn test_field_builder() {
        use crate::matcher::MatchableDerive;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct User {
            age: u32,
        }

        let user = User { age: 25 };

        let condition = field::<User>("age").gte(&18u32);
        let mut matcher = Matcher::new(MatcherMode::AND);
        matcher.add_condition(condition);
        
        assert!(matcher.run(&user).unwrap());
    }

    #[test]
    fn test_convenience_constructors() {
        let and_matcher: Matcher<&str> = Matcher::and();
        assert_eq!(and_matcher.mode, MatcherMode::AND);
        
        let or_matcher: Matcher<&str> = Matcher::or();
        assert_eq!(or_matcher.mode, MatcherMode::OR);
        
        let xor_matcher: Matcher<&str> = Matcher::xor();
        assert_eq!(xor_matcher.mode, MatcherMode::XOR);
    }

    #[test]
    fn test_error_on_missing_field() {
        use crate::matcher::MatchableDerive;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct User {
            name: String,
        }

        let user = User { name: "Alice".to_string() };

        let mut matcher: Matcher<User> = Matcher::new(MatcherMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("nonexistent", &"value"),
            operator: ConditionOperator::Equals,
        });

        let result = matcher.run_detailed(&user).unwrap();
        assert!(!result.is_match());
        
        let failed = result.failed_conditions();
        assert_eq!(failed.len(), 1);
        assert!(failed[0].error.is_some());
    }

    #[test]
    fn test_not_operator() {
        use crate::matcher::MatchableDerive;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Item {
            active: bool,
        }

        let item = Item { active: false };

        // Test NOT operator - should match because NOT(active=true) is true when active=false
        let inner_condition = Condition {
            selector: ConditionSelector::FieldValue("active", &true),
            operator: ConditionOperator::Equals,
        };

        let mut matcher: Matcher<Item> = Matcher::new(MatcherMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::Not(Box::new(inner_condition)),
            operator: ConditionOperator::Equals, // operator is ignored for NOT
        });

        assert!(matcher.run(&item).unwrap());
    }

    #[test]
    fn test_optional_fields() {
        use crate::matcher::MatchableDerive;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Profile {
            name: String,
            nickname: Option<String>,
        }

        let profile_with_nick = Profile {
            name: "Alice".to_string(),
            nickname: Some("Ali".to_string()),
        };

        let profile_without_nick = Profile {
            name: "Bob".to_string(),
            nickname: None,
        };

        // Test matching optional field when present
        let mut matcher: Matcher<Profile> = Matcher::new(MatcherMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("nickname", &"Ali"),
            operator: ConditionOperator::Equals,
        });

        assert!(matcher.run(&profile_with_nick).unwrap());
        // When None, field access returns None, so the match fails
        assert!(!matcher.run(&profile_without_nick).unwrap());
    }

    #[cfg(feature = "regex")]
    #[test]
    fn test_regex_matching() {
        use crate::matcher::MatchableDerive;

        #[derive(MatchableDerive, PartialEq, Debug)]
        struct Email {
            address: String,
        }

        let email = Email {
            address: "user@example.com".to_string(),
        };

        let mut matcher: Matcher<Email> = Matcher::new(MatcherMode::AND);
        matcher.add_condition(Condition {
            selector: ConditionSelector::FieldValue("address", &r"^[a-z]+@[a-z]+\.[a-z]+$"),
            operator: ConditionOperator::Regex,
        });

        assert!(matcher.run(&email).unwrap());

        // Test non-matching regex
        let bad_email = Email {
            address: "not-an-email".to_string(),
        };
        assert!(!matcher.run(&bad_email).unwrap());
    }
}
