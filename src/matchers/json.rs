//! JSON-based matcher implementation.

use crate::{
    condition::{ConditionMode, JsonNestedCondition},
    evaluators::JsonEvaluator,
    matchable::Matchable,
    result::JsonEvalResult,
    traits::{Evaluate, Matcher},
};

/// A matcher for JSON-deserialized conditions.
///
/// Ideal for conditions loaded from databases or config files.
///
/// # Example
///
/// ```rust,ignore
/// use condition_matcher::{JsonMatcher, Matcher};
///
/// let json = r#"{"mode": "AND", "rules": [{"field": "age", "operator": "greater_than_or_equal", "value": 18}]}"#;
/// let matcher = JsonMatcher::from_json(json).unwrap();
///
/// // Evaluate against a Matchable type
/// assert!(matcher.matches(&user));
/// ```
#[derive(Debug, Clone)]
pub struct JsonMatcher(pub JsonNestedCondition);

impl JsonMatcher {
    /// Parse a matcher from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let condition: JsonNestedCondition = serde_json::from_str(json)?;
        Ok(JsonMatcher(condition))
    }

    /// Parse a matcher from a serde_json::Value.
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let condition: JsonNestedCondition = serde_json::from_value(value)?;
        Ok(JsonMatcher(condition))
    }

    /// Create from an existing JsonNestedCondition.
    pub fn from_condition(condition: JsonNestedCondition) -> Self {
        JsonMatcher(condition)
    }

    /// Get a reference to the underlying condition.
    pub fn condition(&self) -> &JsonNestedCondition {
        &self.0
    }
}

impl serde::Serialize for JsonMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for JsonMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let condition = JsonNestedCondition::deserialize(deserializer)?;
        Ok(JsonMatcher(condition))
    }
}

impl<T: Matchable> Matcher<T> for JsonMatcher {
    fn matches(&self, value: &T) -> bool {
        JsonEvaluator::evaluate(&self.0, value).matched
    }

    fn mode(&self) -> ConditionMode {
        self.0.mode
    }
}

impl<T: Matchable> Evaluate<T> for JsonMatcher {
    type Output = JsonEvalResult;

    fn evaluate(&self, value: &T) -> JsonEvalResult {
        JsonEvaluator::evaluate(&self.0, value)
    }
}

