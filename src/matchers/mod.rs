//! Matcher implementations.
//!
//! This module contains concrete matcher types that implement the [`Matcher`](crate::traits::Matcher) trait.

mod rule;

#[cfg(feature = "json_condition")]
mod json;

pub use rule::RuleMatcher;

#[cfg(feature = "json_condition")]
pub use json::JsonMatcher;

