pub trait Accessor {
    type Value: private::Sealed;

    fn load(&'static self) -> Self::Value;
    fn store(&'static self, val: Self::Value);
    fn fetch_add(&'static self, val: Self::Value) -> Self::Value;
    fn fetch_sub(&'static self, val: Self::Value) -> Self::Value;
    fn inc(&'static self);
    fn dec(&'static self);
    fn xor(&'static self, val: Self::Value);
    fn or(&'static self, val: Self::Value);
    fn and(&'static self, val: Self::Value);
}

mod private {
    pub trait Sealed {}
}

macro_rules! impl_sealed {
    ($t:ty) => {
        impl private::Sealed for $t {}
    };

    ($t:ty, $($rest:tt)*) => {
        impl_sealed!($t);
        impl_sealed!($($rest)*);
    };
}

impl_sealed!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);

pub struct Atomic<T: 'static + private::Sealed> {
    acc: &'static dyn Accessor<Value = T>,
}

impl<T: private::Sealed> Atomic<T> {
    pub const unsafe fn new(acc: &'static dyn Accessor<Value = T>) -> Self {
        Self { acc }
    }

    #[inline(always)]
    pub fn load(&'static self) -> T {
        self.acc.load()
    }

    #[inline(always)]
    pub fn store(&'static self, val: T) {
        self.acc.store(val);
    }

    #[inline(always)]
    pub fn fetch_add(&'static self, val: T) -> T {
        self.acc.fetch_add(val)
    }

    #[inline(always)]
    pub fn fetch_sub(&'static self, val: T) -> T {
        self.acc.fetch_sub(val)
    }

    #[inline(always)]
    pub fn inc(&'static self) {
        self.acc.inc();
    }

    #[inline(always)]
    pub fn dec(&'static self) {
        self.acc.dec();
    }

    #[inline(always)]
    pub fn xor(&'static self, val: T) {
        self.acc.xor(val);
    }

    #[inline(always)]
    pub fn or(&'static self, val: T) {
        self.acc.or(val);
    }

    #[inline(always)]
    pub fn and(&'static self, val: T) {
        self.acc.and(val);
    }
}

#[macro_export]
macro_rules! percpu_atom {
    () => {};

    // process multiple declarations
    ($(#[$attr:meta])* $vis:vis static $name:ident: u8 = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<u8> = $crate::__percpu_atom_internal!(u8, $init, "b", reg_byte);
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i8 = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<i8> = $crate::__percpu_atom_internal!(i8, $init, "b", reg_byte);
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u16 = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<u16> = $crate::__percpu_atom_internal!(u16, $init, "w", reg);
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i16 = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<i16> = $crate::__percpu_atom_internal!(i16, $init, "w", reg);
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u32 = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<u32> = $crate::__percpu_atom_internal!(u32, $init, "l", reg);
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i32 = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<i32> = $crate::__percpu_atom_internal!(i32, $init, "l", reg);
        $crate::percpu_tom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u64 = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<u64> = $crate::__percpu_atom_internal!(u64, $init, "q", reg);
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i64 = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<i64> = $crate::__percpu_atom_internal!(i64, $init, "q", reg);
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: usize = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<usize> = $crate::__percpu_atom_internal!(usize, $init, "q", reg);
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: isize = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<isize> = $crate::__percpu_atom_internal!(isize, $init, "q", reg);
        $crate::percpu_atom!($($rest)*);
    };

    // handle a single declaration
    ($(#[$attr:meta])* $vis:vis static $name:ident: u8 = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<u8> = $crate::__percpu_atom_internal!(u8, $init, "b", reg_byte);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i8 = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<i8> = $crate::__percpu_atom_internal!(i8, $init, "b", reg_byte);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u16 = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<u16> = $crate::__percpu_atom_internal!(u16, $init, "w", reg);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i16 = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<i16> = $crate::__percpu_atom_internal!(i16, $init, "w", reg);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u32 = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<u32> = $crate::__percpu_atom_internal!(u32, $init, "l", reg);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i32 = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<i32> = $crate::__percpu_atom_internal!(i32, $init, "l", reg);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u64 = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<u64> = $crate::__percpu_atom_internal!(u64, $init, "q", reg);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i64 = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<i64> = $crate::__percpu_atom_internal!(i64, $init, "q", reg);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: usize = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<usize> = $crate::__percpu_atom_internal!(usize, $init, "q", reg);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: isize = $init:expr;) => {
        $(#[$attr])* $vis const $name: $crate::percpu::atomic::Atomic<isize> = $crate::__percpu_atom_internal!(isize, $init, "q", reg);
    };
}

#[macro_export]
macro_rules! __percpu_atom_internal {
    (
        $t:ty,
        $init:expr,
        $mod:literal,
        $reg:ident
    ) => {
        {
            #[link_section = ".percpu"]
            static PRIVATE: $t = $init;

            struct Private;

            impl $crate::percpu::atomic::Accessor for Private {
                type Value = $t;

                #[inline(always)]
                fn load(&'static self) -> $t {
                    let val: $t;
                    unsafe {
                        core::arch::asm!(
                            core::concat!(core::concat!("mov", $mod), "   %gs:({}), {}"),
                            sym PRIVATE,
                            out($reg) val,
                            options(att_syntax, nostack, preserves_flags, readonly)
                        );
                    }
                    val
                }

                #[inline(always)]
                fn store(&'static self, val: $t) {
                    unsafe {
                        core::arch::asm!(
                            core::concat!(core::concat!("mov", $mod), " {}, %gs:({})"),
                            in($reg) val,
                            sym PRIVATE,
                            options(att_syntax, nostack, preserves_flags)
                        );
                    }
                }

                #[inline(always)]
                fn fetch_add(&'static self, val: $t) -> $t {
                    let mut tmp: $t = val;
                    unsafe {
                        core::arch::asm!(
                            core::concat!(core::concat!("xadd", $mod), " {}, %gs:({})"),
                            inout($reg) tmp,
                            sym PRIVATE,
                            options(att_syntax, nostack)
                        );
                    }
                    val + tmp
                }

                #[inline(always)]
                fn fetch_sub(&'static self, val: $t) -> $t {
                    self.fetch_add(val.wrapping_neg())
                }

                #[inline(always)]
                fn inc(&'static self) {
                    unsafe {
                        core::arch::asm!(
                            core::concat!(core::concat!("inc", $mod), " %gs:({})"),
                            sym PRIVATE,
                            options(att_syntax, nostack)
                        );
                    }
                }

                #[inline(always)]
                fn dec(&'static self) {
                    unsafe {
                        core::arch::asm!(
                            core::concat!(core::concat!("dec", $mod), " %gs:({})"),
                            sym PRIVATE,
                            options(att_syntax, nostack)
                        );
                    }
                }

                #[inline(always)]
                fn and(&'static self, val: $t) {
                    unsafe {
                        core::arch::asm!(
                            core::concat!(core::concat!("and", $mod), " {}, %gs:({})"),
                            in($reg) val,
                            sym PRIVATE,
                            options(att_syntax, nostack)
                        );
                    }
                }

                #[inline(always)]
                fn or(&'static self, val: $t) {
                    unsafe {
                        core::arch::asm!(
                            core::concat!(core::concat!("or", $mod), " {}, %gs:({})"),
                            in($reg) val,
                            sym PRIVATE,
                            options(att_syntax, nostack)
                        );
                    }
                }

                #[inline(always)]
                fn xor(&'static self, val: $t) {
                    unsafe {
                        core::arch::asm!(
                            core::concat!(core::concat!("xor", $mod), " {}, %gs:({})"),
                            in($reg) val,
                            sym PRIVATE,
                            options(att_syntax, nostack)
                        );
                    }
                }
            }

            unsafe {
                $crate::percpu::atomic::Atomic::new(&Private)
            }
        }
    };
}
