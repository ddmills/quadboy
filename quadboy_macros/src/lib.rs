use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemFn, parse_macro_input};

/// Derive macro for SerializableComponent trait
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

/// Attribute macro that automatically adds Tracy profiling spans to system functions
/// Zero overhead when tracy feature is disabled
#[proc_macro_attribute]
pub fn profiled_system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;

    let output = quote! {
        #(#attrs)*
        #vis #sig {
            #[cfg(feature = "tracy")]
            let _tracy_span = tracy_client::span!(#fn_name_str);

            #block
        }
    };

    TokenStream::from(output)
}