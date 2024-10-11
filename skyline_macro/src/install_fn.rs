use quote::{quote, quote_spanned, ToTokens};
use super::attributes::HookAttrs;
use proc_macro2::Span;

pub fn generate(name: &syn::Ident, orig: &syn::Ident, attrs: &HookAttrs) -> impl ToTokens {
    let _install_fn = quote::format_ident!("{}_skyline_internal_install_hook", name);
    let pointer_offset = attrs
                    .pointer_offset
                    .as_ref()
                    .map(ToTokens::into_token_stream)
                    .unwrap_or(quote! {0});

    let replace = attrs.replace
                    .as_ref()
                    .map(ToTokens::into_token_stream)
                    .or_else(||{
                        attrs.offset.as_ref().map(|offset|{
                            quote! {
                                unsafe {
                                    ::skyline::hooks::getRegionAddress(
                                        ::skyline::hooks::Region::Text
                                    ) as *mut u8
                                }.add(#offset)
                            }
                        })
                    })
                    .unwrap_or_else(||{
                        quote_spanned!(Span::call_site() =>
                            compile_error!("Missing 'replace' item in hook macro");
                        )
                    });

    if attrs.inline {
        quote!{
            const _: fn() = ||{
                trait InlineCtxRef {}

                impl InlineCtxRef for &::skyline::hooks::InlineCtx {}
                impl InlineCtxRef for &mut ::skyline::hooks::InlineCtx {}

                fn assert_inline_ctx<T: InlineCtxRef>(_: unsafe extern "C" fn(T)) {}

                assert_inline_ctx(#name);
            };
            pub fn #_install_fn() {
                if (::skyline::hooks::A64InlineHook as *const ()).is_null() {
                    panic!("A64InlineHook is null");
                }

                unsafe {
                    ::skyline::hooks::A64InlineHook(
                        ((#replace as *const u8).offset(#pointer_offset) as *const ::skyline::libc::c_void),
                        #name as *const ::skyline::libc::c_void,
                    )
                }
            }
        }
    } else {
        quote!{
            pub fn #_install_fn() {
                if (::skyline::hooks::A64HookFunction as *const ()).is_null() {
                    panic!("A64HookFunction is null");
                }

                unsafe {
                    #[allow(static_mut_refs)]
                    ::skyline::hooks::A64HookFunction(
                        ((#replace as *const u8).offset(#pointer_offset) as *const ::skyline::libc::c_void),
                        #name as *const ::skyline::libc::c_void,
                        &mut #orig as *mut *mut ::skyline::libc::c_void 
                    )
                }
            }
        }
    }
}
