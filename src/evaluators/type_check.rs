//! Type evaluator for comparing type names.

use crate::{condition::ConditionOperator, matchable::Matchable, result::ConditionResult};

/// Evaluator for type name comparisons.
pub struct TypeEvaluator;

impl TypeEvaluator {
    /// Evaluate a type condition against a Matchable value.
    pub fn evaluate<T: Matchable>(
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
}

