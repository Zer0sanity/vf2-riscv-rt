//! Convenience macros for CSR access based on [riscv](https://github.com/rust-embedded/riscv) macros.

#[macro_export]
macro_rules! read_as_csr {
    ($(#[$attr:meta])*, $csr_ty:ident, $csr_num:expr) => {
        impl $csr_ty {
            $(#[$attr])*
            pub fn read() -> Self {
                match () {
                    #[cfg(target_arch = "riscv64")]
                    () => {
                        let bits: usize;
                        unsafe {
                            core::arch::asm!(concat!("csrr {}, ", stringify!($csr_num)), out(reg) bits);
                        }
                        Self { bits }
                    }
                    #[cfg(not(target_arch = "riscv64"))]
                    () => Self { bits: 0 },
                }
            }
        }

        $(#[$attr])*
        pub fn read() -> $csr_ty {
            $csr_ty::read()
        }
    }
}

#[macro_export]
macro_rules! write_as_csr {
    ($(#[$attr:meta])*, $csr_ty:ident, $csr_num:expr) => {
        impl $csr_ty {
            $(#[$attr])*
            pub fn write(&self) {
                match () {
                    #[cfg(target_arch = "riscv64")]
                    () => unsafe {
                        core::arch::asm!(concat!("csrw ", stringify!($csr_num), ", {}"), in(reg) self.bits);
                    },
                    #[cfg(not(target_arch = "riscv64"))]
                    () => (),
                }
            }
        }

        $(#[$attr])*
        pub fn write(val: $csr_ty) {
            val.write();
        }
    }
}

/*#[macro_export]
macro_rules! set {
    ($csr_number:literal) => {
        /// Set the CSR
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _set(bits: usize) {
            core::arch::asm!(concat!("csrrs x0, ", stringify!($csr_number), ", {0}"), in(reg) bits)
        }
    };
}*/

#[macro_export]
macro_rules! set_csr {
    ($csr:expr) => {
        unsafe fn _set(_bits: usize) {
            match () {
                #[cfg(target_arch = "riscv64")]
                () => unsafe { core::arch::asm!(concat!("csrrs x0, ", stringify!($csr), ", {}"), in(reg) _bits); },
                #[cfg(not(target_arch = "riscv64"))]
                () => (),
            }
        }
    }
}

/*#[macro_export]
macro_rules! clear {
    ($csr_number:literal) => {
        /// Clear the CSR
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _clear(bits: usize) {
            core::arch::asm!(concat!("csrrc x0, ", stringify!($csr_number), ", {0}"), in(reg) bits)
        }
    };
}*/

#[macro_export]
macro_rules! clear_csr {
    ($csr:expr) => {
        unsafe fn _clear(_bits: usize) {
            match () {
                #[cfg(target_arch = "riscv64")]
                () => unsafe { core::arch::asm!(concat!("csrrc x0, ", stringify!($csr), ", {}"), in(reg) _bits); },
                #[cfg(not(target_arch = "riscv64"))]
                () => (),
            }
        }
    }
}

#[macro_export]
macro_rules! set_clear_csr {
    ($(#[$attr:meta])*, $set_fn:ident, $clear_fn:ident, $bits:expr) => {
        $(#[$attr])*
        pub unsafe fn $set_fn() {
            _set($bits);
        }

        $(#[$attr])*
        pub unsafe fn $clear_fn() {
            _clear($bits);
        }
    };
}
