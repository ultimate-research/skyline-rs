use crate::error::{Error, SwitchResult};
use crate::hooks::{getRegionAddress, Region};
use crate::libc::{c_void, size_t};

#[cfg(not(feature = "std"))]
use alloc::string::String;

static NOP: u32 = 0xd503201f;

extern "C" {
    pub fn sky_memcpy(dst: *const c_void, src: *const c_void, size: size_t) -> SwitchResult;
}

/// Overwrite a string in read-only data with a Rust string given the offset from the start of .text
#[doc(hidden)]
#[deprecated(note = "Use Patch instead.")]
pub unsafe fn patch_str(offset: usize, string: &str) -> Result<(), Error> {
    Patch::in_text(offset).cstr(string)
}

/// Overwrite a value in read-only data with a passed value given an offset from the start of .text
#[doc(hidden)]
#[deprecated(note = "Use Patch instead.")]
pub unsafe fn patch_data<T: Sized + Copy>(offset: usize, val: &T) -> Result<(), Error> {
    Patch::in_text(offset).data(val)
}

/// Overwrite a value in read-only data with a passed value given an offset from the start of .text
#[doc(hidden)]
#[deprecated(note = "Use Patch instead.")]
pub unsafe fn patch_data_from_text<T: Sized + Copy>(
    text_offset: *const u8,
    offset: usize,
    val: &T,
) -> Result<(), Error> {
    PatchBuilder(text_offset.add(offset)).data(val)
}

/// Replace the instruction at the given offset from the start of .text with NOP
#[doc(hidden)]
#[deprecated(note = "Use Patch instead.")]
pub unsafe fn nop_data(offset: usize) -> Result<(), Error> {
    Patch::in_text(offset).nop()
}

/// Overwrite a value in read-only data with a passed value given a pointer plus offset
pub unsafe fn patch_pointer_with_offset<T: Sized + Copy>(
    pointer: *const u8,
    offset: isize,
    val: &T,
) -> Result<(), Error> {
    sky_memcpy(
        pointer.offset(offset) as _,
        val as *const _ as _,
        core::mem::size_of::<T>(),
    )
    .ok()?;

    Ok(())
}

/// Overwrite a value in read-only data with a passed value given a pointer
pub unsafe fn patch_pointer<T: Sized + Copy>(pointer: *const u8, val: &T) -> Result<(), Error> {
    patch_pointer_with_offset(pointer, 0, val)
}

/// Replace the instruction at the given pointer with NOP
pub unsafe fn nop_pointer(pointer: *const u8) -> Result<(), Error> {
    patch_pointer(pointer as _, &NOP)
}

/// Replace the instruction at the given pointer plus offset with NOP
pub unsafe fn nop_pointer_with_offset(pointer: *const u8, offset: isize) -> Result<(), Error> {
    patch_pointer(pointer.offset(offset) as _, &NOP)
}

/// A constructor to acquire a [`PatchBuilder`], which you can use to patch the game's memory.
///
/// Example:
///
/// ```
/// // Replace the instruction at `main` + 0x14a8504 with a branch
/// // to `main` + 0x14a853C
/// let text_builder: PatchBuilder = Patch::in_text(0x69),
/// ```
pub struct Patch(usize);

impl Patch {
    fn compute_address(offset: usize, region: Region) -> *const u8 {
        unsafe { (getRegionAddress(region) as *const u8).add(offset) }
    }

    /// Provide the base offset to work with for methods.
    /// This offset will be treated as absolute.
    ///
    /// If you would like to work relative to a region, prefer using the other methods like [Patch::in_text](Patch#in_text).
    ///
    /// Some methods, such as [branch_to](Patch#branch_to), will assume a Region for you.
    ///
    /// Example:
    ///
    /// ```
    /// // In this context, branch_to will overwrite the instruction at offset 0x69
    /// // Since branch_to assumes that you are working using Region::Text, your offset will be turned into .text+offset.
    /// Patch::at_offset(0x69).branch_to(0x420);
    /// ```
    pub fn at_offset(offset: usize) -> Self {
        Self(offset)
    }

    /// Insert a ``b`` ARM instruction to jump to the destination offset.
    /// It is assumed that the offset you provided is relative to the Text region of the running executable
    ///
    /// Shortcut method for:
    /// ```
    /// BranchBuilder::branch().branch_offset().branch_to_offset().replace()
    /// ```
    ///
    /// Example:
    ///
    /// ```
    /// // Overwriting the instruction at offset 0x69 with a branch in the .text section that redirects the Program Counter to address 0x420
    /// Patch::at_offset(0x69).branch_to(0x420);
    /// ```
    pub fn branch_to(self, dest_offset: usize) {
        BranchBuilder::branch()
            .branch_offset(self.0)
            .branch_to_offset(dest_offset)
            .replace()
    }

