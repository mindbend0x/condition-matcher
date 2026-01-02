use std::any::Any;

/// Trait for types that can be matched against conditions.
/// 
/// This trait allows different types to opt-in to specific matching capabilities.
/// For structs, you can use `#[derive(Matchable)]` to automatically implement field access.
/// 
/// ## Example
/// 
/// ```rust
/// use condition_matcher::{Matchable, MatchableDerive};
/// use std::any::Any;
/// 
/// #[derive(MatchableDerive, PartialEq)]
/// struct MyStruct {
///     value: i32,
///     name: String,
/// }
/// 
/// // The derive macro automatically implements get_field for all fields
/// ```
pub trait Matchable: PartialEq + Sized {
    /// Get the length of the value if supported (for strings, collections, etc.)
    fn get_length(&self) -> Option<usize> {
        None
    }
    
    /// Get a field value by name as a type-erased reference.
    /// Returns None if field access is not supported or field doesn't exist.
    fn get_field(&self, _field: &str) -> Option<&dyn Any> {
        None
    }
    
    /// Get a nested field value by path.
    /// Default implementation walks through get_field calls.
    fn get_field_path(&self, _path: &[&str]) -> Option<&dyn Any> {
        None
    }
    
    /// Get the type name as a string
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
    
    /// Check if the value is considered "empty" (for collections, strings, options)
    fn is_empty(&self) -> Option<bool> {
        self.get_length().map(|len| len == 0)
    }
    
    /// Check if this is a None/null value (for Option types)
    fn is_none(&self) -> bool {
        false
    }
}