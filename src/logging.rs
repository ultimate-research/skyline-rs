use core::fmt::Display;
use core::fmt;
use std::ffi::CStr;
use crate::libc::{c_char, strlen, free};
use crate::hooks::{ getRegionAddress, Region };
use nnsdk::{
    root::cxa_demangle,
    ro::LookupSymbol,
    diag::{
        GetBacktrace,
        GetSymbolName,
    },
};

extern "C" {
    fn skyline_tcp_send_raw(bytes: *const u8, usize: u64);
}

pub fn log(message: &str) {
    unsafe {
        skyline_tcp_send_raw(message.as_bytes().as_ptr(), message.as_bytes().len() as _);
    }
}

/// Prints to the standard output, with a newline. For use in no_std plugins.
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
    Format wrapper used for displaying a [`Sized`] type to hex with 8 byte rows

    Example usage:
    ```rust
    # use skyline::logging::HexDump;
    let val: u32 = 3;
    println!("Hexdump:\n {}", HexDump(&val));
    ```
*/
pub struct HexDump<'a, T: Sized>(pub &'a T);

impl<'a, T> Display for HexDump<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex_dump_value(f, self.0)
    }
}

pub fn hex_dump_ptr<T>(ptr: *const T) {
    println!("{}", HexDump(unsafe { &*(ptr as *const u8) }))
}

pub fn hex_dump_str(ptr: *const c_char) {
    let len = unsafe { strlen(ptr) };
    let addr = ptr as usize;

    println!("{}", StrDumper(ptr, addr..addr + len));
}

struct StrDumper(pub *const c_char, core::ops::Range<usize>);

impl fmt::Display for StrDumper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        hex_dump(f, self.0, Some(self.1.clone()))
    }
}

const CHUNK_SIZE: usize = 0x10;
const NUMBERING_HEX: &str = "00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F ";
const NUMBERING_SEP: &str = "│";
const NUMBERING_ASCII: &str = " 0123456789ABCDEF";

#[cfg(not(feature = "std"))]
const LOG2_TAB: [usize; 64] = [
    63,  0, 58,  1, 59, 47, 53,  2,
    60, 39, 48, 27, 54, 33, 42,  3,
    61, 51, 37, 40, 49, 18, 28, 20,
    55, 30, 34, 11, 43, 14, 22,  4,
    62, 57, 46, 52, 38, 26, 32, 41,
    50, 36, 17, 19, 29, 10, 13, 21,
    56, 45, 25, 31, 35, 16,  9, 12,
    44, 24, 15,  8, 23,  7,  6,  5
];

#[cfg(not(feature = "std"))]
fn log2 (mut value: usize) -> f64 {
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value |= value >> 32;
    LOG2_TAB[((value - (value >> 1)) * 0x07EDD5E59A4E28C2) >> 58] as f64
}

#[cfg(feature = "std")]
fn log2 (value: usize) -> f64 {
    (value as f64).log2()
}

fn hex_num_len(val: usize) -> usize {
    (log2(val) / log2(0x10)) as usize + 1
}

fn to_ascii_dots(x: u8) -> char {
    match x {
        0..=0x1F | 0x7F..=0xA0 | 0xAD => '.',
        x => x as char,
    }
}

fn dump_hex_line(f: &mut fmt::Formatter, line: &[u8], addr: usize, highlight: &core::ops::Range<usize>) -> fmt::Result {
    write!(f, "{:08X}", addr)?;
    for (j, half) in line.chunks(8).enumerate() {
        write!(f, " ")?;
        for (i, x) in half.iter().enumerate() {
            let addr = addr + i + (j * 8);
            if highlight.contains(&addr) {
                write!(f, "\x1b[7m")?; // set highlight
            }
            write!(f, "{:02X}", x)?;
            if !highlight.contains(&(addr + 1)) || (j == 1 && i == 7) {
                write!(f, "\x1b[0m")?; // reset colors
            }
            write!(f, " ")?;
        }
    }
    write!(f, "│ ")?;
    for (i, &x) in line.iter().enumerate() {
        if highlight.contains(&(addr + i)) {
            write!(f, "\x1b[7m")?; // set highlight
        }
        write!(f, "{}", to_ascii_dots(x))?;
        write!(f, "\x1b[0m")?; // reset colors
    }
    writeln!(f)
}

fn hex_dump_bytes(f: &mut fmt::Formatter, byte_slice: &[u8], start: usize, highlight: core::ops::Range<usize>) -> fmt::Result {
    let num_spaces = hex_num_len(start.saturating_add(CHUNK_SIZE * 6)) + 1;
    for _ in 0..num_spaces {
        write!(f, " ")?;
    }
    writeln!(f, "{}{}{}", NUMBERING_HEX, NUMBERING_SEP, NUMBERING_ASCII)?;
    for _ in 0..num_spaces {
        write!(f, " ")?;
    }
    for _ in 0..NUMBERING_HEX.len() {
        write!(f, "─")?;
    }
    write!(f, "┼")?;
    for _ in 0..NUMBERING_ASCII.len() {
        write!(f, "─")?;
    }
    writeln!(f)?;

    let lines = byte_slice.chunks(CHUNK_SIZE).zip((0..).map(|x| (x * CHUNK_SIZE) + start));

    for (x, addr) in lines {
        dump_hex_line(f, x, addr, &highlight)?;
    }

    Ok(())
}

fn hex_dump<T>(f: &mut fmt::Formatter, addr: *const T, highlight: Option<core::ops::Range<usize>>) -> fmt::Result {
    let addr = addr as usize;
    let highlight = highlight.unwrap_or(addr..addr + 1);
    let aligned_addr = addr & !0xF;
    let start = aligned_addr.saturating_sub(CHUNK_SIZE * 3);
    let num_chunks = 7 + ((highlight.end - highlight.start) / CHUNK_SIZE);
    let byte_slice = unsafe { 
        core::slice::from_raw_parts(
            start as *const u8,
            CHUNK_SIZE * num_chunks
        )
    };

    hex_dump_bytes(f, byte_slice, start, highlight)
}

fn hex_dump_value<T: Sized>(f: &mut fmt::Formatter, val: &T) -> fmt::Result {
    let addr = val as *const T as usize;
    let size = core::mem::size_of::<T>();
    hex_dump(f, val as *const _, Some(addr..addr + size))
}

pub fn print_stack_trace() {
    let addresses = &mut [0 as *const u8;32];
    let addr_count = unsafe { GetBacktrace(addresses.as_mut_ptr(), 32) };

    for (idx, &addr) in addresses[0..addr_count].iter().enumerate() {
        if addr.is_null() {
            continue;
        }

        let name = &mut [0u8;255];
        
        unsafe { GetSymbolName(name.as_mut_ptr(), name.len() as u64, addr as u64) };

        let mut symbol_addr = 0;
        unsafe { LookupSymbol(&mut symbol_addr, name.as_ptr()) };

        let mut result = 0;
        let demangled_symbol = unsafe { cxa_demangle(name.as_ptr(), 0 as _, 0 as _, &mut result) };

        let c_name;

        if result == 0 {
            c_name = unsafe { CStr::from_ptr(demangled_symbol as _).to_str().unwrap_or("None") };
        } else {
            c_name = unsafe { CStr::from_ptr(name.as_ptr() as _).to_str().unwrap_or("None") };
        }

        println!("[{}] Address: {:x}, Symbol: {}+{:x}\n", idx, (symbol_addr as u64 - unsafe { getRegionAddress(Region::Text) as u64 } + 0x7100000000), c_name, addr as u64 - symbol_addr as u64);
        unsafe { free(demangled_symbol as _) };
    }
}