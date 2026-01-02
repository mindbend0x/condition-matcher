//! Comparison utilities for evaluating conditions.

use std::any::Any;
use std::fmt;

use crate::condition::ConditionOperator;

/// Compare two numeric values with an operator.
pub fn compare_numeric<N: PartialOrd>(actual: N, expected: N, operator: &ConditionOperator) -> bool {
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

/// Compare two type-erased values.
/// Returns (passed, actual_string, expected_string).
pub fn compare_any_values(
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

/// Try to compare string values with string-specific operators.
pub fn try_compare_strings(
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

