//! JSON evaluator for evaluating JSON-based conditions.

use std::any::Any;

use crate::{
    condition::{ConditionMode, ConditionOperator, JsonCondition, JsonNestedCondition},
    matchable::Matchable,
    result::{JsonConditionResult, JsonEvalResult},
};

/// Evaluator for JSON-based conditions.
pub struct JsonEvaluator;

impl JsonEvaluator {
    /// Evaluate a JsonNestedCondition against a Matchable value.
    pub fn evaluate<T: Matchable>(
        condition: &JsonNestedCondition,
        value: &T,
    ) -> JsonEvalResult {
        let mut details = Vec::new();
        Self::evaluate_recursive(condition, value, &mut details)
    }

    fn evaluate_recursive<T: Matchable>(
        group: &JsonNestedCondition,
        value: &T,
        details: &mut Vec<JsonConditionResult>,
    ) -> JsonEvalResult {
        let mut flags = Vec::new();

        // Evaluate all rules at this level
        for rule in &group.rules {
            let result = Self::evaluate_rule(rule, value);
            flags.push(result.passed);
            details.push(result);
        }

        // Evaluate nested groups recursively
        for nested in &group.nested {
            let nested_result = Self::evaluate_recursive(nested, value, details);
            flags.push(nested_result.matched);
        }

        let matched = combine_results(&flags, group.mode);
        JsonEvalResult {
            matched,
            details: details.clone(),
        }
    }

    fn evaluate_rule<T: Matchable>(rule: &JsonCondition, value: &T) -> JsonConditionResult {
        let field = &rule.field;

        // Support dotted paths like "user.age" by splitting on '.'
        let path_segments: Vec<&str> = field.split('.').collect();

        // Try to resolve the field value
        let actual_value = if path_segments.len() == 1 {
            value.get_field(field)
        } else {
            value.get_field_path(&path_segments)
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
}

/// Extract a numeric value as f64 from a type-erased Any reference.
pub fn extract_as_f64(actual: &dyn Any) -> Option<f64> {
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

/// Compare a JSON value against a type-erased Any reference.
pub fn compare_json_to_any(
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

fn combine_results(results: &[bool], mode: ConditionMode) -> bool {
    match mode {
        ConditionMode::AND => results.iter().all(|&r| r),
        ConditionMode::OR => results.iter().any(|&r| r),
        ConditionMode::XOR => results.iter().filter(|&&r| r).count() == 1,
    }
}

