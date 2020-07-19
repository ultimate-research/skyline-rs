#![cfg_attr(not(feature = "std"), no_std)]
#![allow(incomplete_features)]
#![feature(alloc_error_handler, lang_items, start, global_asm, const_generics, impl_trait_in_bindings, proc_macro_hygiene, alloc_prelude, panic_info_message, try_trait, track_caller)]

use libc::strlen;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, string::String};

/// The rust core allocation and collections library
pub extern crate alloc;

#[doc(hidden)]
pub use skyline_macro;

/// Types and functions for working with hooking
pub mod hooks;

/// Types and functions for logging and debugging
pub mod logging;

/// Functions for helping patch executables
pub mod patching;

/// Types and helpers related to error-handling
pub mod error;

/// Types and functions needed to handle NRO hooking
pub mod nro;

#[doc(hidden)]
pub mod extern_alloc;

#[doc(hidden)]
pub mod build;

// nnsdk API bindings
pub mod nn;

#[doc(inline)]
pub use {
    libc,
    skyline_macro::{main, hook, install_hook, from_offset, null_check}, 
    hooks::iter_hooks,
    error::{Error, ErrorKind},
};

/// Helper to convert a str to a *const u8 (to be replaced)
pub fn c_str(string: &str) -> *const u8 {
    string.as_bytes().as_ptr()
}


/// Helper to convert a C-str to a Rust string
pub unsafe fn from_c_str(c_str: *const u8) -> String {
    let name_slice = core::slice::from_raw_parts(c_str as *mut _, strlen(c_str));
    match core::str::from_utf8(&name_slice) {
        Ok(v) => v.to_owned(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

/// A set of items that will likely be useful to import anyway
///
/// Designed to be used as such:
/// ```
/// use skyline::prelude::*;
/// ```
pub mod prelude {
    pub use crate::println;
    pub use alloc::format;
    pub use alloc::vec;
    pub use crate::alloc::prelude::v1::*;
}
