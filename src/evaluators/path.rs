//! Path evaluator for comparing nested field values.

use std::any::Any;

use crate::{
    condition::ConditionOperator, error::MatchError, matchable::Matchable, result::ConditionResult,
};

use super::comparison::compare_any_values;

/// Evaluator for nested field path comparisons.
pub struct PathEvaluator;

impl PathEvaluator {
    /// Evaluate a field path condition against a Matchable value.
    pub fn evaluate<T: Matchable>(
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

