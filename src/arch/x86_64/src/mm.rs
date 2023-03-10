//! Kernel memory management.
//!
//! ## Early
//!
//! During the 'early' phase of the boot process (after we come from assembly),
//! we're still using temporary pages. Under these pages, the first 4G of physical
//! memory are identity mapped, while the upper 2G of the virtual address space
//! are mapped to the first 2G of physical address space. This is sufficient to
//! get us running in the higher-half of the virtual address space.
//!
//! Once we're up and running, the first step is to parse the memory map as provided
//! by the bootloader, and discover how many cores are available. For each of the
//! cores, we set aside some space for CPU local storage. Then we're finally
//! ready to setup the mappings for the boot processor, and actually get the system
//! up and running.

use core::{
    ops::Range,
    panic,
    ptr::{self},
};

use x86::controlregs::cr3_write;

use crate::{
    linker,
    paging::{self, PDEFlags, PDPTEFlags, PML4EFlags, PD, PDE, PDPT, PDPTE, PML4, PML4E},
    println,
};

/// *The* global kernel map, used by each address space.
static mut KERNEL_MAP: KernelMap = KernelMap::NULL;

/// Memory region kinds.
///
/// Standard E820 region 'types'. See the [osdev wiki](https://wiki.osdev.org/Detecting_Memory_(x86)).
#[derive(Debug)]
#[repr(u8)]
pub enum RegionKind {
    Usable = 0,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    Defective,
}

/// A (physical) memory region.
///
/// Memory regions have a base address, length, and a kind. This is based directly on the E820
/// memory description as described [here](https://wiki.osdev.org/Detecting_Memory_(x86)).
#[derive(Debug)]
pub struct Region {
    pub base: u64,
    pub length: u64,
    pub kind: RegionKind,
}

/// The global kernel map.
///
/// The map handles the upper 512G of virtual memory. All other kernel addresses (meaning
/// address of which the most significant bit is '1') are avilable to the [AddressSpaces](AddressSpace).
/// It also manages the physical memory owned by the kernel. All other physical memory is
/// managed by the userspace 'paging' server. It is that server's responsibility to make
/// sure the [AddressSpaces](AddressSpace) don't clash (by mapping two different processes
/// to the same physical frame for example).
#[derive(Debug)]
pub struct KernelMap {
    /// Level-3 table.
    ///
    /// Each [AddressSpace] has entry #511 of its top (PML4) pointing to this table.
    /// This table is used to map common kernel memory across all address spaces.
    lvl_3: PDPT,

    /// Level-2 kernel table.
    ///
    /// This table maintains the 1G window (starting at 0xffffffff80000000) in which
    /// the kernel text and data live.
    kern_lvl_2: PD,

    /// Level-2 memory map.
    ///
    /// A big table used to map 16G of memory. The address-spaces themselves live inside this region.
    /// The 64G is a pretty arbitrary number. It should be plent
    mem_lvl_2: [PD; 64],
}

impl KernelMap {
    /// The virtual address range of the kernel map.
    const KERNEL_ADDRESS_RANGE: Range<usize> =
        linker::VIRT_OFFSET as usize..(linker::VIRT_OFFSET + linker::KERNEL_SIZE) as usize;

    /// Default 'null' kernel map.
    pub const NULL: KernelMap = KernelMap {
        lvl_3: [PDPTE::NULL; 512],
        kern_lvl_2: [PDE::NULL; 512],
        mem_lvl_2: [[PDE::NULL; 512]; 64],
    };

    /// Initialise the kernel map.
    pub fn init(&mut self) {
        assert!(
            linker::KERNEL_SIZE <= paging::GIGABYTE,
            "KERNEL_SIZE too big!"
        );

        let bios_mem = linker::VIRT_OFFSET as usize..linker::KERNEL_START as usize;
        let text = unsafe { linker::_text() as usize..linker::_etext() as usize };
        let rodata = unsafe { linker::_rodata() as usize..linker::_erodata() as usize };
        let data = unsafe { linker::_data() as usize..linker::_edata() as usize };
        let bss = unsafe { linker::_bss() as usize..linker::_ebss() as usize };

        self.map(bios_mem.start as u64, 0, bios_mem.len());

        self.map(
            text.start as u64,
            virt_to_phys(text.start as u64),
            text.len(),
        );

        self.map(
            rodata.start as u64,
            virt_to_phys(rodata.start as u64),
            rodata.len(),
        );

        self.map(
            data.start as u64,
            virt_to_phys(data.start as u64),
            data.len(),
        );

        self.map(bss.start as u64, virt_to_phys(bss.start as u64), bss.len());
    }

    /// Allocate an address space.
    ///
    /// The returned address space, when successful, will reference the kernel map.
    fn allocate_address_space(&mut self) {}

    #[inline]
    pub const fn kernel_lvl_3(&self) -> &PDPT {
        &self.lvl_3
    }

