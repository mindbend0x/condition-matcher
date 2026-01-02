//! Core traits for the condition-matcher library.
//!
//! This module defines the trait hierarchy that enables polymorphic matching:
//! - [`Matcher`]: Core trait for any type that can match against a value
//! - [`Evaluate`]: Extended trait for detailed evaluation results
//! - [`Predicate`]: Trait for individual condition evaluation
//! - [`MatcherExt`]: Extension trait providing batch operations

use crate::{
    condition::ConditionMode,
    matchable::Matchable,
    result::ConditionResult,
};

/// Core trait for any type that can match against a value.
///
/// This is the primary interface users interact with. Both [`RuleMatcher`](crate::matchers::RuleMatcher)
/// and [`JsonMatcher`](crate::matchers::JsonMatcher) implement this trait.
///
/// # Example
///
/// ```rust
/// use condition_matcher::{Matcher, MatcherBuilder};
///
/// let matcher = MatcherBuilder::<i32>::new()
///     .value_equals(42)
///     .build();
///
/// assert!(matcher.matches(&42));
/// assert!(!matcher.matches(&41));
/// ```
pub trait Matcher<T: Matchable> {
    /// Check if this matcher matches the given value.
    fn matches(&self, value: &T) -> bool;

    /// Get the logical combination mode (AND, OR, XOR).
    fn mode(&self) -> ConditionMode;
}

/// Extended trait for matchers that provide detailed evaluation results.
///
/// Implement this trait when you need to provide detailed information about
/// why a match succeeded or failed.
pub trait Evaluate<T: Matchable>: Matcher<T> {
    /// The result type for detailed evaluation.
    type Output;

    /// Evaluate with full details (field values, errors, descriptions).
    fn evaluate(&self, value: &T) -> Self::Output;
}

/// Trait for individual condition/predicate evaluation.
///
/// Implemented by [`Condition`](crate::condition::Condition) to evaluate
/// a single rule against a value.
pub trait Predicate<T: Matchable> {
    /// Test this predicate against a value.
    fn test(&self, value: &T) -> bool;

    /// Test with detailed result information.
    fn test_detailed(&self, value: &T) -> ConditionResult;
}

/// Extension trait providing batch operations.
///
/// This trait is blanket implemented for all [`Matcher`] types, providing
/// convenient methods for operating on collections of values.
///
/// # Example
///
/// ```rust
/// use condition_matcher::{Matcher, MatcherExt, MatcherBuilder};
///
/// let matcher = MatcherBuilder::<i32>::new()
///     .value_equals(42)
///     .build();
///
/// let values = vec![40, 41, 42, 43, 42];
/// let matches = matcher.filter(&values);
/// assert_eq!(matches.len(), 2);
/// ```
pub trait MatcherExt<T: Matchable>: Matcher<T> {
    /// Filter values, returning references to those that match.
    fn filter<'a>(&self, values: &'a [T]) -> Vec<&'a T> {
        values.iter().filter(|v| self.matches(v)).collect()
    }

    /// Filter values in parallel (requires `parallel` feature).
    #[cfg(feature = "parallel")]
    fn filter_par<'a>(&self, values: &'a [T]) -> Vec<&'a T>
    where
        T: Sync,
        Self: Sync,
    {
        use rayon::prelude::*;
        values.par_iter().filter(|v| self.matches(v)).collect()
    }

    /// Check all values, returning match results as a vector of booleans.
    fn matches_all(&self, values: &[T]) -> Vec<bool> {
        values.iter().map(|v| self.matches(v)).collect()
    }

    /// Check all values in parallel (requires `parallel` feature).
    #[cfg(feature = "parallel")]
    fn matches_all_par(&self, values: &[T]) -> Vec<bool>
    where
        T: Sync,
        Self: Sync,
    {
        use rayon::prelude::*;
        values.par_iter().map(|v| self.matches(v)).collect()
    }
}

// Blanket implementation - any Matcher gets batch operations for free
impl<T: Matchable, M: Matcher<T>> MatcherExt<T> for M {}

// Blanket implementation - references to Matchers also implement Matcher
impl<T: Matchable, M: Matcher<T>> Matcher<T> for &M {
    fn matches(&self, value: &T) -> bool {
        (*self).matches(value)
    }

    fn mode(&self) -> ConditionMode {
        (*self).mode()
    }
}

