//! # Condition Matcher Derive
//!
//! This crate provides the `#[derive(Matchable)]` procedural macro for the `condition-matcher` crate.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use condition_matcher::{Matchable, MatchableDerive};
//!
//! #[derive(MatchableDerive, PartialEq)]
//! struct User {
//!     name: String,
//!     age: u32,
//!     email: Option<String>,
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// Derive macro for implementing the `Matchable` trait.
///
/// This macro automatically implements the `get_field` method for structs,
/// allowing field-based condition matching.
///
/// ## Example
///
/// ```rust,ignore
/// #[derive(Matchable, PartialEq)]
/// struct Product {
///     id: i32,
///     name: String,
///     price: f64,
///     in_stock: bool,
///     description: Option<String>,
/// }
/// ```
///
/// The macro generates:
/// - `get_field(&self, field: &str) -> Option<&dyn Any>` - Returns a reference to any field by name
/// - Handles `Option<T>` fields by unwrapping them when present
#[proc_macro_derive(Matchable)]
pub fn matchable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let field_match_arms = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let arms = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    let field_name_str = field_name.as_ref().unwrap().to_string();
                    let field_type = &f.ty;
                    
                    // Check if the field is an Option type
                    if is_option_type(field_type) {
                        quote! {
                            #field_name_str => self.#field_name.as_ref().map(|v| v as &dyn std::any::Any),
                        }
                    } else {
                        quote! {
                            #field_name_str => Some(&self.#field_name as &dyn std::any::Any),
                        }
                    }
                });
                quote! {
                    #(#arms)*
                }
            }
            Fields::Unnamed(_) => {
                // For tuple structs, use indices
                quote! {}
            }
            Fields::Unit => {
                quote! {}
            }
        },
        Data::Enum(_) => {
            quote! {}
        }
        Data::Union(_) => {
            quote! {}
        }
    };

    // Generate is_none implementation for types with Option fields
    let has_option_fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields.named.iter().any(|f| is_option_type(&f.ty)),
            _ => false,
        },
        _ => false,
    };

    let is_none_impl = if has_option_fields {
        quote! {
            fn is_none(&self) -> bool {
                false
            }
        }
    } else {
        quote! {}
    };

    // Generate length implementation if the struct has a "len" field or method
    let length_impl = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let has_len_field = fields.named.iter().any(|f| {
                    f.ident.as_ref().map(|i| i == "len" || i == "length").unwrap_or(false)
                });
                if has_len_field {
                    quote! {
                        fn get_length(&self) -> Option<usize> {
                            Some(self.len as usize)
                        }
                    }
                } else {
                    quote! {}
                }
            }
            _ => quote! {},
        },
        _ => quote! {},
    };

    let expanded = quote! {
        impl #impl_generics Matchable for #name #ty_generics #where_clause {
            fn get_field(&self, field: &str) -> Option<&dyn std::any::Any> {
                match field {
                    #field_match_arms
                    _ => None,
                }
            }
            
            #length_impl
            #is_none_impl
        }
    };

    TokenStream::from(expanded)
}

/// Check if a type is an Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}