    /// Map the given virtual address to the given physical address for the given size.
    ///
    /// Mapping a virtual address outside of [KernelMap::KERNEL_ADDRESS_RANGE] is not allowed and will
    /// cause a panic. Furthermore, remapping of virtual addresses to a different physical address is also
    /// not allowed and will panic.
    /// Mapping an existing virtual address to the same physical address is allowed.
    fn map(&mut self, mut virt: u64, mut phys: u64, len: usize) {
        fn is_range_valid(range: &Range<usize>) -> bool {
            range.start >= KernelMap::KERNEL_ADDRESS_RANGE.start
                && range.start <= KernelMap::KERNEL_ADDRESS_RANGE.end
                && range.end >= KernelMap::KERNEL_ADDRESS_RANGE.start
                && range.end <= KernelMap::KERNEL_ADDRESS_RANGE.end
        }

        // Make sure the requested range fits in the kernel map.
        let range = virt as usize..virt as usize + len;

        if !is_range_valid(&range) {
            panic!(
                "KernelMap: requested map ({:?}) falls outside of the valid range!",
                range
            );
        }

        // Fixup the addresses so they are aligned on the correct page boundary.
        virt = align_down::<{ paging::LARGE_PAGE }>(virt);
        phys = align_down::<{ paging::LARGE_PAGE }>(phys);

        // Point the level-3 entry to the level-2 table. Panics if the entry is pointing to the wrong table (which would be a bug).
        let entry = &mut self.lvl_3[paging::pdpt_index(virt)];
        if *entry == PDPTE::NULL {
            entry.set_address(virt_to_phys(self.kern_lvl_2.as_ptr() as _));
            entry.set_flags(PDPTEFlags::P | PDPTEFlags::RW);
        } else if entry.address() != virt_to_phys(self.kern_lvl_2.as_ptr() as _) {
            panic!("KernelMap: level-3 entry pointing to an unexpected level-2 table ({:#018x})! This is a bug.", entry.address());
        }

        let frames = align_up::<{ paging::LARGE_PAGE }>(len as u64) / paging::LARGE_PAGE;
        for _ in 0..frames {
            // Point the level-2 entry to the correct frame. Panics in case the entry is already mapped to a different frame.
            let entry = &mut self.kern_lvl_2[paging::pd_index(virt)];
            if *entry == PDE::NULL {
                entry.set_address(phys);
                entry.set_flags(PDEFlags::P | PDEFlags::RW | PDEFlags::PS)
            } else if entry.address() != phys {
                panic!("KernelMap: requested map ({:#018x} -> {:#018x}) is already mapped to a different address ({:#018x})", virt, phys, entry.address());
            }

            phys += paging::LARGE_PAGE;
            virt += paging::LARGE_PAGE;
        }
    }
}

/// Per-core address space.
///
/// Every logical core has its own address space, and keeps track of its own per-core storage (TLS-like).
/// The linker script mandates that the kernel image size cannot exceed 512M. This *includes* the address
/// space and other per-core storage. Every address space maps itself to `0xffffbf8000000000` (see
/// linker script). The physical frames it uses are handled by the kernel.
#[derive(Debug)]
#[repr(C, align(4096))]
pub struct AddressSpace {
    /// Reference to self.
    ///
    /// This is a *physical* pointer to self.
    this: *const AddressSpace,

    /// APIC ID of the owning CPU.
    apic_id: u32,

    /// The top of the address space for the owning CPU.
    top: PML4,

    /// The level 3 local table used.
    local_lvl_3: PDPT,
    local_lvl_2: PD,
}

impl AddressSpace {
    pub const NULL: AddressSpace = AddressSpace {
        this: ptr::null(),
        apic_id: 0,
        top: [PML4E::NULL; 512],
        local_lvl_3: [PDPTE::NULL; 512],
        local_lvl_2: [PDE::NULL; 512],
    };

    /// Initialise the address space.
    pub fn init(&mut self) {}
}

/// Convert a virtual (kernel) address to a physical address.
///
/// If the given address is not in the kernel address space range, a panic will occur.
#[inline]
pub fn virt_to_phys(virt: u64) -> u64 {
    assert!(KernelMap::KERNEL_ADDRESS_RANGE.contains(&(virt as usize)));
    virt - linker::VIRT_OFFSET
}

/// Returns true if `addr` is aligned on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn is_aligned<const ALIGNMENT: u64>(addr: u64) -> bool {
    assert!(ALIGNMENT.is_power_of_two());
    addr & ((1 << ALIGNMENT.trailing_zeros()) - 1) == 0
}

/// Align `addr` down on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn align_down<const ALIGNMENT: u64>(addr: u64) -> u64 {
    assert!(ALIGNMENT.is_power_of_two());
    addr & !((1 << ALIGNMENT.trailing_zeros()) - 1)
}

/// Align `addr` up on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn align_up<const ALIGNMENT: u64>(addr: u64) -> u64 {
    assert!(ALIGNMENT.is_power_of_two());
    (addr + ALIGNMENT - 1) & !((1 << ALIGNMENT.trailing_zeros()) - 1)
}

static mut TOP: PML4 = [PML4E::NULL; 512];

/// To initialise our kernel mappings:
///  - find out how many CPUs there are, for each CPU we need to allocate a 'local' memory space.
///  - calculate the total amount of memory required to for all 'local' memory spaces. This also includes
///     'cpu-local' storage.
///  - find a contiguous chunk of physical memory that fits all
///  - somehow mark that region + the kernel region as 'reserved' (as all the other reserved/unusable regions)
///
/// So:
///  - copy localspace for each CPU to some memory region
///  - setup cpu-local access shit, so we can load LOCAL_SPACE.top into cr3 for each core.
pub unsafe fn init_early() {
    KERNEL_MAP.init();
    let kernel_start = paging::pml4_index(linker::VIRT_OFFSET);
    TOP[kernel_start] = PML4E::new(virt_to_phys(KERNEL_MAP.lvl_3.as_ptr() as _), PML4EFlags::P);
    cr3_write(virt_to_phys(TOP.as_ptr() as _));
    println!("We're on the kernel map now!");
}
