use core::fmt::Display;
use std::fmt;

extern "C" {
    fn skyline_tcp_send_raw(bytes: *const u8, usize: u64);
}

pub fn log(message: &str) {
    unsafe {
        skyline_tcp_send_raw(message.as_bytes().as_ptr(), message.as_bytes().len() as _);
    }
}

/// Prints to the standard output, with a newline.
#[macro_export] macro_rules! println {
    () => {
        $crate::log();
    };
    ($($arg:tt)*) => {
        {
            use $crate::alloc::format;
            $crate::logging::log(&format!(
                $($arg)*
            ));
        }
    };
}

/**  
    For dumping a struct to 8 bytes per row
    Example usage:
    let val = SomeStruct::new();
    println!("Hexdump:\n {}", HexDump(&val));
*/
pub struct HexDump<'a, T: Sized>(pub &'a T);

impl<'a, T> Display for HexDump<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = core::mem::size_of::<T>();
        let ptr = self.0 as *const T as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, size) };
        let hex_dump = slice.iter()
            .map(|byte| format!("{:02X}", byte))
            .collect::<Vec<_>>()
            .chunks(8)
            .map(|chunk| chunk.join(" "))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", hex_dump)
    }
}