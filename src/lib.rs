//! This crate provides a derive-macro to generate the builder pattern for a struct.
//! The builder implementation contains a method for each field of the struct, ignoring fields with
//! the #[skip] attribute.
//! Each field to generate a method for must be of type Option<T>. If any field is not of type
//! Option<T>, and doesn't have the #[skip] attribute, the macro will panic.
//!
//!
//! # Examples
//! ```
//! use build_it::Builder;
//! #[derive(Default, Builder)]
//! struct MyAwesomeStruct {
//!     name: Option<String>,
//!     pub age: Option<u32>,
//!     #[skip]
//!     address: String,
//!     #[skip]
//!     pub phone: Option<String>,
//! }
//! let builder = MyAwesomeStruct::default()
//!     .name("Alice".to_string())
//!     .age(42);
//!     // Note that `address` and `phone` do not have builder methods because of the #[skip]
//!     // attribute.
//! assert_eq!(builder.name, Some("Alice".to_string()));
//! assert_eq!(builder.age, Some(42));
//!
//! // These fields are skipped, so they're value will still be the default value.
//! assert_eq!(builder.address, String::default());
//! assert_eq!(builder.phone, None);
//!```
//!
//! The generated builder methods will also display the field's documentation:
//! ```
//! use build_it::Builder;
//! #[derive(Default, Builder)]
//! struct MyAwesomeStruct {
//!     /// Name of the person
//!     name: Option<String>,
//!     /// Age of the person
//!     age: Option<u32>,
//! }
//! ```
//! This will generate the following builder methods:
//! ```
//! # struct MyAwesomeStruct {
//! #     name: Option<String>,
//! #     age: Option<u32>,
//! # }
//! impl MyAwesomeStruct {
//!    /// Name of the person
//!     pub fn name(mut self, name: String) -> Self {
//!         self.name = Some(name);
//!         self
//!     }
//!     /// Age of the person
//!     pub fn age(mut self, age: u32) -> Self {
//!         self.age = Some(age);
//!         self
//!     }
//! }
//!

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

type Fields = syn::punctuated::Punctuated<syn::Field, syn::token::Comma>;

#[proc_macro_derive(Builder, attributes(skip))]
/// Derive the builder pattern for a struct.
/// The builder implementation contains a method for each field of the struct, ignoring fields with
/// a #[skip] attribute.
/// Each field to generate a method for must be of type Option<T>.
///
/// # Example
///
/// The following struct:
/// ```
/// # use build_it::Builder;
/// #[derive(Builder)]
/// struct SimpleStruct {
///    name: Option<String>,
///    age: Option<u32>,
///    #[skip]
///    address: String,
/// }
/// ```
/// will generate the following implementation:
/// ```
/// # struct SimpleStruct {
/// #    name: Option<String>,
/// #    age: Option<u32>,
/// # }
/// impl SimpleStruct {
///    pub fn name(mut self, name: String) -> Self {
///        self.name = Some(name);
///        self
///     }
///     pub fn age(mut self, age: u32) -> Self {
///         self.age = Some(age);
///         self
///     }
/// }
/// ```
pub fn derive_builder(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let data = match input.data {
        syn::Data::Struct(ref data) => data,
        _ => panic!("Builder derive only works on structs"),
    };
    let fields = match data.fields {
        syn::Fields::Named(ref fields) => &fields.named,
        _ => panic!("Builder derive only works on structs with named fields"),
    };

    generate_builder_impl(&input, fields).into()
}

/// Generate the builder implementation for a struct.
/// The builder implementation contains a method for each field of the struct, ignoring fields with
/// a #[skip] attribute.
///
/// # Example
///
/// For a struct with fields `name: Option<String>` and `age: Option<u32>`, the generated
/// implementation is:
/// ```
/// # struct SimpleStruct {
/// #    name: Option<String>,
/// #    age: Option<u32>,
/// # }
/// impl SimpleStruct {
///    pub fn name(mut self, name: String) -> Self {
///        self.name = Some(name);
///        self
///     }
///     pub fn age(mut self, age: u32) -> Self {
///         self.age = Some(age);
///         self
///     }
/// }
/// ```
fn generate_builder_impl(input: &DeriveInput, fields: &Fields) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let methods = fields.iter().map(generate_builder_method);
    quote! {
        impl #generics #name #generics {
            #(#methods)*
        }
    }
}

/// Generate the builder method for a field.
/// The method has the same name as the field and takes the field type by value.
///
/// # Example
///
/// For a field `name: Option<String>`, the generated method is:
/// ```
/// # struct SimpleStruct {
/// #    name: Option<String>,
/// # }
/// # impl SimpleStruct {
/// pub fn name(mut self, name: String) -> Self {
///    self.name = Some(name);
///    self
/// }
/// # }
/// ```
fn generate_builder_method(field: &syn::Field) -> proc_macro2::TokenStream {
    // Skip fields with a #[skip] attribute
    if field.attrs.iter().any(|attr| attr.path().is_ident("skip")) {
        return quote! {};
    }

    let field_name = field.ident.as_ref().unwrap();
    let field_ty = get_inner_type(&field.ty).expect(
        "Builder only works on Option<T> fields.
        Consider using #[skip] to skip fields that should not be optional.",
    );
    let docs = field.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("doc") {
            Some(attr.clone())
        } else {
            None
        }
    });
    quote! {
        #docs
        pub fn #field_name(mut self, #field_name: #field_ty) -> Self {
            self.#field_name = Some(#field_name);
            self
        }
    }
}

/// Get the inner type of an Option<T> type.
fn get_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(ref type_path) = ty {
        if let Some(segment) = type_path.path.segments.first() {
            // Check if the type is an Option
            if segment.ident == "Option" {
                // Get the type inside the Option: the first generic argument
                if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if let Some(syn::GenericArgument::Type(ref ty)) = args.args.first() {
                        return Some(ty);
                    }
                }
            }
        }
    }
    None
}
