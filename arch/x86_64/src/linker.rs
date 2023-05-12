//! This module provides Rust access to the values defined in the linker
//! script. Constants defined in `kernel-x86_64.lds` are just copied here.
//! Values that are computed during linkage are accessible through functions.

use crate::mm::paging;

/// The virtual offset of the kernel, mapped to hardware address 0.
pub const VIRT_OFFSET: u64 = 0xffffffff80000000;

/// The physical start address.
pub const KERNEL_PHYS_START: u64 = 0x1000000;

/// The virtual address of the kernel start.
pub const KERNEL_START: u64 = KERNEL_PHYS_START + VIRT_OFFSET;

/// The size of the virtual memory block reserved for the kernel. It does *not*
/// represent the *actual* size of the kernel image.
pub const KERNEL_SIZE: usize = 512 * paging::MEGABYTE;

/// The virtual address at which the 4K APIC registers will be mapped.
pub const APIC_OFFSET: u64 = KERNEL_START + KERNEL_SIZE as u64;

/// The maximum number of supported (logical) CPUs.
///
/// The page tables for per-CPU data are statically allocated. Lowering this
/// value will decrease the kernels' memory footprint.
pub const MAX_CPUS: usize = 16;

/// The virtual offset of the per-cpu data.
pub const PERCPU_OFFSET: u64 = 0xffffff8000000000;

/// The virtual offset of where the physical memory will be mapped to.
pub const PHYS_OFFSET: u64 = 0xffff800000000000;

/// The maximum amount of supported physical memory.
///
/// All physical memory is mapped into kernel space using 1G pages, since we
/// need at least 1 table (which can contain 512 entries, each entry able to map
/// 1G of memory), it makes sense to make this value a multiple of 512G.
///
/// Note that this value is not allowed to exceed 64TB!
pub const MAX_PHYS_MEMORY: usize = 512 * paging::GIGABYTE;

macro_rules! __linker_fn {
    (
        $(
            $(#[$($attr:tt)*])*
            $fn:ident() -> u64
        )*
    ) => {
        $(
            $(#[$($attr)*])*
            pub fn $fn() -> u64 {
                mod internal {
                    extern "C" {
                        pub static $fn: u8;
                    }
                }
                // SAFETY: '$fn' should be provided by the linker script.
                unsafe { &internal::$fn as *const _ as u64 }
            }
        )*
    };
}

__linker_fn!(
    #[doc = "Return the virtual start address of the text section.
    # Unsafety
    This function depends on `_text` in `kernel-x86_64.lds`."]
    _text() -> u64

    #[doc = "Return the virtual address of the end of the text section.
    # Unsafety
    This function depends on `_etext` in `kernel-x86_64.lds`."]
    _etext() -> u64

    #[doc = "Return the virtual address of the start of the rodata section.
    # Unsafety
    This function depends on `_rodata` in `kernel-x86_64.lds`."]
    _rodata() -> u64

    #[doc = "Return the virtual address of the end of the rodata section.
    # Unsafety
    This function depends on `_erodata` in `kernel-x86_64.lds`."]
    _erodata() -> u64

    #[doc = "Return the virtual address of the start of the data section.
    # Unsafety
    This function depends on `_data` in `kernel-x86_64.lds`."]
    _data() -> u64

    #[doc = "Return the virtual address of the end of the data section.
    # Unsafety
    This function depends on `_edata` in `kernel-x86_64.lds`."]
    _edata() -> u64

    #[doc = "Return the virtual address of the start of the bss section.
    # Unsafety
    This function depends on `_ebss` in `kernel-x86_64.lds`."]
    _bss() -> u64

    #[doc = "Return the virtual address of the end of the bss section.
    # Unsafety
    This function depends on `_ebss` in `kernel-x86_64.lds`."]
    _ebss() -> u64

    #[doc = "Return the virtual address of the start of the percpu section.
    # Unsafety
    This function depends on `_percpu` in `kernel-x86_64.lds`."]
    _percpu() -> u64

    #[doc = "Return the virtual address of the end of the percpu section.
    # Unsafety
    This function depends on `_epercpu` in `kernel-x86_64.lds`."]
    _epercpu() -> u64

    #[doc = "Return the virtual end of the kernel image.
    # Unsafety
    This function depends on `_end` in `kernel-x86_64.lds`."]
    _end() -> u64
);
