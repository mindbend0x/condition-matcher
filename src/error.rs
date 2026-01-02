use std::fmt;

/// Errors that can occur during condition matching
#[derive(Debug, Clone, PartialEq)]
pub enum MatchError {
    /// The specified field was not found on the type
    FieldNotFound {
        field: String,
        type_name: String,
    },
    /// Type mismatch between expected and actual values
    TypeMismatch {
        field: String,
        expected: String,
        actual: String,
    },
    /// The operator is not supported for this type/context
    UnsupportedOperator {
        operator: String,
        context: String,
    },
    /// Length check is not supported for this type
    LengthNotSupported {
        type_name: String,
    },
    /// Regex compilation failed
    #[cfg(feature = "regex")]
    RegexError {
        pattern: String,
        message: String,
    },
    /// The field path is empty
    EmptyFieldPath,
    /// Nested field not found
    NestedFieldNotFound {
        path: Vec<String>,
        failed_at: String,
    },
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatchError::FieldNotFound { field, type_name } => {
                write!(f, "Field '{}' not found on type '{}'", field, type_name)
            }
            MatchError::TypeMismatch { field, expected, actual } => {
                write!(f, "Type mismatch for field '{}': expected '{}', got '{}'", field, expected, actual)
            }
            MatchError::UnsupportedOperator { operator, context } => {
                write!(f, "Operator '{}' not supported for {}", operator, context)
            }
            MatchError::LengthNotSupported { type_name } => {
                write!(f, "Length check not supported for type '{}'", type_name)
            }
            #[cfg(feature = "regex")]
            MatchError::RegexError { pattern, message } => {
                write!(f, "Invalid regex pattern '{}': {}", pattern, message)
            }
            MatchError::EmptyFieldPath => {
                write!(f, "Field path cannot be empty")
            }
            MatchError::NestedFieldNotFound { path, failed_at } => {
                write!(f, "Nested field not found at '{}' in path {:?}", failed_at, path)
            }
        }
    }
}

impl std::error::Error for MatchError {}