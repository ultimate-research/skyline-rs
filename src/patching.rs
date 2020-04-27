use crate::libc::{c_void, size_t, strlen};
use crate::error::{SwitchResult, Error, ErrorKind};
use crate::hooks::{Region, getRegionAddress};

extern "C" {
    pub fn sky_memcpy(dst: *const c_void, src: *const c_void, size: size_t) -> SwitchResult;
}

/// Overwrite a string in read-only data with a Rust string given the offset from the start of .text
pub unsafe fn patch_str(offset: usize, string: &str) -> Result<(), Error> {
    let text_ptr = getRegionAddress(Region::Text) as *const u8;
    let str_ptr = text_ptr.offset(offset as isize);

    let len = strlen(str_ptr);

    if len < string.len() {
        return Err(Error::Skyline { kind: ErrorKind::StringTooLong })
    }

    let string = String::from(string) + "\0";
    
    sky_memcpy(str_ptr as _, string.as_ptr() as _, string.len()).ok()?;

    Ok(())
}

/// Overwrite a value in read-only data with a passed value given an offset from the start of .text
pub unsafe fn patch_data<T: Sized + Copy>(offset: usize, val: &T) -> Result<(), Error> {
    let text_ptr = getRegionAddress(Region::Text) as *const u8;
    patch_data_from_text(text_ptr, offset, val)
}

/// Overwrite a value in read-only data with a passed value given an offset from the start of .text
pub unsafe fn patch_data_from_text<T: Sized + Copy>(text_offset: *const u8, offset: usize, val: &T) -> Result<(), Error> {
    let text_ptr = text_offset;
    let data_ptr = text_ptr.offset(offset as isize);

    sky_memcpy(data_ptr as _, val as *const _ as _, core::mem::size_of::<T>()).ok()?;

    Ok(())
}
