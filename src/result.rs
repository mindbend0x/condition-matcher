use crate::{condition::ConditionMode, error::MatchError};

/// Result of a match operation with detailed information
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Whether the overall match succeeded
    pub matched: bool,
    /// Individual results for each condition
    pub condition_results: Vec<ConditionResult>,
    /// The matching mode used
    pub mode: ConditionMode,
}

impl MatchResult {
    /// Returns true if the match succeeded
    pub fn is_match(&self) -> bool {
        self.matched
    }

    /// Returns the conditions that passed
    pub fn passed_conditions(&self) -> Vec<&ConditionResult> {
        self.condition_results.iter().filter(|r| r.passed).collect()
    }

    /// Returns the conditions that failed
    pub fn failed_conditions(&self) -> Vec<&ConditionResult> {
        self.condition_results
            .iter()
            .filter(|r| !r.passed)
            .collect()
    }
}

/// Result of evaluating a single condition
#[derive(Debug, Clone)]
pub struct ConditionResult {
    /// Whether this condition passed
    pub passed: bool,
    /// Description of what was checked
    pub description: String,
    /// The actual value that was compared (as string for display)
    pub actual_value: Option<String>,
    /// The expected value (as string for display)
    pub expected_value: Option<String>,
    /// Error if evaluation failed
    pub error: Option<MatchError>,
}