// Register definitions taken from @blu-dev

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct CpuRegister(u64);

impl CpuRegister {
    /// Returns the Aarch64 64-bit representation of this register
    pub fn x(self) -> u64 {
        self.0
    }

    /// Returns the Aarch64 32-bit representation of this register
    /// 
    /// This is equivalent to [`CpuRegister::r`]
    pub fn w(self) -> u32 {
        (self.0 & 0xFFFF_FFFF) as u32
    }

    /// Returns the Aarch32 32-bit representation of this register
    /// 
    /// This is equivalent to [`CpuRegister::w`]
    pub fn r(self) -> u32 {
        self.w()
    }

    /// Sets the Aarch64 64-bit representation of this register
    pub fn set_x(&mut self, x: u64) {
        self.0 = x;
    }

    /// Sets the Aarch64 32-bit representation of this register
    /// 
    /// This is equivalent to [`CpuRegister::set_r`]
    pub fn set_w(&mut self, w: u32) {
        self.0 = w as u64;
    }

    /// Sets the Aarch32 32-bit representation of this register
    /// 
    /// This is equivalent to [`CpuRegister::set_w`]
    pub fn set_r(&mut self, r: u32) {
        self.0 = r as u64;
    }
}

/// A structure to represent one of the Aarch64 NEON/SIMD registers.
/// 
/// There are 32 128-bit SIMD registers on Aarch64 systems, and they can be split into "lanes".
/// Each lane must be  8 * 2^n bits, where 0 <= n <= 4
/// 
/// It is common to pack structures such as 3-float Vectors, 4-float Vectors, 2-double Vectors, etc.
/// on to a singular SIMD register for easy addition/multiplication of the components.
/// 
/// The lanes are overlapping and can be set without disrupting the other lanes. For example, one
/// could divy up the 128-bits into three lanes: 32-bit, 32-bit, 64-bit. These lanes would be referenced
/// as `S[0]`, `S[1]`, and `D[1]` respectively.
/// 
/// The [`VectorRegister`] shares the same locations as the [`FpuRegister`], which only ever references the first
/// lane of the vector representation
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct VectorRegister(u128);

impl VectorRegister {
    /// Returns the 128-bit representation of this register
    pub fn v(self) -> u128 {
        self.0
    }

    /// Returns the two 64-bit components of this register as [`f64`] values
    pub fn d(self) -> [f64; 2] {
        unsafe {
            [
                *(&self as *const Self as *const f64).add(0),
                *(&self as *const Self as *const f64).add(1),
            ]
        }
    }

    /// Returns the four 32-bit components of this register as [`f32`] values
    pub fn s(self) -> [f32; 4] {
        unsafe {
            [
                *(&self as *const Self as *const f32).add(0),
                *(&self as *const Self as *const f32).add(1),
                *(&self as *const Self as *const f32).add(2),
                *(&self as *const Self as *const f32).add(3),
            ]
        }
    }

    /// Returns the eight 16-bit components of this register
    pub fn h(self) -> [u16; 8] {
        unsafe {
            [
                *(&self as *const Self as *const u16).add(0),
                *(&self as *const Self as *const u16).add(1),
                *(&self as *const Self as *const u16).add(2),
                *(&self as *const Self as *const u16).add(3),
                *(&self as *const Self as *const u16).add(4),
                *(&self as *const Self as *const u16).add(5),
                *(&self as *const Self as *const u16).add(6),
                *(&self as *const Self as *const u16).add(7),
            ]
        }
    }

