use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Name)]
pub fn derive_simple_name(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let output = quote! {
        impl #impl_generics Name<'a> for #name #ty_generics #where_clause {
            fn name(&self) -> Option<&'a str> { Some(self.name) }
        }
    };

    proc_macro::TokenStream::from(output)
}
