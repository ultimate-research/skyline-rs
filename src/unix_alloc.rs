use std::alloc::*;
use crate::libc;

pub struct UnixAllocator;

impl UnixAllocator {
    pub unsafe fn aligned_malloc(layout: &Layout) -> *mut u8 {
        let mut out = std::ptr::null_mut();

        let align = layout.align().max(std::mem::size_of::<usize>());
        let ret = libc::posix_memalign(&mut out, align, layout.size());
        if ret != 0 {
            std::ptr::null_mut()
        } else {
            out as *mut u8
        }
    }

    pub unsafe fn manual_realloc(ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());

        let new_ptr = Self.alloc(new_layout);
        if !new_ptr.is_null() {
            let size = std::cmp::min(old_layout.size(), new_size);
            std::ptr::copy_nonoverlapping(ptr, new_ptr, size);
            Self.dealloc(ptr, old_layout);
        }
        new_ptr
    }
}

unsafe impl GlobalAlloc for UnixAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= 16 && layout.align() <= layout.size() {
            libc::malloc(layout.size()) as *mut u8
        } else {
            Self::aligned_malloc(&layout)
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= 16 && layout.align() <= layout.size() {
            libc::calloc(layout.size(), 1) as *mut u8
        } else {
            let ptr = self.alloc(layout);
            if !ptr.is_null() {
                std::ptr::write_bytes(ptr, 0, layout.size());
            }
            ptr
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as _)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        Self::manual_realloc(ptr, layout, new_size)
    }
}