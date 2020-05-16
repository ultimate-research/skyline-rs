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

pub enum BranchType {
    Branch,
    BranchLink
}

pub struct BranchBuilder {
    branch_type: BranchType,
    offset: Option<usize>,
    ptr: Option<*const ()>,
    // TODO: add NRO support
}

impl BranchBuilder {
    fn internal_new() -> Self {
        Self {
            branch_type: BranchType::Branch,
            offset: None,
            ptr: None
        }
    }

    /// Create new branch builder for a `b` ARM instruction
    pub fn branch() -> Self {
        Self {
            branch_type: BranchType::Branch,
            ..BranchBuilder::internal_new()
        }
    }

    /// Create new branch builder for a `bl` ARM instruction
    pub fn branch_link() -> Self {
        Self {
            branch_type: BranchType::BranchLink,
            ..BranchBuilder::internal_new()
        }
    }

    /// Set the offset within the executable of the instruction to replace
    pub fn branch_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);

        self
    }

    /// Offset within the executable for the branch to jump to
    pub fn branch_to_offset(mut self, offset: usize) -> Self {
        unsafe {
            self.ptr = Some(
                 (getRegionAddress(Region::Text) as *const u8)
                    .offset(offset as isize) as *const ()
            );
        }

        self
    }

    /// Set a pointer for the branch to be jumped to. Must be within +/- 128 MiB of the given offset
    pub fn branch_to_ptr<T>(mut self, ptr: *const T) -> Self {
        self.ptr = Some(ptr as *const ());

        self
    }

    ///
    /// Replaces an instruction at the provided offset with a branch to the given pointer.
    ///
    /// # Panics
    ///
    /// Panics if an offset/ptr hasn't been provided or if the pointer is out of range of the
    /// branch
    #[track_caller]
    pub fn replace(self) {
        let offset = match self.offset {
            Some(offset) => offset,
            None => panic!("Offset is required to replace")
        };

        let instr_magic = match self.branch_type {
            BranchType::Branch => 0b000101,
            BranchType::BranchLink => 0b100101,
        } << 26;

        let branch_ptr = unsafe {
            (getRegionAddress(Region::Text) as *const u8).offset(offset as isize)
        } as isize;

        let branch_to_ptr = match self.ptr {
            Some(ptr) => ptr as *const u8,
            None => panic!("Either branch_to_ptr or branch_to_offset is required to replace")
        } as isize;

        let imm26 = match (branch_to_ptr - branch_ptr) / 4 {
            distance if within_branch_range(distance)
                => ((branch_to_ptr - branch_ptr) as usize) >> 2,
            _ => panic!("Branch target is out of range, must be within +/- 128 MiB")
        };

        let instr: u64 = (instr_magic | imm26) as u64;

        unsafe {
            if let Err(err) = patch_data(offset, &instr) {
                panic!("Failed to patch data, error: {:?}", err)
            }
        }
    }
}

#[allow(non_upper_case_globals)]
const MiB: isize = 0x100000;
const BRANCH_RANGE: isize = 128 * MiB;

fn within_branch_range(distance: isize) -> bool {
    (-BRANCH_RANGE..BRANCH_RANGE).contains(&distance)
}
