use quote::{ToTokens, quote};
use proc_macro::TokenStream;
use syn::{parse_quote, parse_macro_input, token, Ident, AttrStyle, Stmt, Lit};
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

    let asm_string = 
        r#"
        .section .nro_header
        .global __nro_header_start
        .word 0
        .word _mod_header
        .word 0
        .word 0

        .section .rodata.mod0
        .global _mod_header
        _mod_header:
            .ascii "MOD0"
            .word __dynamic_start - _mod_header
            .word __bss_start - _mod_header
            .word __bss_end - _mod_header
            .word __eh_frame_hdr_start - _mod_header
            .word __eh_frame_hdr_end - _mod_header
            .word __nx_module_runtime - _mod_header // runtime-generated module object offset
            
        .global IS_NRO
        IS_NRO:
            .word 1

        .section .bss.module_runtime
        __nx_module_runtime:
        .space 0xD0
        "#;

    let asm = syn::LitStr::new(asm_string, Span::call_site());

    if cfg!(feature = "nso") {
        main_function.attrs.push(
            syn::parse_quote!( #[export_name = "nnMain"] )
        );
    } else {
        main_function.attrs.push(
            syn::parse_quote!( #[export_name = "main"] )
        )
    }
    
    // extern "C"
    main_function.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern { span: Span::call_site() },
        name: Some(syn::LitStr::new("C", Span::call_site()))
    });

    let mut output = TokenStream2::new();

    quote! {
        #attr_code
        ::skyline::setup!();
        ::std::arch::global_asm!(#asm);
    }.to_tokens(&mut output);

    quote!(
        // this is both fine and normal and don't think too hard about it
        const _: fn() = || {
            use ::skyline::libc::{pthread_mutex_t, pthread_key_t, c_int, c_void};

            // re-export pthread_mutex_lock as __pthread_mutex_lock
            //
            // this is done in order to fix the fact that switch libstd depends on libc-nnsdk
            // which itself links against symbol aliases only present in certain versions of 
            // nnsdk.
            #[export_name = "__pthread_mutex_lock"]
            pub unsafe extern "C" fn _skyline_internal_pthread_mutex_lock_shim(lock: *mut pthread_mutex_t) -> c_int {
                extern "C" {
                    fn pthread_mutex_lock(lock: *mut pthread_mutex_t) -> c_int;
                }

                pthread_mutex_lock(lock)
            }

            #[export_name = "__pthread_key_create"]
            pub unsafe extern "C" fn _skyline_internal_pthread_key_create_shim(key: *mut pthread_key_t, func: extern fn(*mut c_void)) -> c_int {
                extern "C" {
                    fn pthread_key_create(
                        key: *mut pthread_key_t, func: extern fn(*mut c_void)
                    ) -> c_int;
                }

                pthread_key_create(key, func)
            }

            #[export_name = "__pthread_key_delete"]
            pub unsafe extern "C" fn _skyline_internal_pthread_key_delete_shim(key: pthread_key_t) -> c_int {
                extern "C" {
                    fn pthread_key_delete(
                        key: pthread_key_t
                    ) -> c_int;
                }

                pthread_key_delete(key)
            }
        };

        #output
        #main_function
    ).into()
}

