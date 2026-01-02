//! # Condition Matcher
//!
//! A flexible and type-safe condition matching library for Rust with automatic struct field access.
//!
//! ## Features
//!
//! - **Automatic struct matching** with derive macro
//! - Multiple matching modes (AND, OR, XOR)
//! - Support for various condition types (value, length, type, field)
//! - String operations (contains, starts_with, ends_with)
//! - Numeric comparisons on fields
//! - Detailed match results with error information
//! - Builder pattern for ergonomic API
//! - Optional serde, regex, and parallel processing support
//!
//! ## Quick Start
//!
//! ```rust
//! use condition_matcher::{RuleMatcher, MatcherMode, Condition, ConditionSelector, ConditionOperator, MatchableDerive, Matcher, Matchable, MatcherBuilder};
//!
//! #[derive(MatchableDerive, PartialEq, Debug)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! let user = User { name: "Alice".to_string(), age: 30 };
//!
//! let mut matcher = MatcherBuilder::<User>::new()
//!     .mode(MatcherMode::AND)
//!     .value_equals(User { name: "Alice".to_string(), age: 30 })
//!     .build();
//! matcher.add_condition(Condition {
//!     selector: ConditionSelector::FieldValue("age", &18u32),
//!     operator: ConditionOperator::GreaterThanOrEqual,
//! });
//!
//! assert!(matcher.matches(&user));
//! ```
//!
//! ## Builder API
//!
//! ```rust
//! use condition_matcher::{MatcherBuilder, MatcherMode, Matcher};
//!
//! let matcher = MatcherBuilder::<&str>::new()
//!     .mode(MatcherMode::AND)
//!     .length_gte(4)
//!     .value_not_equals("bad")
//!     .build();
//!
//! assert!(matcher.matches(&"good"));
//! ```
//!
//! ## Batch Operations
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
//! // Evaluate all combinations (with parallel feature)
//! let all_matches = batch::parallel::evaluate_matrix(&records, &matchers);
//! ```

// Core modules
mod traits;
mod condition;
mod matchable;
mod matchers;
mod evaluators;
mod result;
mod error;

/// Builder module for creating matchers.
pub mod builder;

// Batch operations module
pub mod batch;

// Keep old matcher module for backwards compatibility
mod matcher;

#[cfg(test)]
mod test;

// ============================================================================
// Core Traits
// ============================================================================

pub use traits::{Evaluate, Matcher, MatcherExt, Predicate};

// ============================================================================
// Matchers
// ============================================================================

pub use matchers::RuleMatcher;

#[cfg(feature = "json_condition")]
pub use matchers::JsonMatcher;

// ============================================================================
// Conditions
// ============================================================================

pub use condition::{
    Condition, ConditionMode, ConditionOperator, ConditionSelector, NestedCondition,
};

#[cfg(feature = "json_condition")]
pub use condition::{JsonCondition, JsonNestedCondition};

// ============================================================================
// Builder
// ============================================================================

pub use builder::{field, FieldConditionBuilder, MatcherBuilder};

// ============================================================================
// Results and Errors
// ============================================================================

pub use result::{ConditionResult, MatchResult};

#[cfg(feature = "json_condition")]
pub use result::{JsonConditionResult, JsonEvalResult};

pub use error::MatchError;

// ============================================================================
// Data Access
// ============================================================================

pub use matchable::Matchable;

// ============================================================================
// Derive Macro
// ============================================================================

pub use condition_matcher_derive::Matchable as MatchableDerive;

// ============================================================================
// Legacy Aliases (backwards compatibility)
// ============================================================================

/// Alias for [`ConditionMode`] for backwards compatibility.
pub type MatcherMode = ConditionMode;

// Re-export the old Matcher struct for backwards compatibility
// Users should migrate to RuleMatcher
#[doc(hidden)]
pub use matcher::Matcher as OldMatcher;

// Re-export evaluate_json_condition for backwards compatibility
#[cfg(feature = "json_condition")]
pub use matcher::evaluate_json_condition;
