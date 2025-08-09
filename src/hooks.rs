use crate::alloc::string::String;
use core::fmt;
use nnsdk::root::nn;

#[macro_export]
macro_rules! install_hooks {
    (
        $(
            $hook_paths:path
        ),*
        $(,)?
    ) => {
        $(
            $crate::install_hook!(
                $hook_paths
            );
        )*
    };
}

#[repr(u8)]
pub enum Region {
    Text,
    Rodata,
    Data,
    Bss,
    Heap,
}

#[repr(C)]
pub struct InlineCtx {
    pub registers: [nn::os::CpuRegister; 31],
    pub sp: nn::os::CpuRegister,
    pub registers_f: [nn::os::FpuRegister; 32],
}

impl fmt::Display for InlineCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, reg) in self.registers.iter().enumerate() {
            unsafe {
                write!(f, "X[{}]: {:#08x?}\n", i, reg.x.as_ref())?;
            }
        }
        for (i, reg) in self.registers_f.iter().enumerate() {
            write!(f, "D[{}]: {:#08x?}\n", i, reg.d())?;
        }
        Ok(())
    }
}

extern "C" {
    pub fn A64HookFunction(
        symbol: *const libc::c_void,
        replace: *const libc::c_void,
        result: *mut *mut libc::c_void,
    );
    pub fn A64InlineHook(symbol: *const libc::c_void, replace: *const libc::c_void);
    pub fn getRegionAddress(region: Region) -> *mut libc::c_void;
}

pub struct HookInfo {
    /// Name of the function being used as the override
    pub fn_name: &'static str,

    /// User-given name of what the hook represents
    pub name: Option<String>,

    /// Offset of where to install the hook
    pub offset: Option<u64>,

    /// Symbol of where to install the hook
    pub symbol: Option<String>,

    /// Whether or not this is an inline hook
    pub inline: bool,
}

/// Type for representing a hook for this plugin
pub struct Hook {
    /// Pointer to the overloading function
    pub ptr: *const (),

    /// Info needed to identify and install this hook
    pub info: &'static HookInfo,
}

unsafe impl Sync for Hook {}

impl Hook {
    pub fn install(&self) {
        todo!()
    }
}

#[allow(improper_ctypes)]
extern "C" {
    static __hook_array_start: Hook;
    static __hook_array_end: Hook;
}

/// Iterate over the loaded hooks for this plugin
pub fn iter_hooks() -> impl Iterator<Item = &'static Hook> {
    let hook_start = unsafe { &__hook_array_start as *const Hook };
    let hook_end = unsafe { &__hook_array_end as *const Hook };

    let hook_count = ((hook_start as usize) - (hook_end as usize)) / core::mem::size_of::<Hook>();

    crate::println!("hook_count: {}", hook_count);
    crate::println!("hook_start: {:?}", hook_start);
    crate::println!("hook_end: {:?}", hook_start);

    unsafe { core::slice::from_raw_parts(hook_start, hook_count) }.iter()
}