    /// Insert a ``b`` ARM instruction to jump to the destination offset.
    /// The offset you provide must be relative to the base address provided to the constructor.
    ///
    /// Shortcut method for:
    /// ```
    /// BranchBuilder::branch().branch_offset().branch_to_offset().replace()
    /// ```
    ///
    /// Example:
    ///
    /// ```
    /// // Overwriting the instruction at offset 0x69420 with a branch in the .text section that redirects the Program Counter to address 0x420
    /// Patch::in_text(0x69000).branch_to_relative(0x420);
    /// ```
    pub fn branch_to_relative(self, dest_offset: usize) {
        BranchBuilder::branch()
            .branch_offset(self.0)
            .branch_to_offset(self.0 + dest_offset)
            .replace()
    }

    /// Insert a ``bl`` ARM instruction to jump to the destination offset.
    /// It is assumed that the offset you provided is relative to the Text region of the running executable
    ///
    /// Shortcut method for:
    /// ```
    /// BranchBuilder::branch_link().branch_offset().branch_to_offset().replace
    /// ```
    ///
    /// Example:
    ///
    /// ```
    /// // Overwriting the instruction at offset 0x69 with a branch link in the .text section that redirects the Program Counter to address 0x420
    /// Patch::at_offset(0x69).branch_link_to(0x420);
    /// ```
    pub fn branch_link_to(self, dest_offset: usize) {
        BranchBuilder::branch()
            .branch_offset(self.0)
            .branch_to_offset(dest_offset)
            .replace()
    }

    /// Insert a ``bl`` ARM instruction to jump to the destination offset.
    /// The offset you provide must be relative to the base address provided to the constructor.
    ///
    /// Shortcut method for:
    /// ```
    /// BranchBuilder::branch_link().branch_offset().branch_to_offset().replace
    /// ```
    ///
    /// Example:
    ///
    /// ```
    /// // Overwriting the instruction at offset 0x69420 with a branch link in the .text section that redirects the Program Counter to address 0x420
    /// Patch::in_text(0x69000).branch_link_to_relative(0x420);
    /// ```
    pub fn branch_link_to_relative(self, dest_offset: usize) {
        BranchBuilder::branch()
            .branch_offset(self.0)
            .branch_to_offset(self.0 + dest_offset)
            .replace()
    }

    /// Use the base offset provided to [at_offset](Patch#at_offset) to get an address for a section of the executable.
    ///
    /// It is preferable that you use the shortcut methods for conciseness.
    ///
    /// Example:
    ///
    /// ```
    /// // In this context, branch_to will overwrite the instruction at offset 0x69
    /// let builder: PatchBuilder = Patch::at_offset(0x69).in_section(Region::Text);
    /// ```
    pub fn in_section(self, region: Region) -> PatchBuilder {
        PatchBuilder(Self::compute_address(self.0, region))
    }

    /// Provide a PatchBuilder targeting the .text section
    ///
    /// Shortcut method for:
    /// ```
    /// PatchBuilder::at_offset(offset).in_section(Region::Text)
    /// ```
    ///
    /// Example:
    /// ```
    /// let builder: PatchBuilder = Patch::in_text(offset);
    /// ```
    pub fn in_text(offset: usize) -> PatchBuilder {
        PatchBuilder(Self::compute_address(offset, Region::Text))
    }

    /// Provide a PatchBuilder targeting the .data section
    ///
    /// Shortcut method for:
    /// ```
    /// PatchBuilder::at_offset(offset).in_section(Region::Data)
    /// ```
    ///
    /// Example:
    /// ```
    /// let builder: PatchBuilder = Patch::in_data(offset);
    /// ```
    pub fn in_data(offset: usize) -> PatchBuilder {
        PatchBuilder(Self::compute_address(offset, Region::Data))
    }

    /// Provide a PatchBuilder targeting the .rodata section
    ///
    /// Shortcut method for:
    /// ```
    /// PatchBuilder::at_offset(offset).in_section(Region::Rodata)
    /// ```
    ///
    /// Example:
    /// ```
    /// let builder: PatchBuilder = Patch::in_rodata(offset);
    /// ```
    pub fn in_rodata(offset: usize) -> PatchBuilder {
        PatchBuilder(Self::compute_address(offset, Region::Rodata))
    }

    /// Provide a PatchBuilder targeting the .bss section
    ///
    /// Shortcut method for:
    /// ```
    /// PatchBuilder::at_offset(offset).in_section(Region::Bss)
    /// ```
    ///
    /// Example:
    /// ```
    /// let builder: PatchBuilder = Patch::in_bss(offset);
    /// ```
    pub fn in_bss(offset: usize) -> PatchBuilder {
        PatchBuilder(Self::compute_address(offset, Region::Bss))
    }

