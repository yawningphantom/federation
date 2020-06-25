use proc_macro::TokenStream;
use proc_macro::TokenTree;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use syn::spanned::Spanned;

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

#[proc_macro_attribute]
pub fn derive_name(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut tokens = input.clone();
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let args: Vec<TokenTree> = args.into_iter().collect();

    let output = if args.is_empty() {
        quote! {
            impl #impl_generics Name<'a> for #name #ty_generics #where_clause {}
        }
    } else if args[0].to_string() == "." && args.len() == 2 {
        // let field = format_ident!("{}", args[1]);
        let arg = args[1].to_string();
        let field = syn::Ident::new(&arg, arg.span());
        quote! {
            impl #impl_generics Name<'a> for #name #ty_generics #where_clause {
                fn name(&self) -> Option<&'a str> { Some(self.#field) }
            }
        }
    } else {
        panic!("invalid use of derive_name with args: {:?}", args);
    };

    // let arg = args.into_iter().map(|t| t.to_string()).collect::<String>();

    tokens.extend(TokenStream::from(output));
    TokenStream::from(tokens)
}
