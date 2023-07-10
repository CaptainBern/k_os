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

/// The maximum number of supported (logical) CPUs.
///
/// The page tables for per-CPU data are statically allocated. Lowering this
/// value will decrease the kernels' memory footprint.
pub const MAX_CPUS: usize = 16;

/// The virtual offset of the per-cpu data.
pub const PERCPU_OFFSET: u64 = 0xffffff8000000000;

/// The size of the kernel stack.
pub const STACK_SIZE: usize = 0x4000;

/// The size of the guard pages surrounding kernel stacks.
pub const STACK_GUARD_SIZE: usize = paging::BASE_PAGE;

/// The size of the interrupt stacks.
pub const INTERRUPT_STACK_SIZE: usize = 0x4000;

/// The virtual address offset at which kernel devices will be mapped.
pub const KDEV_OFFSET: u64 = 0xffffffffc0000000;

/// Virtual address where the local APIC mmio will be mapped.
pub const LOCAL_APIC_ADDRESS: u64 = KDEV_OFFSET;

/// The maximum number of IOAPICs.
pub const MAX_IOAPICS: usize = 1;

/// Offset of the IOAPICs.
pub const IO_APIC_OFFSET: u64 = LOCAL_APIC_ADDRESS + paging::BASE_PAGE as u64;

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
            $fn:ident() -> u64;
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
    #[doc = "Return the physical address of the boot16 code.
    # Safety
    This function depends on `_boot16` in `kernel-x86_64.lds`."]
    _boot16() -> u64;

    #[doc = "Return the physical address of end the boot16 code.
    # Safety
    This function depends on `_eboot16` in `kernel-x86_64.lds`."]
    _eboot16() -> u64;

    #[doc = "Return the virtual start address of the text section.
    # Safety
    This function depends on `_text` in `kernel-x86_64.lds`."]
    _text() -> u64;

    #[doc = "Return the virtual address of the end of the text section.
    # Safety
    This function depends on `_etext` in `kernel-x86_64.lds`."]
    _etext() -> u64;

    #[doc = "Return the virtual address of the start of the rodata section.
    # Safety
    This function depends on `_rodata` in `kernel-x86_64.lds`."]
    _rodata() -> u64;

    #[doc = "Return the virtual address of the end of the rodata section.
    # Safety
    This function depends on `_erodata` in `kernel-x86_64.lds`."]
    _erodata() -> u64;

    #[doc = "Return the virtual address of the start of the data section.
    # Safety
    This function depends on `_data` in `kernel-x86_64.lds`."]
    _data() -> u64;

    #[doc = "Return the virtual address of the end of the data section.
    # Ssafety
    This function depends on `_edata` in `kernel-x86_64.lds`."]
    _edata() -> u64;

    #[doc = "Return the virtual address of the start of the bss section.
    # Safety
    This function depends on `_ebss` in `kernel-x86_64.lds`."]
    _bss() -> u64;

    #[doc = "Return the virtual address of the end of the bss section.
    # Ssafety
    This function depends on `_ebss` in `kernel-x86_64.lds`."]
    _ebss() -> u64;

    #[doc = "Return the virtual address of the start of the percpu section.
    # Safety
    This function depends on `_percpu_load` in `kernel-x86_64.lds`."]
    _percpu_load() -> u64;

    #[doc = "Return the virtual address of the end of the percpu section.
    # Safety
    This function depends on `_epercpu_load` in `kernel-x86_64.lds`."]
    _epercpu_load() -> u64;

    #[doc = "Return the virtual end of the kernel image.
    # Safety
    This function depends on `_end` in `kernel-x86_64.lds`."]
    _end() -> u64;
);