    /// Provide a PatchBuilder targeting the heap
    ///
    /// Shortcut method for:
    /// ```
    /// PatchBuilder::at_offset(offset).in_section(Region::Heap)
    /// ```
    ///
    /// Example:
    /// ```
    /// let builder: PatchBuilder = Patch::in_heap(offset);
    /// ```
    pub fn in_heap(offset: usize) -> PatchBuilder {
        PatchBuilder(Self::compute_address(offset, Region::Heap))
    }
}

/// A builder which you can use the patch the game's memory.
///
/// Example:
///
/// ```
/// // Replace the instruction at `main` + 0x69 with a NOP instruction
/// Patch::in_text(0x69).nop().unwrap()
/// ```
pub struct PatchBuilder(*const u8);

impl PatchBuilder {
    /// Overwrites data at the provided offset with the provided value.
    /// Equivalent to memcpy
    pub fn data<T: Sized + Copy>(self, val: T) -> Result<(), Error> {
        unsafe {
            sky_memcpy(
                self.0 as _,
                &val as *const _ as _,
                core::mem::size_of::<T>(),
            )
            .ok()?
        };
        Ok(())
    }

    /// Overwrites data at the provided offset with the content of a slice.
    ///
    /// # Example:
    /// ```
    /// // An array of four u8
    /// Patch::at_data(0x69).bytes(b"Ferris").unwrap();
    ///
    /// // A &str (with no null-terminator)
    /// let a_string = String::from("Ferris");
    /// Patch::in_data(0x69).bytes(&a_string).unwrap();
    ///
    /// // A &[u8] slice
    /// let log_wide = &[0xef, 0xbc, 0xac, 0xef, 0xbd, 0x8f, 0xef, 0xbd, 0x87,  0x00, 0x00];
    /// Patch::in_data(0x69).bytes(log_wide).unwrap();
    /// ```
    pub fn bytes<B: AsRef<[u8]>>(self, val: B) -> Result<(), Error> {
        let slice = val.as_ref();
        unsafe { sky_memcpy(self.0 as _, slice.as_ptr() as *const _ as _, slice.len()).ok()? };

        Ok(())
    }

    /// Overwrites data at the provided offset with a C string.
    /// The null-terminator is appended for you.
    ///
    /// If you do not wish for the null-terminator to be added, use [bytes](PatchBuilder#bytes) instead.
    ///
    /// Example:
    /// ```
    /// Patch::at_data(0x69).cstr("Ferris").unwrap();
    /// ```
    pub fn cstr(self, string: &str) -> Result<(), Error> {
        let string = String::from(string) + "\0";
        self.bytes(&string)
    }

    /// Overwrites bytes at the provided offset with a NOP instruction.
    ///
    /// Example:
    /// ```
    /// Patch::at_text(0x69).nop().unwrap();
    /// ```
    pub fn nop(self) -> Result<(), Error> {
        self.data(NOP)
    }
}

enum BranchType {
    Branch,
    BranchLink,
}

/// A builder type to help when replacing branches in games
///
/// Example:
///
/// ```rust
/// // Replace the instruction at `main` + 0x14a8504 with a branch
/// // to `main` + 0x14a853C
/// BranchBuilder::branch()
///     .branch_offset(0x14a8504)
///     .branch_to_offset(0x14a853C)
///     .replace()
///
/// // Replace the instruction at `main` + 0x14a8504 with a branch
/// // to `replacement_function`
/// BranchBuilder::branch()
///     .branch_offset(0x14a8504)
///     .branch_to_ptr(replacement_function as *const ())
///     .replace()
/// ```
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
            ptr: None,
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
                (getRegionAddress(Region::Text) as *const u8).offset(offset as isize) as *const (),
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
            None => panic!("Offset is required to replace"),
        };

        let instr_magic = match self.branch_type {
            BranchType::Branch => 0b000101,
            BranchType::BranchLink => 0b100101,
        } << 26;

        let branch_ptr =
            unsafe { (getRegionAddress(Region::Text) as *const u8).offset(offset as isize) }
                as isize;

        let branch_to_ptr = match self.ptr {
            Some(ptr) => ptr as *const u8,
            None => panic!("Either branch_to_ptr or branch_to_offset is required to replace"),
        } as isize;

        let imm26 = match (branch_to_ptr - branch_ptr) / 4 {
            distance if within_branch_range(distance) => {
                ((branch_to_ptr - branch_ptr) as usize) >> 2
            }
            _ => panic!("Branch target is out of range, must be within +/- 128 MiB"),
        };

        let instr: u64 = (instr_magic | imm26) as u64;

        if let Err(err) = Patch::in_text(offset).data(instr) {
            panic!("Failed to patch data, error: {:?}", err)
        }
    }
}

#[allow(non_upper_case_globals)]
const MiB: isize = 0x100000;
const BRANCH_RANGE: isize = 128 * MiB;

fn within_branch_range(distance: isize) -> bool {
    (-BRANCH_RANGE..BRANCH_RANGE).contains(&distance)
}
