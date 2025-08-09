#![cfg_attr(not(feature = "std"), no_std)]
#![allow(incomplete_features, stable_features)]
#![feature(
    alloc_error_handler,
    lang_items,
    global_asm,
    proc_macro_hygiene,
    panic_info_message,
    track_caller
)]

#[cfg(feature = "std")]
use std::str::Utf8Error;

use libc::strlen;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, string::String};

#[cfg(feature = "std")]
pub mod unix_alloc;

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

/// Functions for iterating through a binary .text section
pub mod text_iter;

/// Types and helpers related to error-handling
pub mod error;

/// Types and functions needed to handle NRO hooking
pub mod nro;

pub mod info;

#[doc(hidden)]
pub mod extern_alloc;

#[doc(hidden)]
pub mod build;

// nnsdk API bindings
pub mod nn;

#[doc(inline)]
pub use {
    error::{Error, ErrorKind},
    hooks::iter_hooks,
    libc,
    skyline_macro::{from_offset, hook, install_hook, main, null_check},
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

#[cfg(feature = "std")]
/// Helper to convert a C-str to a Rust string, returning an error if it failed
pub unsafe fn try_from_c_str(c_str: *const u8) -> Result<String, Utf8Error> {
    let name_slice = core::slice::from_raw_parts(c_str as *mut _, strlen(c_str));
    core::str::from_utf8(&name_slice).map(|string| string.to_owned())
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
}
