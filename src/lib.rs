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
/// # use simple_builder::Builder;
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
