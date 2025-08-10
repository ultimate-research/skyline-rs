use core::fmt;
use crate::nn::ro;

pub type Callback = extern "Rust" fn(&NroInfo);

/// An error representing the NRO hook plugin not successfully being linked against
#[derive(Debug, Clone, Copy)]
pub struct NroHookPluginMissing;

#[allow(improper_ctypes)]
extern "C" {
    fn add_nro_load_hook(callback: Callback);
    fn add_nro_unload_hook(callback: Callback);
}

/// A function to allow adding a hook for immediately after an NRO has been loaded.
///
/// **Note:** Requires the NRO hook plugin. Will return an error otherwise.
pub fn add_hook(callback: Callback) -> Result<(), NroHookPluginMissing> {
    // Removed the null check on the function because function pointers are not nullable. Should probably get them through the nnsdk symbol resolver for this purpose.
        unsafe { add_nro_load_hook(callback); }
        Ok(())

}

pub fn add_unload_hook(callback: Callback) -> Result<(), NroHookPluginMissing> {
    unsafe { add_nro_unload_hook(callback); }
    Ok(())
}

/// A struct to hold information related to the NRO being loaded
#[repr(C)]
#[non_exhaustive]
pub struct NroInfo<'a> {
    pub name: &'a str,
    pub module: &'a mut ro::Module
}

impl<'a> NroInfo<'a> {
    /// A function only to be used by nro_hook in order to construct NroInfos. Since fields may be
    /// added, this API is subject to change.
    #[cfg(feature = "nro_internal")]
    pub fn new(name: &'a str, module: &'a mut ro::Module) -> Self {
        Self { name, module }
    }
}

impl fmt::Display for NroHookPluginMissing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The NRO hook plugin could not be found and is required to add NRO hooks. Make sure hook_nro.nro is installed.")
    }
}
