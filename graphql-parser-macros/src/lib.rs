use proc_macro::TokenStream;
use proc_macro::TokenTree;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput};

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
        let arg = args[1].to_string();
        let field = syn::Ident::new(&arg, arg.span());
        quote! {
            impl #impl_generics Name<'a> for #name #ty_generics #where_clause {
                fn name(&self) -> Option<&'a str> { Some(self.#field) }
            }
        }
    } else if args.len() == 1 && args[0].to_string() == "enum" {
        derive_name_on_enum(ast)
    } else {
        panic!("invalid use of derive_name with args: {:?}", args);
    };

    tokens.extend(TokenStream::from(output));
    TokenStream::from(tokens)
}

fn derive_name_on_enum(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let variants = match ast.data {
        syn::Data::Enum(variants) => variants,
        _ => panic!("#[derive_name(enum)] can only be used with enums"),
    };
    let variants = variants.variants.iter();

    let variants = variants.map(|&syn::Variant { ref ident, .. }| {
        quote! {
            #name :: #ident(d) => Some(d.name),
        }
    });

    quote! {
        impl #impl_generics Name<'a> for #name #ty_generics #where_clause {
            fn name(&self) -> Option<&'a str> {
                match self {
                    #(
                        #variants
                    )*
                }
             }
        }
    }
}
