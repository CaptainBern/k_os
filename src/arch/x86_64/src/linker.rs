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

/// The size of the virtual memory block occupied by the kernel. It does *not*
/// represent the *actual* size of the kernel image.
pub const KERNEL_SIZE: usize = 512 * paging::MEGABYTE;

/// The size of the address space window.
pub const ASPACE_WINDOW_SIZE: usize = 2 * paging::GIGABYTE;

/// Virtual start address of the (global) address space window.
pub const ASPACE_WINDOW_START: u64 = 0xffffff8000000000;

/// The start address of the local-storage space. (entry 510 in PML4).
pub const ASPACE_LOCAL_START: u64 = 0xffffff0000000000;

macro_rules! __linker_fn {
    (
        $(
            $(#[$($attr:tt)*])*
            $fn:ident() -> u64
        )*
    ) => {
        $(
            $(#[$($attr)*])*
            pub unsafe fn $fn() -> u64 {
                mod internal {
                    extern "C" {
                        pub static $fn: u8;
                    }
                }
                &internal::$fn as *const _ as u64
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

    #[doc = "Return the virtual address of the start of the CPU local section.
    # Unsafety
    This function depends on `_cpulocal` in `kernel-x86_64.lds`."]
    _cpulocal_load_addr() -> u64

    #[doc = "Return the virtual address of the end of the CPU local section.
    # Unsafety
    This function depends on `_ecpulocal` in `kernel-x86_64.lds`."]
    _ecpulocal_load_addr() -> u64

    #[doc = "Return the virtual end of the kernel image.
    # Unsafety
    This function depends on `_end` in `kernel-x86_64.lds`."]
    _end() -> u64
);
