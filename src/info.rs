use crate::libc;

extern "C" {
    #[link_name = "get_program_id"]
    fn get_program_id_impl() -> u64;

    #[link_name = "get_plugin_addresses"]
    fn get_plugin_addresses(internal_addr: *const libc::c_void, start: *mut *mut libc::c_void, end: *mut *mut libc::c_void);
}

/// Get the program id for the current process.
pub fn get_program_id() -> u64 {
    unsafe {
        get_program_id_impl()
    }
}

pub unsafe fn containing_plugin(address: *const libc::c_void) -> (u64, u64) {
    let mut plug_start: *mut libc::c_void = 0 as _;
    let mut plug_end: *mut libc::c_void = 0 as _;
    get_plugin_addresses(address, &mut plug_start, &mut plug_end);
    (plug_start as u64, plug_end as u64)
}
