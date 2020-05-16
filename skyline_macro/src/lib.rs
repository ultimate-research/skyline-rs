use quote::{ToTokens, quote};
use proc_macro::TokenStream;
use syn::{parse_quote, parse_macro_input, token, Ident, AttrStyle, Stmt, Lit, spanned::Spanned};
use syn::{punctuated::Punctuated, FnArg, BareFnArg, token::Comma};
use proc_macro2::{Span, TokenStream as TokenStream2};

mod attributes;
mod install_fn;

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

    let attr_code = parse_macro_input!(attrs as attributes::MainAttrs);

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

    let mut output = TokenStream2::new();

    quote!(
        #attr_code
        ::skyline::setup!();
    ).to_tokens(&mut output);
    main_function.to_tokens(&mut output);

    output.into()
}

#[proc_macro_attribute]
pub fn hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut mod_fn = parse_macro_input!(input as syn::ItemFn);
    let attrs = parse_macro_input!(attrs as attributes::HookAttrs);
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
        "{}_skyline_internal_original_fn",
        mod_fn.sig.ident
    );

    // allow original!
    let orig_stmt: Stmt = parse_quote! {
        macro_rules! original {
            () => {
                {
                    // Hacky solution to allow `unused_unsafe` to be applied to an expression
                    #[allow(unused_unsafe)]
                    if true {
                        unsafe {
                            core::mem::transmute::<_, extern "C" fn(#args_tokens) #return_tokens>(
                                #_orig_fn as *const()
                            ) 
                        } 
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    };
    mod_fn.block.stmts.insert(0, orig_stmt);

    mod_fn.to_tokens(&mut output);

    let install_fn = install_fn::generate(&mod_fn.sig.ident, &_orig_fn, &attrs);

    quote!(
        #install_fn
        
        #[allow(non_upper_case_globals)]
        pub static mut #_orig_fn: *mut ::skyline::libc::c_void = 0 as *mut ::skyline::libc::c_void;
    ).to_tokens(&mut output);

    output.into()
}

#[proc_macro]
pub fn install_hook(input: TokenStream) -> TokenStream {
    let mut path = parse_macro_input!(input as syn::Path);

    let last_seg = path.segments.iter_mut().last().unwrap();

    last_seg.ident = quote::format_ident!("{}_skyline_internal_install_hook", last_seg.ident);

    quote!(
        #path();
    ).into()
}

fn into_bare_args(args: &Punctuated<FnArg, Comma>) -> Punctuated<BareFnArg, Comma> {
    args.iter()
        .map(|arg|{
            if let FnArg::Typed(pat_type) = arg {
                BareFnArg {
                    attrs: pat_type.attrs.clone(),
                    name: None,
                    ty: (*pat_type.ty).clone()
                }
            } else {
                todo!()
            }
        })
        .collect()
}

fn get_arg_pats(args: &Punctuated<FnArg, Comma>) -> Punctuated<syn::Pat, Comma> {
    args.iter()
        .map(|arg|{
            if let FnArg::Typed(pat_type) = arg {
                (*pat_type.pat).clone()
            } else {
                todo!()
            }
        })
        .collect()
}

#[proc_macro_attribute]
pub fn from_offset(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut fn_sig = parse_macro_input!(input as syn::ForeignItemFn);
    let offset = parse_macro_input!(attr as syn::LitInt);

    let mut inner_fn_type: syn::TypeBareFn = parse_quote!( extern "C" fn() );

    inner_fn_type.output = fn_sig.sig.output.clone();
    inner_fn_type.variadic = fn_sig.sig.variadic.clone();
    inner_fn_type.inputs = into_bare_args(&fn_sig.sig.inputs);

    let visibility = fn_sig.vis;
    fn_sig.sig.unsafety = Some(syn::token::Unsafe { span: Span::call_site() });

    let sig = fn_sig.sig;
    let args = get_arg_pats(&sig.inputs);

    // Generate a shim for the function at the offset
    quote!(
        #visibility #sig {
            let inner = core::mem::transmute::<_,#inner_fn_type>(
                unsafe {::skyline::hooks::getRegionAddress(
                    ::skyline::hooks::Region::Text
                ) as *const u8}.offset(#offset as isize)
            );
            inner(
                #args
            )
        }
    ).into()
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
