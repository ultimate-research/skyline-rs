extern "C" {
    #[link_name = "get_program_id"]
    fn get_program_id_impl() -> u64;
}

/// Get the program id for the current process.
pub fn get_program_id() -> u64 {
    unsafe {
        get_program_id_impl()
    }
}
