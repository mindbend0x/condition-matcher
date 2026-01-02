//! Field evaluator for comparing field values.

use std::any::Any;

use crate::{
    condition::ConditionOperator, error::MatchError, matchable::Matchable, result::ConditionResult,
};

use super::comparison::compare_any_values;

/// Evaluator for single field comparisons.
pub struct FieldEvaluator;

impl FieldEvaluator {
    /// Evaluate a field condition against a Matchable value.
    pub fn evaluate<T: Matchable>(
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
}