    /// Returns the sixteen 8-bit components of this register
    pub fn b(self) -> [u8; 16] {
        unsafe {
            [
                *(&self as *const Self as *const u8).add(0),
                *(&self as *const Self as *const u8).add(1),
                *(&self as *const Self as *const u8).add(2),
                *(&self as *const Self as *const u8).add(3),
                *(&self as *const Self as *const u8).add(4),
                *(&self as *const Self as *const u8).add(5),
                *(&self as *const Self as *const u8).add(6),
                *(&self as *const Self as *const u8).add(7),
                *(&self as *const Self as *const u8).add(8),
                *(&self as *const Self as *const u8).add(9),
                *(&self as *const Self as *const u8).add(10),
                *(&self as *const Self as *const u8).add(11),
                *(&self as *const Self as *const u8).add(12),
                *(&self as *const Self as *const u8).add(13),
                *(&self as *const Self as *const u8).add(14),
                *(&self as *const Self as *const u8).add(15),
            ]
        }
    }

    /// Sets all 128 bits of the vector register
    pub fn set_v(&mut self, v: u128) {
        self.0 = v;
    }

    /// Sets the specified 64-bit lane of this register (other 64-bits are unmodified)
    pub fn set_d(&mut self, index: usize, d: f64) {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut Self as *mut f64, 2)[index] = d;
        }
    }

    /// Sets the specified 32-bit lane of this register (other lanes are unmodified)
    pub fn set_s(&mut self, index: usize, s: f32) {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut Self as *mut f32, 4)[index] = s;
        }
    }

    /// Sets the specified 16-bit lane of this register (other lanes are unmodified)
    pub fn set_h(&mut self, index: usize, h: u16) {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut Self as *mut u16, 8)[index] = h;
        }
    }

    /// Sets the specified 8-bit lane of this register (other lanes are unmodified)
    pub fn set_b(&mut self, index: usize, b: u8) {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut Self as *mut u8, 16)[index] = b;
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FpuRegister(u128);

impl FpuRegister {
    /// Copies this register into a [`VectorRegister`]
    pub fn as_vec(self) -> VectorRegister {
        VectorRegister(self.0)
    }

    /// Transforms the view of this register into a [`VectorRegister`] view.
    pub fn as_vec_mut(&mut self) -> &mut VectorRegister {
        unsafe {
            core::mem::transmute(self)
        }
    }

    /// Gets the 128-bit representation of this register
    pub fn q(self) -> u128 {
        self.0
    }

    /// Gets the 64-bit representation of this register as an [`f64`]
    pub fn d(self) -> f64 {
        unsafe {
            *(&self.0 as *const u128 as *const f64)
        }
    }

    /// Gets the 32-bit representation of this register as an [`f32`]
    pub fn s(self) -> f32 {
        unsafe {
            *(&self.0 as *const u128 as *const f32)
        }
    }

    /// Gets the 16-bit representation of this register
    pub fn h(self) -> u16 {
        unsafe {
            *(&self.0 as *const u128 as *const u16)
        }
    }

    /// Gets the 8-bit representation of this register
    pub fn b(self) -> u8 {
        unsafe {
            *(&self.0 as *const u128 as *const u8)
        }
    }

    /// Sets all 128-bits of the register
    pub fn set_q(&mut self, q: u128) {
        self.0 = q;
    }

    /// Sets the first 64-bits of the register, zeroing out the remaining bits
    pub fn set_d(&mut self, d: f64) {
        let vec = self.as_vec_mut();
        vec.set_v(0);
        vec.set_d(0, d);
    }

    /// Sets the first 32-bits of the register, zeroing out the remaining bits
    pub fn set_s(&mut self, s: f32) {
        let vec = self.as_vec_mut();
        vec.set_v(0);
        vec.set_s(0, s);
    }

    /// Sets the first 16-bits of the register, zeroing out the remaining bits
    pub fn set_h(&mut self, h: u16) {
        let vec = self.as_vec_mut();
        vec.set_v(0);
        vec.set_h(0, h);
    }

    /// Sets the first 8-bits of the register, zeroing out the remaining bits
    pub fn set_b(&mut self, b: u8) {
        let vec = self.as_vec_mut();
        vec.set_v(0);
        vec.set_b(0, b);
    }
}