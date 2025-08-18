use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(SerializableComponent)]
pub fn derive_serializable_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    let expanded = quote! {
        impl SerializableComponent for #name {
            fn to_serializable(&self) -> Box<dyn crate::engine::SerializableValue> {
                Box::new(self.clone())
            }

            fn from_serializable(value: &dyn crate::engine::SerializableValue, cmds: &mut EntityCommands) {
                if let Some(component) = value.as_any().downcast_ref::<#name>() {
                    cmds.insert(component.clone());
                }
            }

            fn type_name() -> &'static str {
                #name_str
            }
        }
    };

    TokenStream::from(expanded)
}
