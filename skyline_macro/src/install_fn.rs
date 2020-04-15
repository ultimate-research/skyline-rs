use quote::{quote, quote_spanned, ToTokens};
use super::attributes::HookAttrs;
use proc_macro2::Span;

pub fn generate(name: &syn::Ident, orig: &syn::Ident, attrs: &HookAttrs) -> impl ToTokens {
    let _install_fn = quote::format_ident!("{}_skyline_internal_install_hook", name);

    let replace = attrs.replace
                    .as_ref()
                    .map(ToTokens::into_token_stream)
                    .unwrap_or_else(||{
                        quote_spanned!(Span::call_site() =>
                            compile_error!("Missing 'replace' item in hook macro");
                        )
                    });

    quote!{
        fn #_install_fn() {
            if (::skyline::hooks::A64HookFunction as *const ()).is_null() {
                panic!("A64HookFunction is null");
            }

            unsafe {
                ::skyline::hooks::A64HookFunction(
                    #replace as *const ::skyline::libc::c_void,
                    #name as *const ::skyline::libc::c_void,
                    &mut #orig as *mut *mut ::skyline::libc::c_void 
                )
            }
        }
    }
}
