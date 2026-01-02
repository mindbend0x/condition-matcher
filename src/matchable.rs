use std::any::Any;
use std::collections::HashMap;

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

// ============================================================================
// Matchable Implementations for Common Types
// ============================================================================

impl Matchable for &str {
    fn get_length(&self) -> Option<usize> {
        Some(self.len())
    }

    fn is_empty(&self) -> Option<bool> {
        Some((*self).is_empty())
    }
}

impl Matchable for String {
    fn get_length(&self) -> Option<usize> {
        Some(self.len())
    }

    fn is_empty(&self) -> Option<bool> {
        Some(self.is_empty())
    }
}

impl<T: Matchable> Matchable for Vec<T> {
    fn get_length(&self) -> Option<usize> {
        Some(self.len())
    }

    fn is_empty(&self) -> Option<bool> {
        Some(self.is_empty())
    }
}

impl<K, V> Matchable for HashMap<K, V>
where
    K: std::borrow::Borrow<str> + std::hash::Hash + Eq,
    V: PartialEq + 'static,
{
    fn get_length(&self) -> Option<usize> {
        Some(self.len())
    }

    fn get_field(&self, field: &str) -> Option<&dyn Any> {
        self.get(field).map(|v| v as &dyn Any)
    }

    fn is_empty(&self) -> Option<bool> {
        Some(self.is_empty())
    }
}

impl<T: Matchable + 'static> Matchable for Option<T> {
    fn get_length(&self) -> Option<usize> {
        self.as_ref().and_then(|v| v.get_length())
    }

    fn get_field(&self, field: &str) -> Option<&dyn Any> {
        self.as_ref().and_then(|v| v.get_field(field))
    }

    fn is_none(&self) -> bool {
        self.is_none()
    }

    fn is_empty(&self) -> Option<bool> {
        Some(self.is_none())
    }
}

// Implement for primitive types
macro_rules! impl_matchable_primitive {
    ($($t:ty),*) => {
        $(
            impl Matchable for $t {}
        )*
    };
}

impl_matchable_primitive!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool, char
);
