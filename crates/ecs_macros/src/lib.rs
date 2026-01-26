//! Procedural macros for the ECS crate.
//!
//! Provides `#[derive(ComponentMeta)]` for automatic component metadata generation.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive macro for ComponentMeta trait.
///
/// Generates `type_name()` and `field_names()` implementations automatically.
///
/// # Example
/// ```ignore
/// use ecs_macros::ComponentMeta;
///
/// #[derive(ComponentMeta)]
/// pub struct Health {
///     pub value: f32,
///     pub max: f32,
/// }
///
/// assert_eq!(Health::type_name(), "Health");
/// assert_eq!(Health::field_names(), &["value", "max"]);
/// ```
#[proc_macro_derive(ComponentMeta)]
pub fn derive_component_meta(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    // Extract field names from struct
    let field_names: Vec<String> = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
                .collect(),
            Fields::Unnamed(_) => {
                // Tuple structs get no field names
                vec![]
            }
            Fields::Unit => vec![],
        },
        Data::Enum(_) | Data::Union(_) => {
            return syn::Error::new_spanned(
                &input.ident,
                "ComponentMeta can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let field_name_strs: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();

    let expanded = quote! {
        impl ComponentMeta for #name {
            fn type_name() -> &'static str {
                #name_str
            }

            fn field_names() -> &'static [&'static str] {
                &[ #( #field_name_strs ),* ]
            }
        }
    };

    TokenStream::from(expanded)
}
