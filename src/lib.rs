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
//! - Optional serde and regex support
//!
//! ## Quick Start
//!
//! ```rust
//! use condition_matcher::{Matcher, MatcherMode, Condition, ConditionSelector, ConditionOperator, MatchableDerive};
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
//! assert!(matcher.run(&user).unwrap());
//! ```
//!
//! ## Builder API
//!
//! ```rust
//! use condition_matcher::{MatcherBuilder, MatcherMode};
//!
//! let matcher = MatcherBuilder::<&str>::new()
//!     .mode(MatcherMode::AND)
//!     .length_gte(4)
//!     .value_not_equals("bad")
//!     .build();
//!
//! assert!(matcher.run(&"good").unwrap());
//! ```

pub mod builder;
pub mod condition;
pub mod error;
pub mod matchable;
pub mod matcher;
pub mod result;

// Re-export main types for convenience
pub use builder::MatcherBuilder;
pub use condition::{
    Condition, ConditionMode, ConditionOperator, ConditionSelector, NestedCondition,
};
pub use error::MatchError;
pub use matchable::Matchable;
pub use matcher::Matcher;
pub use result::{ConditionResult, MatchResult};

// Re-export the derive macro
pub use condition_matcher_derive::Matchable as MatchableDerive;

// Re-export JSON condition types when json_condition feature is enabled
#[cfg(feature = "json_condition")]
pub use condition::{JsonCondition, JsonNestedCondition};
#[cfg(feature = "json_condition")]
pub use matcher::{JsonConditionResult, JsonEvalResult, evaluate_json_condition};

#[cfg(test)]
mod test;
