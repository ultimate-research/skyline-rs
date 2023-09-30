use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, token, Token};

pub struct MainAttrs {
    pub name: String,
}

mod kw {
    syn::custom_keyword!(inline);
    syn::custom_keyword!(name);
    syn::custom_keyword!(replace);
    syn::custom_keyword!(symbol);
    syn::custom_keyword!(pointer_offset);
    syn::custom_keyword!(offset);
}

impl Parse for MainAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(kw::name) {
            let meta: syn::MetaNameValue = input.parse()?;

            match meta.lit {
                syn::Lit::Str(string) => Ok(MainAttrs {
                    name: string.value(),
                }),
                _ => panic!("Invalid literal, must be a string"),
            }
        } else {
            Ok(MainAttrs {
                name: "skyline_rust_plugin".into(),
            })
        }
    }
}

impl ToTokens for MainAttrs {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name[..];
        quote::quote!(
            ::skyline::set_module_name!(#name);
        )
        .to_tokens(tokens);
    }
}

#[derive(Default, Debug)]
pub struct HookAttrs {
    pub replace: Option<syn::Path>,
    pub symbol: Option<syn::LitStr>,
    pub pointer_offset: Option<syn::Expr>,
    pub offset: Option<syn::Expr>,
    pub inline: bool,
}

fn merge(attr1: HookAttrs, attr2: HookAttrs) -> HookAttrs {
    let (
        HookAttrs {
            replace: r1,
            symbol: s1,
            pointer_offset: so1,
            offset: o1,
            inline: i1,
        },
        HookAttrs {
            replace: r2,
            symbol: s2,
            pointer_offset: so2,
            offset: o2,
            inline: i2,
        },
    ) = (attr1, attr2);

    HookAttrs {
        replace: r1.or(r2),
        offset: o1.or(o2),
        symbol: s1.or(s2),
        pointer_offset: so1.or(so2),
        inline: i1 || i2,
    }
}

impl Parse for HookAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let look = input.lookahead1();
        let attr = if look.peek(kw::symbol) {
            let MetaItem::<kw::symbol, syn::LitStr> { item: string, .. } = input.parse()?;

            let mut a = HookAttrs::default();
            a.symbol = Some(string);
            a
        } else if look.peek(kw::offset) {
            let MetaItem::<kw::offset, syn::Expr> { item: offset, .. } = input.parse()?;

            let mut a = HookAttrs::default();
            a.offset = Some(offset);
            a
        } else if look.peek(kw::pointer_offset) {
            let MetaItem::<kw::pointer_offset, syn::Expr> {
                item: pointer_offset,
                ..
            } = input.parse()?;

            let mut a = HookAttrs::default();
            a.pointer_offset = Some(pointer_offset);
            a
        } else if look.peek(kw::replace) {
            let MetaItem::<kw::replace, syn::Path> { item: replace, .. } = input.parse()?;

            let mut a = HookAttrs::default();
            a.replace = Some(replace);
            a
        } else if look.peek(kw::inline) {
            let _: kw::inline = input.parse()?;
            let mut a = HookAttrs::default();
            a.inline = true;
            a
        } else {
            return Err(look.error());
        };

        Ok(if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.is_empty() {
                attr
            } else {
                merge(attr, input.parse()?)
            }
        } else {
            attr
        })
    }
}

#[derive(Debug, Clone)]
pub struct MetaItem<Keyword: Parse, Item: Parse> {
    pub ident: Keyword,
    pub item: Item,
}

impl<Keyword: Parse, Item: Parse> Parse for MetaItem<Keyword, Item> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let item = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            content.parse()?
        } else {
            input.parse::<Token![=]>()?;
            input.parse()?
        };

        Ok(Self { ident, item })
    }
}
