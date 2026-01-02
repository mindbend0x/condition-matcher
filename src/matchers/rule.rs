//! Rule-based matcher implementation.

use crate::{
    condition::{Condition, ConditionMode},
    matchable::Matchable,
    result::{ConditionResult, MatchResult},
    traits::{Evaluate, Matcher, Predicate},
};

/// A rule-based matcher built from programmatic conditions.
///
/// Use [`MatcherBuilder`](crate::builder::MatcherBuilder) for a fluent construction API.
///
/// # Example
///
/// ```rust
/// use condition_matcher::{RuleMatcher, MatcherMode, Condition, ConditionSelector, ConditionOperator, Matcher};
///
/// let mut matcher: RuleMatcher<i32> = RuleMatcher::new(MatcherMode::AND);
/// matcher.add_condition(Condition {
///     selector: ConditionSelector::Value(42),
///     operator: ConditionOperator::Equals,
/// });
///
/// assert!(matcher.matches(&42));
/// ```
#[derive(Debug)]
pub struct RuleMatcher<'a, T: Matchable> {
    /// The logical combination mode (AND, OR, XOR).
    pub mode: ConditionMode,
    /// The conditions to evaluate.
    pub conditions: Vec<Condition<'a, T>>,
}

impl<'a, T: Matchable + 'static> RuleMatcher<'a, T> {
    /// Create a new matcher with the specified mode.
    pub fn new(mode: ConditionMode) -> Self {
        Self {
            mode,
            conditions: Vec::new(),
        }
    }

    /// Create a new matcher with AND mode.
    pub fn and() -> Self {
        Self::new(ConditionMode::AND)
    }

    /// Create a new matcher with OR mode.
    pub fn or() -> Self {
        Self::new(ConditionMode::OR)
    }

    /// Create a new matcher with XOR mode.
    pub fn xor() -> Self {
        Self::new(ConditionMode::XOR)
    }

    /// Add a condition to this matcher.
    pub fn add_condition(&mut self, condition: Condition<'a, T>) -> &mut Self {
        self.conditions.push(condition);
        self
    }

    /// Add multiple conditions at once.
    pub fn add_conditions(
        &mut self,
        conditions: impl IntoIterator<Item = Condition<'a, T>>,
    ) -> &mut Self {
        self.conditions.extend(conditions);
        self
    }
}

impl<'a, T: Matchable + 'static> Matcher<T> for RuleMatcher<'a, T> {
    fn matches(&self, value: &T) -> bool {
        let results: Vec<bool> = self.conditions.iter().map(|c| c.test(value)).collect();
        combine_results(&results, self.mode)
    }

    fn mode(&self) -> ConditionMode {
        self.mode
    }
}

impl<'a, T: Matchable + 'static> Evaluate<T> for RuleMatcher<'a, T> {
    type Output = MatchResult;

    fn evaluate(&self, value: &T) -> MatchResult {
        let condition_results: Vec<ConditionResult> =
            self.conditions.iter().map(|c| c.test_detailed(value)).collect();

        let matched = combine_results(
            &condition_results.iter().map(|r| r.passed).collect::<Vec<_>>(),
            self.mode,
        );

        MatchResult {
            matched,
            condition_results,
            mode: self.mode,
        }
    }
}

fn combine_results(results: &[bool], mode: ConditionMode) -> bool {
    match mode {
        ConditionMode::AND => results.iter().all(|&r| r),
        ConditionMode::OR => results.iter().any(|&r| r),
        ConditionMode::XOR => results.iter().filter(|&&r| r).count() == 1,
    }
}

