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
//!     #[build_it(skip)]
//!     address: String,
//!
//!     /// The `#[build_it(rename = "new_name")]` attribute can be used to rename the builder
//!     /// method. In this case, the builder method will be called `new_name` instead of
//!     /// `renamed`:
//!     /// `let builder = MyAwesomeStruct::default().new_name("Alice".to_string());`
//!     #[build_it(rename = "new_name")]
//!     renamed: Option<String>,
//!
//!     /// The `#[build_it(into)]` attribute can be used to allow the builder method to accept
//!     /// types that can be converted into the field type. In this case, the builder method will
//!     /// accept a `&str` instead of a `String`:
//!     /// `let builder = MyAwesomeStruct::default().name_into("Alice");`
//!     #[build_it(into)]
//!     name_into: Option<String>,
//!
//!     #[build_it(skip)]
//!     // NOTE: While the `#[skip]` attribute is still supported, it is deprecated in favor of
//!     // the `#[build_it(skip)]` attribute.
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
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

type Fields = syn::punctuated::Punctuated<syn::Field, syn::token::Comma>;

#[proc_macro_derive(Builder, attributes(build_it, skip))]
/// Derive the builder pattern for a struct.
///
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
///    #[build_it(skip)]
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
    let global_attr = parse_global_attr(&input);
    let data = match input.data {
        syn::Data::Struct(ref data) => Ok(data),
        syn::Data::Enum(ref data) => Err(syn::Error::new(
            data.enum_token.span(),
            "Builder derive does not work on enums",
        )),
        syn::Data::Union(ref data) => Err(syn::Error::new(
            data.union_token.span(),
            "Builder derive does not work on unions",
        )),
    };
    if let Err(err) = data {
        return err.to_compile_error().into();
    }
    let data = data.expect("data is a struct");

    let fields = match data.fields {
        syn::Fields::Named(ref fields) => &fields.named,
        syn::Fields::Unit => return quote! {}.into(),
        syn::Fields::Unnamed(ref fields) => {
            return syn::Error::new(
                fields.span(),
                "Builder derive only works on structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    generate_builder_impl(&input, &global_attr, fields).into()
}

/// Generate the builder implementation for a struct.
/// The builder implementation contains a method for each field of the struct, ignoring fields with
/// a #[build_it(skip)] attribute.
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
fn generate_builder_impl(
    input: &DeriveInput,
    global_attr: &GlobalAttr,
    fields: &Fields,
) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let methods = fields
        .iter()
        .map(|f| generate_builder_method(f, global_attr));
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
fn generate_builder_method(
    field: &syn::Field,
    global_attr: &GlobalAttr,
) -> proc_macro2::TokenStream {
    // Skip fields with a #[skip] attribute
    // NOTE: This is deprecated in favor of the `build_it` attribute
    if field.attrs.iter().any(|attr| attr.path().is_ident("skip")) {
        return quote! {};
    }

    let attr = parse_attr(field);
    if attr.skip {
        return quote! {};
    }

    let field_name = field.ident.as_ref().unwrap();
    let fn_name = syn::Ident::new(
        &attr.rename.unwrap_or(field_name.to_string()),
        Span::call_site(),
    );
    let field_ty = get_inner_type(&field.ty);
    if field_ty.is_none() {
        return syn::Error::new(
            field.span(),
            "Builder only works on Option<T> fields. Consider using #[skip] to skip fields that should not be optional.",
        )
        .to_compile_error();
    }
    let field_ty = field_ty.expect("field type is an Option<T>");

    let docs = field.attrs.iter().filter_map(|attr| {
        if attr.path().is_ident("doc") {
            Some(attr.clone())
        } else {
            None
        }
    });
    if attr.into || global_attr.into {
        quote! {
            #(#docs)*
            pub fn #fn_name(mut self, #field_name: impl core::convert::Into<#field_ty>) -> Self {
                self.#field_name = Some(#field_name.into());
                self
            }
        }
    } else {
        quote! {
            #(#docs)*
            pub fn #fn_name(mut self, #field_name: #field_ty) -> Self {
                self.#field_name = Some(#field_name);
                self
            }
        }
    }
}

#[derive(Default)]
struct GlobalAttr {
    into: bool,
}

fn parse_global_attr(input: &DeriveInput) -> GlobalAttr {
    let mut result = GlobalAttr::default();
    let attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("build_it"));
    if let Some(attr) = attr {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("into") {
                result.into = true;
            }
            Ok(())
        })
        .expect("Failed to parse global build_it attribute");
    }
    result
}

#[derive(Default)]
struct Attr {
    skip: bool,
    into: bool,
    rename: Option<String>,
}

fn parse_attr(field: &syn::Field) -> Attr {
    let attr = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("build_it"));
    let mut result = Attr::default();
    if let Some(attr) = attr {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                result.skip = true;
            } else if meta.path.is_ident("into") {
                result.into = true;
            } else if meta.path.is_ident("rename") {
                let content = meta.value().expect("Expected a value");
                let lit: syn::LitStr = content.parse()?;
                result.rename = Some(lit.value());
            }
            Ok(())
        })
        .expect("Failed to parse build_it attribute");
    }
    result
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
