use core::marker::PhantomData;

macro_rules! define_atomic {
    (
        $t:ty,
        $local:ident,
        $global:ident
    ) => {
        /// A local accessor for a per-cpu variable.
        pub trait $local {
            unsafe fn load() -> $t;
            unsafe fn store<const VAL: $t>();
            unsafe fn inc();
            unsafe fn dec();
            unsafe fn fetch_add(val: $t) -> $t;
            unsafe fn fetch_sub(val: $t) -> $t;
            unsafe fn xor<const VAL: $t>();
            unsafe fn or<const VAL: $t>();
            unsafe fn and<const VAL: $t>();
        }

        pub struct $global<T: $local> {
            _phantom: PhantomData<T>,
        }

        impl<T: $local> $global<T> {
            #[doc(hidden)]
            pub const unsafe fn new() -> Self {
                Self {
                    _phantom: PhantomData,
                }
            }

            #[inline(always)]
            pub fn load(&'static self) -> $t {
                unsafe { T::load() }
            }

            #[inline(always)]
            pub fn store<const VAL: $t>(&'static self) {
                unsafe {
                    T::store::<VAL>();
                }
            }

            #[inline(always)]
            pub fn inc(&'static self) {
                unsafe {
                    T::inc();
                }
            }

            #[inline(always)]
            pub fn dec(&'static self) {
                unsafe {
                    T::dec();
                }
            }

            #[inline(always)]
            pub fn fetch_add(&'static self, val: $t) -> $t {
                unsafe { T::fetch_add(val) }
            }

            #[inline(always)]
            pub fn fetch_sub(&'static self, val: $t) -> $t {
                unsafe { T::fetch_sub(val) }
            }

            #[inline(always)]
            pub fn xor<const VAL: $t>(&'static self) {
                unsafe {
                    T::xor::<VAL>();
                }
            }

            #[inline(always)]
            pub fn or<const VAL: $t>(&'static self) {
                unsafe {
                    T::or::<VAL>();
                }
            }

            #[inline(always)]
            pub fn and<const VAL: $t>(&'static self) {
                unsafe {
                    T::and::<VAL>();
                }
            }
        }
    };
}

define_atomic!(u8, LocalU8, AtomicU8);
define_atomic!(i8, LocalI8, AtomicI8);
define_atomic!(u16, LocalU16, AtomicU16);
define_atomic!(i16, LocalI16, AtomicI16);
define_atomic!(u32, LocalU32, AtomicU32);
define_atomic!(i32, LocalI32, AtomicI32);
define_atomic!(u64, LocalU64, AtomicU64);
define_atomic!(i64, LocalI64, AtomicI64);
define_atomic!(usize, LocalUsize, AtomicUsize);
define_atomic!(isize, LocalIsize, AtomicIsize);

#[macro_export]
macro_rules! percpu_atomic {
    () => {};

    // process multiple declarations
    ($(#[$attr:meta])* $vis:vis static $name:ident: u8 = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            u8,
            $crate::percpu::atomic::AtomicU8<$name>,
            $crate::percpu::atomic::LocalU8,
            $init,
            "b",
            reg_byte
        );
        $crate::percpu_atomic!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i8 = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            i8,
            $crate::percpu::atomic::AtomicI8<$name>,
            $crate::percpu::atomic::LocalI8,
            $init,
            "b",
            reg_byte
        );
        $crate::percpu_atomic!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u16 = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            u16,
            $crate::percpu::atomic::AtomicU16<$name>,
            $crate::percpu::atomic::LocalU16,
            $init,
            "w",
            reg
        );
        $crate::percpu_atomic!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i16 = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            i16,
            $crate::percpu::atomic::AtomicI16<$name>,
            $crate::percpu::atomic::LocalI16,
            $init,
            "w",
            reg
        );
        $crate::percpu_atomic!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u32 = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            u32,
            $crate::percpu::atomic::AtomicU32<$name>,
            $crate::percpu::atomic::LocalU32,
            $init,
            "l",
            reg
        );
        $crate::percpu_atomic!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i32 = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            i32,
            $crate::percpu::atomic::AtomicI32<$name>,
            $crate::percpu::atomic::LocalI32,
            $init,
            "l",
            reg
        );
        $crate::percpu_atomic!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: u64 = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            u64,
            $crate::percpu::atomic::AtomicU64<$name>,
            $crate::percpu::atomic::LocalU64,
            $init,
            "q",
            reg
        );
        $crate::percpu_atomic!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: i64 = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            i64,
            $crate::percpu::atomic::AtomicI64<$name>,
            $crate::percpu::atomic::LocalI64,
            $init,
            "q",
            reg
        );
        $crate::percpu_atom!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: usize = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            usize,
            $crate::percpu::atomic::AtomicUsize<$name>,
            $crate::percpu::atomic::LocalUsize,
            $init,
            "q",
            reg
        );
        $crate::percpu_atomic!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: isize = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_atomic_internal!(
            $(#[$attr])*,
            $vis,
            $name,
            isize,
            $crate::percpu::atomic::AtomicIsize<$name>,
            $crate::percpu::atomic::LocalIsize,
            $init,
            "q",
            reg
        );
        $crate::percpu_atomic!($($rest)*);
    };
}

