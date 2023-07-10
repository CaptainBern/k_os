//! Handling of per-cpu data.

pub mod atomic;

use core::{
    arch::asm,
    cell::{OnceCell, RefCell},
};

use x86::msr::{wrmsr, IA32_GS_BASE};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Access,
}

pub struct PerCpu<T: 'static> {
    inner: unsafe fn() -> Option<&'static T>,
}

unsafe impl<T> Sync for PerCpu<T> {}

impl<T: 'static> PerCpu<T> {
    /// Construct a new per-cpu variable.
    ///
    /// # Safety
    /// This function should never be used for creating per-cpu variables! The macros
    /// should be used instead.
    pub const unsafe fn new(inner: unsafe fn() -> Option<&'static T>) -> PerCpu<T> {
        PerCpu { inner }
    }

    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.try_with(f).expect("Failed to access per-cpu")
    }

    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&T) -> R,
    {
        let percpu = unsafe { (self.inner)().ok_or(Error::Access)? };
        Ok(f(&percpu))
    }

    /// Provides a raw pointer to the percpu.
    pub fn as_ptr(&'static self) -> *const T {
        unsafe { (self.inner)().expect("Failed to get self as ptr") as *const T }
    }
}

impl<T: 'static> PerCpu<OnceCell<T>> {
    pub fn set(&'static self, val: T) -> Result<(), T> {
        self.with(|f| f.set(val))
    }

    pub fn with_or_init<I, F, R>(&'static self, init: I, f: F) -> R
    where
        I: FnOnce() -> T,
        F: FnOnce(&T) -> R,
    {
        self.with(|g| f(g.get_or_init(init)))
    }
}

impl<T: 'static> PerCpu<RefCell<T>> {
    pub fn with_borrow<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.with(|cell| f(&cell.borrow()))
    }

    pub fn with_borrow_mut<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.with(|cell| f(&mut cell.borrow_mut()))
    }
}

/// Declare a new per-cpu variable.
#[macro_export]
macro_rules! percpu {
    () => {};

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::PerCpu<$t> = $crate::__percpu_internal!($t, $init);
        $crate::percpu!($($rest)*);
    };
}

#[macro_export]
macro_rules! __percpu_internal {
    ($t:ty, $init:expr) => {{
        #[link_section = ".percpu"]
        static mut PRIVATE: $t = $init;

        extern "Rust" {
            #[allow(unused)] // we *DO* use it, but for some reason rust keeps complaining we don't.
            static PERCPU_OFFSET: u64;
        }

        #[inline(always)]
        unsafe fn offset() -> u64 {
            let tmp: u64;
            core::arch::asm!(
                "mov    %gs:({}), {}",
                sym PERCPU_OFFSET,
                out(reg) tmp,
                options(att_syntax, nostack, preserves_flags)
            );
            tmp
        }

        #[inline]
        unsafe fn inner() -> Option<&'static $t> {
            let raw = offset() + &PRIVATE as *const _ as u64;
            (raw as *const $t).as_ref()
        }

        unsafe { $crate::percpu::PerCpu::new(inner) }
    }};
}

/// Initialise the per-CPU for the current CPU.
///
/// # Safety
/// The given offset must be the start of the per-CPU region for the executing
/// CPU. Furthermore, this value should be unique for each CPU. When this function
/// returns, per-CPU storage is available.
pub unsafe fn init(offset: u64) {
    #[no_mangle]
    #[link_section = ".percpu"]
    static mut PERCPU_OFFSET: u64 = 0;

    // Store the offset in `%gs`.
    wrmsr(IA32_GS_BASE, offset);

    // Set the PERCPU_OFFSET for this CPU.
    asm!(
        "mov    {}, %gs:({})",
        in(reg) offset,
        sym PERCPU_OFFSET,
        options(att_syntax, preserves_flags, nostack)
    );
}
