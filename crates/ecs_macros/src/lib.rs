//! Procedural macros for the ECS crate.
//!
//! Provides `#[derive(ComponentMeta)]` for automatic component metadata generation.

use proc_macro::TokenStream;

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
    // Implementation in next task
    TokenStream::new()
}
