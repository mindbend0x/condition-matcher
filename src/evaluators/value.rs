//! Value evaluator for comparing values directly.

use crate::{condition::ConditionOperator, matchable::Matchable, result::ConditionResult};

/// Evaluator for direct value comparisons.
pub struct ValueEvaluator;

impl ValueEvaluator {
    /// Evaluate a value condition against a Matchable value.
    pub fn evaluate<T: Matchable>(
        value: &T,
        expected: &T,
        operator: &ConditionOperator,
    ) -> ConditionResult {
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
}