fn remove_mut(arg: &syn::FnArg) -> syn::FnArg {
    let mut arg = arg.clone();

    if let syn::FnArg::Typed(ref mut arg) = arg {
        if let syn::Pat::Ident(ref mut arg) = *arg.pat {
            arg.by_ref = None;
            arg.mutability = None;
            arg.subpat = None;
        }
    }

    arg
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

    let args_tokens = mod_fn.sig.inputs.iter().map(remove_mut);
    let return_tokens = mod_fn.sig.output.to_token_stream();

    let _orig_fn = quote::format_ident!(
        "{}_skyline_internal_original_fn",
        mod_fn.sig.ident
    );

    // allow original!
    if !attrs.inline {
        let orig_stmt: Stmt = parse_quote! {
            #[allow(unused_macros)]
            macro_rules! original {
                () => {
                    {
                        // Hacky solution to allow `unused_unsafe` to be applied to an expression
                        #[allow(unused_unsafe)]
                        if true {
                            let temp = #_orig_fn.get().unwrap();

                            unsafe {
                                core::mem::transmute::<*const (), extern "C" fn(#(#args_tokens),*) #return_tokens>(
                                    *temp as *const u64 as *const()
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
        let orig_stmt: Stmt = parse_quote! {
            #[allow(unused_macros)] 
            macro_rules! call_original {
                ($($args:expr),* $(,)?) => {
                    original!()($($args),*)
                }
            }
        };
        mod_fn.block.stmts.insert(1, orig_stmt);
    }

    mod_fn.to_tokens(&mut output);

    let install_fn = install_fn::generate(&mod_fn.sig.ident, &_orig_fn, &attrs);

    if attrs.inline {
        install_fn.to_tokens(&mut output);
    } else {
        quote!(
            #install_fn
            
            #[allow(non_upper_case_globals)]
            pub static #_orig_fn: ::std::sync::OnceLock<u64> = ::std::sync::OnceLock::new();
        ).to_tokens(&mut output);
    }

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
    let offset = parse_macro_input!(attr as syn::Expr);

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

fn sig_to_token_func_call(sig: &syn::Signature) -> (Ident, TokenStream2) {
    let ident = quote::format_ident!("__{}_internal_unchecked", sig.ident);
    let args: Vec<_> =
        sig.inputs
            .iter()
            .map(|fn_arg|{
                if let syn::FnArg::Typed(pat) = fn_arg {
                    pat.pat.to_token_stream()
                } else {
                    todo!()
                }
            })
            .collect();

    (
        ident.clone(),
        quote!(
            #ident(
                #(
                    #args
                ),*
            )
        )
    )
}

/// Add a null check to dynamically linked functions. Applied at the extern block level.
///
/// Example:
///
/// ```rust
/// #[null_check]
/// extern "C" {
///     fn not_an_available_import() -> u64;
/// }
/// ```
///
/// Then, if `not_an_available_import` is not available it will panic with the following message:
///
/// ```text
/// thread '<unnamed>' panicked at 'not_an_available_import is null (likely unlinked)', src/lib.rs:5:1
/// ```
///
/// # Note
///
/// Due to a bug, this may not consistently panic on release builds, use `--debug` for install/run
/// commands to ensure this does not happen when testing.
#[proc_macro_attribute]
pub fn null_check(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut extern_block = parse_macro_input!(input as syn::ItemForeignMod);

    let (vis, sigs): (Vec<_>, Vec<_>) =
        extern_block
            .items
            .iter_mut()
            .filter_map(|item|{
                if let syn::ForeignItem::Fn(ref mut func) = item {
                    let has_link_name = func.attrs.iter().any(|attr|{
                        if let Some(ident) = attr.path.get_ident() {
                            ident.to_string() == "link_name"
                        } else {
                            false
                        }
                    });

                    if !has_link_name {
                        let name = func.sig.ident.to_string();

                        let attr: syn::Attribute = parse_quote!(
                            #[link_name = #name]
                        );

                        func.attrs.push(attr);
                    }

                    let old_sig = func.sig.clone();

                    func.sig.ident = quote::format_ident!("__{}_internal_unchecked", func.sig.ident);

                    Some((func.vis.clone(), old_sig))
                } else {
                    None
                }
            })
            .unzip();

    let (idents, func_calls): (Vec<_>, Vec<_>) = sigs.iter().map(sig_to_token_func_call).unzip();

    quote!(
        #(
            #vis unsafe #sigs {
                //panic!(concat!(stringify!(#idents), " is 0x{:X}"), (#idents as u64));
                
                println!("ptr is 0x{:X}", #idents as u64);

                if (#idents as u64 as *const ()).is_null() {
                    return panic!(concat!(stringify!(#idents), " is null (likely unlinked)"));
                }

                #func_calls
            }
        )*

        #extern_block
    ).into()
}
