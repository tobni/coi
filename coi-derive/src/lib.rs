extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Data, DeriveInput, Error, Expr, Fields, Result, Type, Visibility,
};

struct Provides {
    vis: Visibility,
    ty: Type,
    with: Expr,
}

impl Parse for Provides {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = input.parse()?;
        let ty = input.parse()?;
        input.parse().and_then(|ident: Ident| {
            if ident.eq("with") {
                Ok(())
            } else {
                Err(Error::new(ident.span(), "expected `with`"))
            }
        })?;
        // FIXME(pfaria) we need to limit the kinds of exprs allowed here. Quite a few will
        // fail to compile
        let with = input.parse()?;
        Ok(Provides { vis, ty, with })
    }
}

#[proc_macro_derive(Inject, attributes(provides, inject))]
pub fn inject_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data_struct = match input.data {
        Data::Struct(data_struct) => data_struct,
        _ => {
            return Error::new_spanned(input, "#[derive(Inject)] only supports structs")
                .to_compile_error()
                .into()
        }
    };
    let provider = format_ident!("{}Provider", input.ident);
    let attr = match input
        .attrs
        .into_iter()
        .filter(|attr| {
            attr.path
                .segments
                .first()
                .map(|p| p.ident.eq("provides"))
                .unwrap_or(false)
        })
        .next()
    {
        None => {
            return Error::new_spanned(
                input.ident,
                "#[derive(Inject)] requires a `provides` attribute",
            )
            .to_compile_error()
            .into()
        }
        Some(attr) => attr,
    };

    let (arg_ident, arg_type): (Vec<Ident>, Vec<Type>) = match data_struct.fields {
        Fields::Named(named_fields) => named_fields
            .named
            .into_iter()
            .map(|field| (field.ident.unwrap(), field.ty))
            .unzip(),
        // FIXME(pfaria) add support for unnamed fields by allowing the name to be
        // specified as part of the attribute params
        Fields::Unnamed(_) => {
            return Error::new_spanned(data_struct.fields, "unnamed fields cannot be injected")
                .to_compile_error()
                .into()
        }
        Fields::Unit => (vec![], vec![]),
    };

    let attr2 = attr.clone();
    let mut token_iter = attr2.tokens.into_iter();
    let provides = match token_iter.next() {
        Some(TokenTree::Group(group)) => {
            let token_stream = TokenStream::from(group.stream());
            parse_macro_input!(token_stream as Provides)
        }
        Some(s) => {
            return Error::new_spanned(s, "expected `(ty with expr)`")
                .to_compile_error()
                .into()
        }
        _ => {
            return Error::new_spanned(attr, "expected `(ty with expr)`")
                .to_compile_error()
                .into()
        }
    };
    let vis = provides.vis;
    let ty = provides.ty;
    let provides_with = provides.with;

    if let Some(s) = token_iter.next() {
        return Error::new_spanned(
            s,
            "Only expected the format #[provides `Interface` with `provider`]",
        )
        .to_compile_error()
        .into();
    }

    let arg_key: Vec<String> = arg_ident.iter().map(|i| i.to_string()).collect();
    let container = format_ident!(
        "{}",
        if arg_ident.len() == 0 {
            "_"
        } else {
            "container"
        }
    );
    let input_ident = input.ident;
    let expanded = quote! {
        impl Inject for #input_ident {}

        #vis struct #provider;

        #[async_trait::async_trait]
        impl ::coi::Provide for #provider {
            type Output = ::std::sync::Arc<#ty>;

            async fn provide(&self, #container: &::coi::Container) -> ::coi::Result<Self::Output> {
                #( let #arg_ident = #container.resolve::<#arg_type>(#arg_key).await?; )*
                Ok(::std::sync::Arc::new(#provides_with) as ::std::sync::Arc<#ty>)
            }
        }
    };
    TokenStream::from(expanded)
}
