//! Builder pattern for creating matchers.

use crate::{
    condition::{Condition, ConditionMode, ConditionOperator, ConditionSelector},
    matchable::Matchable,
    matchers::RuleMatcher,
};
use std::any::Any;

// ============================================================================
// Builder API
// ============================================================================

/// A builder for creating matchers with a fluent API
///
/// ## Example
///
/// ```rust
/// use condition_matcher::{MatcherBuilder, MatcherMode, ConditionOperator, Matcher};
///
/// let matcher = MatcherBuilder::<i32>::new()
///     .mode(MatcherMode::AND)
///     .value_equals(42)
///     .build();
///
/// assert!(matcher.matches(&42));
/// ```
pub struct MatcherBuilder<'a, T: Matchable> {
    mode: ConditionMode,
    conditions: Vec<Condition<'a, T>>,
}

impl<'a, T: Matchable + 'static> MatcherBuilder<'a, T> {
    /// Create a new builder with default AND mode
    pub fn new() -> Self {
        Self {
            mode: ConditionMode::AND,
            conditions: Vec::new(),
        }
    }

    /// Set the matching mode
    pub fn mode(mut self, mode: ConditionMode) -> Self {
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
    pub fn build(self) -> RuleMatcher<'a, T> {
        RuleMatcher {
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
/// use condition_matcher::{FieldConditionBuilder, Matchable, MatchableDerive, RuleMatcher, MatcherMode, Matcher};
///
/// #[derive(MatchableDerive, PartialEq)]
/// struct User {
///     age: u32,
/// }
///
/// let condition = FieldConditionBuilder::<User>::new("age").gte(&18u32);
///
/// let mut matcher = RuleMatcher::new(MatcherMode::AND);
/// matcher.add_condition(condition);
///
/// let user = User { age: 25 };
/// assert!(matcher.matches(&user));
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
