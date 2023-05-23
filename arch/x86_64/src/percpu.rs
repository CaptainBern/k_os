//! Handling of per-cpu data.

use core::cell::{Cell, RefCell};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Access,
}

/// A per-cpu variable.
///
/// This struct is used to access the cpu-local copy of the contained value. As with
/// the [`thread_local!`] macro provided by std, this too should only be instantiated
/// using the [`percpu!`] macro. The primary methods to access the contained value are
/// [`with`] and [`try_with`].
///
/// As with the std [`thread_local!`] variant, per-cpu variables are lazily initialised
/// upon first access using [`with`] or [`try_with`].
///
/// # Safety
///
/// Before accessing any per-cpu, the caller must make sure that:
///
/// 1. Each cpu has its own block of memory in which the variables are stored. These
///    blocks may *not* overlap.
/// 2. Blocks for APs *must* be zeroed. The block reserved for the BSP may be zeroed
///    but it is not required. BSP access should be a little faster since its variables
///    are statically allocated.
/// 3. `gs.base` *must* contain the correct base address of the per-cpu section reserved
///    for whichever cpu the caller is running on. Accessing any per-cpu variable prior
///    to properly setting `gs.base` will cause UB!
/// 4. Each cpu must have a unique `gs.base`. No safety guarantees can be made if multiple
///    cpus attempt to access the same block.
///
/// ## Details
///
/// Each variable is stored in a self-referencing wrapper. These wrappers are lazily
/// initialised upon first access. To keep track of the state of the wrapper, we also
/// store an extra byte. On each access, we read this byte and swap it with `2`. If the
/// byte was `0` before, we initialise the per-cpu, else just obtain a reference to the
/// containing value using the self-referencing wrapper.
pub struct PerCpu<T: 'static> {
    inner: unsafe fn() -> Option<&'static T>,
}

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
        unsafe {
            let percpu = (self.inner)().ok_or(Error::Access)?;
            Ok(f(percpu))
        }
    }
}

impl<T: 'static> PerCpu<Cell<T>> {}

impl<T: 'static> PerCpu<RefCell<T>> {}

/// Declare a new per-cpu variable.
#[macro_export]
macro_rules! percpu {
    () => {};

    // process multiple declarations
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => {
        $crate::__percpu_inner!($(#[$attr])* $vis $name, $t, $init);
        $crate::percpu!($($rest)*);
    };

    // handle a single declaration
    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr) => {
        $crate::__percpu_inner!($(#[$attr])* $vis $name, $t, $init);
    };
}

#[macro_export]
macro_rules! __percpu_inner {
    (@key $t:ty, $init:expr) => {
        {
            #[repr(C)]
            struct Wrapper<T: 'static> {
                /// A pointer to self.
                ///
                /// The purpose of this is so that we can use `%gs:0(__this_percpu_offset())`
                /// to quickly get a reference to the contained value. This should be safe
                /// since the wrapper never moves.
                this: *const Wrapper<T>,

                /// The containing value.
                val: T,
            }

            /// SAFETY: we know only *this* cpu has access to the data.
            unsafe impl<T: 'static> Sync for Wrapper<T> {}

            #[inline]
            const fn __init() -> $t {
                $init
            }

            /// Returns the offset for the variable.
            #[inline]
            fn __this_percpu_offset() -> u64 {
                // We only use this to reserve space in the percpu section and determine the offset
                // in the percpu block where we can access our copy of the variable. Accessing it
                // directly here will cause a pagefault at best.
                #[link_section = ".percpu"]
                static _THE_THING: Wrapper<$t> = Wrapper {
                    this: core::ptr::null(),
                    val: __init(),
                };

                // TODO: use expose_addr when strict provenance is more worked out.
                (&_THE_THING as *const _) as u64
            }

            /// Uninitialised.
            const STATE_UNINIT: u8 = 0;

            /// Partially initialised. This is for the BSP, for which the value is already
            /// set, but the 'this' pointer is incorrect.
            const STATE_PARTIAL: u8 = 1;

            /// Fully initialised.
            const STATE_INIT: u8 = 2;

            /// Retrieve the current state, while simultaneously setting the current
            /// state to STATE_INIT. It's important the variable is initialised
            /// properly after this call depending on the returned state!
            ///
            /// # Safety
            /// The variable *must* be initialised if the result of this function is
            /// STATE_UNINIT or STATE_PARTIAL!
            #[inline]
            unsafe fn __state() -> u8 {
                // Keep track if this copy of the variable is initialised or not.
                #[link_section = ".percpu"]
                static STATE: u8 = STATE_PARTIAL;

                let mut state: u8 = STATE_INIT;
                core::arch::asm!(
                    "xchg   %gs:0({}), {}",
                    in(reg) (&STATE as *const _) as u64, inout(reg_byte) state,
                    options(att_syntax, nostack, preserves_flags, readonly),
                );
                state
            }

            /// Retrieve the per-cpu value and lazily initialise it if necessary.
            ///
            /// # Safety
            /// This code depends on a couple things:
            ///  - gsbase must point to the start of the per-cpu block for the current cpu.
            ///    These blocks are not allowed to overlap with each other in any case!
            ///  - The per-cpu block must be zeroed for non-BSP cpu's.
            unsafe fn __percpu() -> Option<&'static $t> {
                let state = __state();

                // Retrieve a pointer to our copy of the variable.
                let the_thing: *const Wrapper<$t> = {
                    if state < STATE_INIT {
                        use x86::bits64::segmentation::rdgsbase;

                        // We're not fully initialised yet, so we can't rely on 'this' in the wrapper.
                        // So, just compute the address where our copy is stored.
                        // This should be safe to do as long as gs.base points to the correct address.
                        let percpu_addr = rdgsbase() + __this_percpu_offset();

                        let raw_ptr = (percpu_addr as *mut Wrapper<$t>);
                        if state == STATE_UNINIT {
                            // We're a new copy. So, initialise ourself.
                            raw_ptr.write(Wrapper {
                                this: raw_ptr,
                                val: __init()
                            })
                        } else { // state == STATE_PARTIAL
                            (*raw_ptr).this = raw_ptr;
                        }

                        raw_ptr as *const _
                    } else {
                        let this: u64;
                        core::arch::asm!(
                            "mov %gs:0({}), {}",
                            in(reg) __this_percpu_offset(), lateout(reg) this,
                            options(att_syntax, nostack, preserves_flags, pure, readonly),
                        );

                        this as *const _
                    }
                };

                // The code above made sure we can now retrieve the correct pointer to ourselves.
                Some(&the_thing.as_ref().unwrap().val)
            }

            unsafe {
                $crate::percpu::PerCpu::new(__percpu)
            }
        }
    };

    ($(#[$attr:meta])* $vis:vis $name:ident, $t:ty, $($init:tt)*) => {
        $(#[$attr])* $vis const $name: $crate::percpu::PerCpu<$t> =
            $crate::__percpu_inner!(@key $t, $($init)*);
    }
}

/// Initialise the per-CPU for the current CPU.
pub fn init() {}
