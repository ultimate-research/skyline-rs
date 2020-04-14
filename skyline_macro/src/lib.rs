#![feature(concat_idents)]

use quote::{ToTokens, quote};
use proc_macro::TokenStream;
use syn::{parse_quote, parse_macro_input, token, Ident, AttrStyle, Stmt, Lit, spanned::Spanned};
use proc_macro2::{Span, TokenStream as TokenStream2};

mod attributes;

fn new_attr(attr_name: &str) -> syn::Attribute {
    syn::Attribute {
        pound_token: token::Pound { spans: [Span::call_site()] },
        style: AttrStyle::Outer,
        bracket_token: token::Bracket { span: Span::call_site() },
        path: Ident::new(attr_name, Span::call_site()).into(),
        tokens: TokenStream2::new()
    }
}

#[proc_macro_attribute]
pub fn main(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut main_function = parse_macro_input!(item as syn::ItemFn);

    let attr_code = parse_macro_input!(attrs as attributes::Attrs);

    // #[no_mangle]
    main_function.attrs.push(
        new_attr("no_mangle")
    );
    
    // extern "C"
    main_function.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern { span: Span::call_site() },
        name: Some(syn::LitStr::new("C", Span::call_site()))
    });
    
    main_function.sig.ident = Ident::new("main", Span::call_site());

    // allow hook!
    let hook_stmt: Stmt = parse_quote! {
        macro_rules! hook {
            ($symbol:ident, $replace:ident) => { 
                hook(
                    $symbol as *const libc::c_void,
                    $replace as *const libc::c_void,
                    unsafe { &mut concat_idents!(orig_, $replace) as *mut *mut libc::c_void })
            }
        }
    };
    main_function.block.stmts.insert(0, hook_stmt);

    let mut output = TokenStream2::new();

    quote!(
        #attr_code
        //use skyline::prelude::*;
        ::skyline::setup!();
    ).to_tokens(&mut output);
    main_function.to_tokens(&mut output);

    output.into()
}

#[proc_macro_attribute]
pub fn hook(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut mod_fn = parse_macro_input!(input as syn::ItemFn);
    let mut output = TokenStream2::new();

    // #[no_mangle]
    mod_fn.attrs.push(
        new_attr("no_mangle")
    );

    // extern "C"
    mod_fn.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern { span: Span::call_site() },
        name: Some(syn::LitStr::new("C", Span::call_site()))
    });

    let args_tokens = mod_fn.sig.inputs.to_token_stream();
    let return_tokens = mod_fn.sig.output.to_token_stream();

    let _orig_fn = quote::format_ident!(
        "orig_{}",
        mod_fn.sig.ident
    );

    // allow original!
    let orig_stmt: Stmt = parse_quote! {
        macro_rules! original {
            () => { unsafe { core::mem::transmute::<_, extern "C" fn(#args_tokens) #return_tokens>(#_orig_fn as *const()) } }
        }
    };
    mod_fn.block.stmts.insert(0, orig_stmt);

    mod_fn.to_tokens(&mut output);

    let mod_fn = mod_fn.sig.ident;

    quote!(
        #[allow(non_upper_case_globals)]
        static mut #_orig_fn: *mut libc::c_void = 0 as *mut libc::c_void;
    ).to_tokens(&mut output);

    output.into()
}

fn lit_to_bytes(lit: &Lit) -> Option<Vec<u8>> {
    match lit {
        Lit::Str(lit_str) => {
            Some(lit_str.value().into_bytes())
        }
        Lit::ByteStr(lit_str) => {
            Some(lit_str.value())
        }
        _ => {
            None
        }
    }
}

#[proc_macro]
pub fn to_null_term_bytes(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Lit);

    match lit_to_bytes(&expr) {
        Some(mut bytes) => {
            bytes.push(0);

            let bytes = syn::LitByteStr::new(&bytes, expr.span());

            TokenStream::from(quote! {
                (#bytes)
            })
        }
        None => {
            let span = expr.span();
            TokenStream::from(quote::quote_spanned!{span =>
                compile_error!("Invalid literal");
            })
        }
    }
}
