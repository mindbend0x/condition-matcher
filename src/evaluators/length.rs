//! Length evaluator for comparing lengths of strings, collections, etc.

use crate::{
    condition::ConditionOperator, error::MatchError, matchable::Matchable, result::ConditionResult,
};

use super::comparison::compare_numeric;

/// Evaluator for length comparisons.
pub struct LengthEvaluator;

impl LengthEvaluator {
    /// Evaluate a length condition against a Matchable value.
    pub fn evaluate<T: Matchable>(
        value: &T,
        expected: usize,
        operator: &ConditionOperator,
    ) -> ConditionResult {
        match value.get_length() {
            Some(actual) => ConditionResult {
                passed: compare_numeric(actual, expected, operator),
                description: format!("length {:?} {}", operator, expected),
                actual_value: Some(actual.to_string()),
                expected_value: Some(expected.to_string()),
                error: None,
            },
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
}

