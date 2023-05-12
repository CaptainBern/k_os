use core::{marker::PhantomPinned, mem::MaybeUninit, pin::Pin};

use x86::gs_deref;

use crate::{linker, println};

#[derive(Debug)]
#[repr(C)]
pub struct PerCpu {
    /// Reference to self.
    pub this: MaybeUninit<&'static Self>,

    /// (x2)APIC ID of the CPU.
    pub apic_id: u32,

    /// We are unmovable.
    pub _pin: PhantomPinned,
}

impl PerCpu {
    pub fn new(apic_id: u32) -> Self {
        unsafe {
            PerCpu {
                this: MaybeUninit::new((linker::PERCPU_OFFSET as *const PerCpu).as_ref().unwrap()),
                apic_id,
                _pin: PhantomPinned,
            }
        }
    }

    #[inline]
    pub const fn apic_id(&self) -> u32 {
        self.apic_id
    }
}

/// Retrieve a pinned reference to the [PerCpu] for the current CPU.
///
/// # Safety
/// The [PerCpu]s must have been allocated and setup properly. Calling this function
/// before that is done will result in UB.
#[inline]
pub unsafe fn current() -> Pin<&'static PerCpu> {
    let ptr = (gs_deref!(0) as *const MaybeUninit<&'static PerCpu>)
        .as_ref()
        .unwrap()
        .assume_init();
    Pin::static_ref(ptr)
}
