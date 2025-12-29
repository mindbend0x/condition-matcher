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
//! use condition_matcher::{Matcher, MatcherMode, Condition, ConditionSelector, ConditionOperator, Matchable, MatchableDerive};
//!
//! #[derive(MatchableDerive, PartialEq, Debug)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! let user = User { name: "Alice".to_string(), age: 30 };
//!
//! let mut matcher = Matcher::new(MatcherMode::AND);
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

pub mod condition;
pub mod matcher;

// Re-export main types for convenience
pub use condition::{Condition, ConditionOperator, ConditionSelector};
pub use matcher::{
    field, ConditionResult, FieldConditionBuilder, MatchError, MatchResult, Matchable,
    MatchableDerive, Matcher, MatcherBuilder, MatcherMode,
};
