//! Batch operations for evaluating matchers against collections.
//!
//! This module provides functions for common multi-value and multi-matcher scenarios:
//! - Finding which matchers match a single value
//! - Evaluating multiple matchers against multiple values (cartesian product)
//!
//! # Example
//!
//! ```rust,ignore
//! use condition_matcher::{batch, JsonMatcher, Matcher};
//!
//! let matchers: Vec<JsonMatcher> = load_from_database()?;
//! let records: Vec<Record> = get_records();
//!
//! // Find which matchers apply to a single record
//! let applicable = batch::matching(&records[0], &matchers);
//!
//! // Evaluate all combinations
//! let all_matches = batch::evaluate_matrix(&records, &matchers);
//! ```

use crate::{matchable::Matchable, traits::Matcher};

/// Find all matchers that match a single value.
///
/// Use case: "Which rules apply to this order?"
///
/// # Example
///
/// ```rust,ignore
/// let applicable_rules = batch::matching(&order, &promotion_rules);
/// for rule in applicable_rules {
///     apply_discount(rule);
/// }
/// ```
pub fn matching<'a, T, M>(value: &T, matchers: &'a [M]) -> Vec<&'a M>
where
    T: Matchable,
    M: Matcher<T>,
{
    matchers.iter().filter(|m| m.matches(value)).collect()
}

/// Find indices of all matchers that match a single value.
///
/// Use case: "Which rule indices apply to this record?"
pub fn matching_indices<T, M>(value: &T, matchers: &[M]) -> Vec<usize>
where
    T: Matchable,
    M: Matcher<T>,
{
    matchers
        .iter()
        .enumerate()
        .filter(|(_, m)| m.matches(value))
        .map(|(i, _)| i)
        .collect()
}

/// Count how many matchers match a single value.
pub fn count_matching<T, M>(value: &T, matchers: &[M]) -> usize
where
    T: Matchable,
    M: Matcher<T>,
{
    matchers.iter().filter(|m| m.matches(value)).count()
}

/// Check if any matcher matches the value.
pub fn any_matches<T, M>(value: &T, matchers: &[M]) -> bool
where
    T: Matchable,
    M: Matcher<T>,
{
    matchers.iter().any(|m| m.matches(value))
}

/// Check if all matchers match the value.
pub fn all_match<T, M>(value: &T, matchers: &[M]) -> bool
where
    T: Matchable,
    M: Matcher<T>,
{
    matchers.iter().all(|m| m.matches(value))
}

/// Evaluate all matchers against all values (cartesian product).
///
/// Returns indices of (value_idx, matcher_idx) pairs that matched.
///
/// Use case: "For each user, which notifications should they receive?"
///
/// # Example
///
/// ```rust,ignore
/// let matches = batch::evaluate_matrix(&users, &notification_rules);
/// for (user_idx, rule_idx) in matches {
///     send_notification(&users[user_idx], &notification_rules[rule_idx]);
/// }
/// ```
pub fn evaluate_matrix<T, M>(values: &[T], matchers: &[M]) -> Vec<(usize, usize)>
where
    T: Matchable,
    M: Matcher<T>,
{
    let mut results = Vec::new();
    for (v_idx, value) in values.iter().enumerate() {
        for (m_idx, matcher) in matchers.iter().enumerate() {
            if matcher.matches(value) {
                results.push((v_idx, m_idx));
            }
        }
    }
    results
}

/// Evaluate all matchers against all values, returning a 2D boolean matrix.
///
/// Returns `results[value_idx][matcher_idx]` indicating if that combination matched.
pub fn evaluate_matrix_full<T, M>(values: &[T], matchers: &[M]) -> Vec<Vec<bool>>
where
    T: Matchable,
    M: Matcher<T>,
{
    values
        .iter()
        .map(|value| matchers.iter().map(|m| m.matches(value)).collect())
        .collect()
}

/// For each value, find the first matcher that matches (if any).
///
/// Returns pairs of (value_idx, matcher_idx) for values that had at least one match.
pub fn first_matching<T, M>(values: &[T], matchers: &[M]) -> Vec<(usize, usize)>
where
    T: Matchable,
    M: Matcher<T>,
{
    values
        .iter()
        .enumerate()
        .filter_map(|(v_idx, value)| {
            matchers
                .iter()
                .enumerate()
                .find(|(_, m)| m.matches(value))
                .map(|(m_idx, _)| (v_idx, m_idx))
        })
        .collect()
}

// ============================================================================
// Parallel versions (requires `parallel` feature)
// ============================================================================

#[cfg(feature = "parallel")]
pub mod parallel {
    //! Parallel versions of batch operations using Rayon.

    use super::*;
    use rayon::prelude::*;

    /// Parallel version of [`matching`](super::matching).
    pub fn matching<'a, T, M>(value: &T, matchers: &'a [M]) -> Vec<&'a M>
    where
        T: Matchable + Sync,
        M: Matcher<T> + Sync,
    {
        matchers.par_iter().filter(|m| m.matches(value)).collect()
    }

    /// Parallel version of [`matching_indices`](super::matching_indices).
    pub fn matching_indices<T, M>(value: &T, matchers: &[M]) -> Vec<usize>
    where
        T: Matchable + Sync,
        M: Matcher<T> + Sync,
    {
        matchers
            .par_iter()
            .enumerate()
            .filter(|(_, m)| m.matches(value))
            .map(|(i, _)| i)
            .collect()
    }

    /// Parallel version of [`evaluate_matrix`](super::evaluate_matrix).
    pub fn evaluate_matrix<T, M>(values: &[T], matchers: &[M]) -> Vec<(usize, usize)>
    where
        T: Matchable + Sync,
        M: Matcher<T> + Sync,
    {
        values
            .par_iter()
            .enumerate()
            .flat_map(|(v_idx, value)| {
                matchers
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| m.matches(value))
                    .map(move |(m_idx, _)| (v_idx, m_idx))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Parallel version of [`evaluate_matrix_full`](super::evaluate_matrix_full).
    pub fn evaluate_matrix_full<T, M>(values: &[T], matchers: &[M]) -> Vec<Vec<bool>>
    where
        T: Matchable + Sync,
        M: Matcher<T> + Sync,
    {
        values
            .par_iter()
            .map(|value| matchers.iter().map(|m| m.matches(value)).collect())
            .collect()
    }

    /// Parallel version of [`any_matches`](super::any_matches).
    pub fn any_matches<T, M>(value: &T, matchers: &[M]) -> bool
    where
        T: Matchable + Sync,
        M: Matcher<T> + Sync,
    {
        matchers.par_iter().any(|m| m.matches(value))
    }

    /// Parallel version of [`all_match`](super::all_match).
    pub fn all_match<T, M>(value: &T, matchers: &[M]) -> bool
    where
        T: Matchable + Sync,
        M: Matcher<T> + Sync,
    {
        matchers.par_iter().all(|m| m.matches(value))
    }
}