#[macro_export]
macro_rules! __percpu_atomic_internal {
    (
        $(#[$attr:meta])*,
        $vis:vis,
        $name:ident,
        $t:ty,
        $global:ty,
        $local:ty,
        $init:expr,
        $mod:literal,
        $reg:ident
    ) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        $(#[$attr])*
        $vis enum $name {}

        #[allow(non_upper_case_globals)]
        $vis const $name: $global = {
            #[link_section = ".percpu"]
            static mut PRIVATE: $t = $init;

            impl $local for $name {
                #[inline(always)]
                unsafe fn load() -> $t {
                    let val: $t;
                    core::arch::asm!(
                        core::concat!(core::concat!("mov", $mod), "   %gs:({}), {}"),
                        sym PRIVATE,
                        out($reg) val,
                        options(att_syntax, nostack, preserves_flags, readonly)
                    );
                    val
                }

                #[inline(always)]
                unsafe fn store<const VAL: $t>() {
                    core::arch::asm!(
                        core::concat!(core::concat!("mov", $mod), " ${}, %gs:({})"),
                        const VAL,
                        sym PRIVATE,
                        options(att_syntax, nostack, preserves_flags)
                    );
                }

                #[inline(always)]
                unsafe fn inc() {
                    core::arch::asm!(
                        core::concat!(core::concat!("inc", $mod), " %gs:({})"),
                        sym PRIVATE,
                        options(att_syntax, nostack)
                    );
                }

                #[inline(always)]
                unsafe fn dec() {
                    core::arch::asm!(
                        core::concat!(core::concat!("dec", $mod), " %gs:({})"),
                        sym PRIVATE,
                        options(att_syntax, nostack)
                    );
                }

                #[inline(always)]
                unsafe fn fetch_add(val: $t) -> $t {
                    let mut tmp: $t = val;
                    core::arch::asm!(
                        core::concat!(core::concat!("xadd", $mod), " {}, %gs:({})"),
                        inout($reg) tmp,
                        sym PRIVATE,
                        options(att_syntax, nostack)
                    );
                    val + tmp
                }

                #[inline(always)]
                unsafe fn fetch_sub(val: $t) -> $t {
                    $name::fetch_add(val.wrapping_neg())
                }

                #[inline(always)]
                unsafe fn xor<const VAL: $t>() {
                    core::arch::asm!(
                        core::concat!(core::concat!("xor", $mod), " ${}, %gs:({})"),
                        const VAL,
                        sym PRIVATE,
                        options(att_syntax, nostack)
                    );
                }

                #[inline(always)]
                unsafe fn or<const VAL: $t>() {
                    core::arch::asm!(
                        core::concat!(core::concat!("or", $mod), " ${}, %gs:({})"),
                        const VAL,
                        sym PRIVATE,
                        options(att_syntax, nostack)
                    );
                }

                #[inline(always)]
                unsafe fn and<const VAL: $t>() {
                    core::arch::asm!(
                        core::concat!(core::concat!("and", $mod), " ${}, %gs:({})"),
                        const VAL,
                        sym PRIVATE,
                        options(att_syntax, nostack)
                    );
                }
            }

            unsafe {
                <$global>::new()
            }
        };
    };
}
