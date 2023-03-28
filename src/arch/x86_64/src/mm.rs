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
    alloc::Layout,
    cmp::{self, Ordering},
    ops::Range,
    panic, slice,
};

use heapless::Vec;
use itertools::Itertools;
use x86::controlregs::cr3_write;

use crate::{
    linker,
    paging::{
        self, pd_index, pdpt_index, pml4_index, pt_index, PDEFlags, PDPTEFlags, PML4EFlags,
        PTEFlags, PD, PDE, PDPT, PDPTE, PML4, PML4E, PT, PTE,
    },
    println, BootInfo,
};

static mut KERNEL_MAP: KernelMap = KernelMap::empty();

/// A memory region.
///
/// Regions can either be physical or virtual, it is up to the user to make clear which
/// one it is.
#[derive(Debug, PartialEq, Eq, Ord, Clone, Copy)]
pub struct Region {
    pub base: u64,
    pub length: u64,
}

impl Region {
    /// Calculate the end address of the region.
    #[inline]
    pub fn end(&self) -> u64 {
        self.base + self.length
    }

    /// Return true if `x` and `y` have any overlap.
    #[inline]
    pub fn are_overlapping(x: &Region, y: &Region) -> bool {
        (x.base <= y.end()) && (x.end() >= y.base)
    }

    /// Merge two regions.
    ///
    /// Returns [None] if the regions are not overlapping or are not of the same [RegionKind].
    #[inline]
    pub fn merge(x: &Region, y: &Region) -> Option<Region> {
        if Region::are_overlapping(x, y) {
            let base = cmp::min(x.base, y.base);
            let length = cmp::max(x.end(), y.end()) - base;
            Some(Region { base, length })
        } else {
            None
        }
    }
}

impl PartialOrd for Region {
    /// Compare `self` with `other.`
    /// * `self < other` if `(self.base < other.base) || (self.base == other.base && self.length < other.length)`
    /// * `self == other` if `self.base == other.base && self.length == other.length`
    /// * `self > other` if `(self.base > other.base) || (self.base == other.base && self.length > other.length)`
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.base.cmp(&other.base) {
            Ordering::Equal => Some(self.length.cmp(&other.length)),
            ordering => Some(ordering),
        }
    }
}

/// Memory kind.
///
/// Standard E820 memory 'types'. See the [osdev wiki](https://wiki.osdev.org/Detecting_Memory_(x86)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MemoryKind {
    Usable = 0,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    Defective,
}

/// A memory descriptor.
///
/// This is based directly on the E820 memory description as described
/// [here](https://wiki.osdev.org/Detecting_Memory_(x86)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
pub struct MemoryDescriptor {
    pub region: Region,
    pub kind: MemoryKind,
}

impl MemoryDescriptor {
    /// Return true if the memory region is usable.
    ///
    /// This is just a shorthand to check if `self.kind` is [RegionKind::Usable].
    #[inline]
    pub fn is_usable(&self) -> bool {
        self.kind == MemoryKind::Usable
    }
}

impl PartialOrd for MemoryDescriptor {
    /// Memory descriptors are sorted according to the region they span.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.region.partial_cmp(&other.region)
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
    /// The top of the address space for the owning CPU.
    top: PML4,

    self_lvl_3: PDPT,

    self_lvl_2: PD,

    self_lvl_1: [PT; 256],

    /// Reference to self, used to find the physical location of self.
    /// It is relative to [linker::ASPACE_WINDOW_START].
    this: &'static AddressSpace,

    apic_id: i16,
}

impl AddressSpace {
    pub fn init(&mut self, kernel_map: &KernelMap, frame: u64) {
        // Just a sanity check to make sure we are effectively pointed at
        // the given frame.
        assert!(
            kernel_map
                .virt_to_phys(self.top.as_ptr() as u64)
                .unwrap_or_default()
                == frame
        );

        // TODO: do this in allocate_aspaces, or should we do it as we bring up each core?
    }

    /// Flush the address space.
    pub unsafe fn flush(&self) {
        // TODO: calculate physical address of self.top
        // we can use kernelmap for this.
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        panic!("AddressSpace not supposed to be dropped!");
    }
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
#[repr(C, align(4096))]
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

    /// Level-2 address space tables.
    ///
    /// These tables are mapped to the address spaces.
    aspace_lvl_2: [PD; linker::ASPACE_WINDOW_SIZE / paging::GIGABYTE],

    /// Level-3 address space tables.
    ///
    /// Address spaces are maped in 4k blocks to save as much memory.
    aspace_lvl_1: [PT; 512 * (linker::ASPACE_WINDOW_SIZE / paging::GIGABYTE)],

    /// The number of allocated address spaces.
    num_aspaces: u16,
}

impl KernelMap {
    /// The virtual address range of the kernel. This is the area in which
    /// global kernel text and data live.
    const KERN_VIRT_ADDR_RANGE: Range<u64> =
        linker::VIRT_OFFSET..(linker::VIRT_OFFSET + linker::KERNEL_SIZE as u64);

    /// Create an empty kernel map.
    #[inline]
    pub const fn empty() -> Self {
        assert!(
            is_aligned::<{ paging::GIGABYTE }>(linker::ASPACE_WINDOW_SIZE as u64),
            "ASPACE_WINDOW_SIZE must be a multiple of paging::GIGABYTE!",
        );

        KernelMap {
            lvl_3: [PDPTE::NULL; 512],
            kern_lvl_2: [PDE::NULL; 512],
            aspace_lvl_2: [[PDE::NULL; 512]; linker::ASPACE_WINDOW_SIZE / paging::GIGABYTE],
            aspace_lvl_1: [[PTE::NULL; 512]; 512 * (linker::ASPACE_WINDOW_SIZE / paging::GIGABYTE)],
            num_aspaces: 0,
        }
    }

    /// Initialise the kernel map.
    pub fn init(&mut self) {
        assert!(
            linker::KERNEL_SIZE <= paging::GIGABYTE,
            "KERNEL_SIZE too big!"
        );
        assert!(
            linker::ASPACE_WINDOW_SIZE <= (64 * paging::GIGABYTE),
            "ASPACE_WINDOW_SIZE must be smaller than 64G!"
        );

        let bios_mem = linker::VIRT_OFFSET as usize..linker::KERNEL_START as usize;
        let text = unsafe { linker::_text() as usize..linker::_etext() as usize };
        let rodata = unsafe { linker::_rodata() as usize..linker::_erodata() as usize };
        let data = unsafe { linker::_data() as usize..linker::_edata() as usize };
        let bss = unsafe { linker::_bss() as usize..linker::_ebss() as usize };

        self.map_kern(bios_mem.start as u64, 0, bios_mem.len());

        self.map_kern(
            text.start as u64,
            virt_to_phys(text.start as u64),
            text.len(),
        );

        self.map_kern(
            rodata.start as u64,
            virt_to_phys(rodata.start as u64),
            rodata.len(),
        );

        self.map_kern(
            data.start as u64,
            virt_to_phys(data.start as u64),
            data.len(),
        );

        self.map_kern(bss.start as u64, virt_to_phys(bss.start as u64), bss.len());
    }

    #[inline]
    pub const fn kernel_lvl_3(&self) -> &PDPT {
        &self.lvl_3
    }

    /// Translate the given virtual address to its physical address.
    ///
    /// This function only works for addresses that are within the kernel map. It
    /// will return `None` in case the given virtual address is not part of the
    /// kernel map, or is not mapped.
    pub fn virt_to_phys(&self, virt: u64) -> Option<u64> {
        if linker::VIRT_OFFSET <= virt && virt <= unsafe { linker::_end() } {
            Some(virt - linker::VIRT_OFFSET)
        } else if linker::ASPACE_WINDOW_START <= virt
            && virt <= (linker::ASPACE_WINDOW_START + linker::ASPACE_WINDOW_SIZE as u64)
        {
            #[inline]
            fn phys_to_virt(phys: u64) -> u64 {
                phys + linker::VIRT_OFFSET
            }

            #[inline]
            fn index(phys: u64, offset: u64) -> usize {
                ((phys_to_virt(phys) - offset) / 4096) as usize
            }

            let lvl_3_entry = &self.lvl_3[pdpt_index(virt)];
            if !lvl_3_entry.flags().contains(PDPTEFlags::P) {
                return None;
            }

            let lvl_2_entry = {
                let phys = lvl_3_entry.address();
                let offset = self.aspace_lvl_2.as_ptr() as u64;
                &self.aspace_lvl_2[index(phys, offset)][pd_index(virt)]
            };

            if !lvl_2_entry.flags().contains(PDEFlags::P) {
                return None;
            }

            let lvl_1_entry = {
                let phys = lvl_2_entry.address();
                let offset = self.aspace_lvl_1.as_ptr() as u64;
                &self.aspace_lvl_1[index(phys, offset)][pt_index(virt)]
            };

            if !lvl_1_entry.flags().contains(PTEFlags::P) {
                return None;
            }

            Some(lvl_1_entry.frame())
        } else {
            None
        }
    }

    /// Allocate a number of address spaces.
    ///
    /// Address spaces will be allocated within the given memory regions.
    ///
    /// * `mem_info` - A *sorted* vec of [Regions](Region).
    /// * `num` - The number of spaces that should be allocated.
    pub unsafe fn allocate_aspaces(&mut self, mem_info: &Vec<MemoryDescriptor, 32>, mut num: u16) {
        let layout = Layout::new::<AddressSpace>();

        assert!(
            layout.align() == paging::BASE_PAGE,
            "AddressSpace has wrong alignment!"
        );
        assert!(
            (layout.size() * num as usize) <= linker::ASPACE_WINDOW_SIZE,
            "ASPACE_WINDOW_SIZE is too small!"
        );

        // The physical kernel region. We count on `base` being 0 further down when
        // coalescing the memory regions.
        let kernel_region = &Region {
            base: 0,
            length: virt_to_phys(linker::_end()),
        };

        // Compute the usable memory regions. These are the regions in which we can allocate
        // the address spaces. Any overlapping regions are merged, and each region is aligned
        // if necessary.
        let regions: Vec<Region, 32> = mem_info
            .iter()
            .filter_map(|x| {
                if x.is_usable() {
                    Some(x.region.clone())
                } else {
                    None
                }
            })
            .coalesce(|x, y| {
                if Region::are_overlapping(&x, &y) {
                    Ok(Region::merge(&x, &y).unwrap())
                } else {
                    Err((x, y))
                }
            })
            .filter_map(|x| {
                if x.end() < kernel_region.end() || x.end() - kernel_region.end() == 0 {
                    // Kernel region starts at 0, so if this region ends before the kernel
                    // end, it is fully inside (reserved) kernel space OR if we do adjust
                    // the region, it will be 0-length, in which case we discard it.
                    None
                } else if x.base < kernel_region.end() {
                    // Region falls partly inside the kernel region, so adjust its base and
                    // length to sit beyond kernel memory.
                    Some(Region {
                        base: kernel_region.end(),
                        length: x.end() - kernel_region.end(),
                    })
                } else {
                    // The region is fine, so just return it.
                    Some(x)
                }
            })
            .filter_map(|x| {
                if !is_aligned::<{ paging::BASE_PAGE }>(x.base) {
                    let diff = align_up::<{ paging::BASE_PAGE }>(x.base) - x.base;
                    if (x.length - diff) > 0 {
                        Some(Region {
                            base: x.base + diff,
                            length: x.length - diff,
                        })
                    } else {
                        None
                    }
                } else {
                    Some(x)
                }
            })
            .collect();

        let mut virt = linker::ASPACE_WINDOW_START;
        assert!(
            is_aligned::<{ paging::BASE_PAGE }>(virt as _),
            "ASPACE_WINDOW_START not properly aligned!"
        );
        assert!(
            pml4_index(virt as _) == 511,
            "ASPACE_WINDOW_START must start at index #0 of the PML4!"
        );
        assert!(
            pdpt_index(virt as _) == 0,
            "ASPACE_WINDOW_START must start at index #0 of the PDPT!"
        );

        (0..(linker::ASPACE_WINDOW_SIZE / paging::GIGABYTE)).for_each(|i| {
            let virt = virt + (i * paging::PD_COVERAGE) as u64;
            let lvl3 = &mut self.lvl_3[pdpt_index(virt as _)];
            lvl3.set_address(virt_to_phys(self.aspace_lvl_2[i].as_ptr() as _));
            lvl3.set_flags(PDPTEFlags::P);
        });

        let num_frames = layout.size() / paging::BASE_PAGE;

        for region in regions.iter() {
            if num == 0 {
                break;
            }

            if (region.length as usize) < layout.size() {
                continue;
            }

            let num_spaces = cmp::min(num as usize, region.length as usize / layout.size());
            let frame_base = align_up::<{ paging::BASE_PAGE }>(region.base);

            for x in 0..num_spaces {
                for y in 0..num_frames {
                    let delta = ((x * layout.size()) + (y * paging::BASE_PAGE)) as u64;

                    let virt = (virt + delta) as u64;
                    let frame = (frame_base + delta) as u64;

                    let lvl2 = &mut self.aspace_lvl_2[pdpt_index(virt)][pd_index(virt)];
                    if *lvl2 == PDE::NULL {
                        lvl2.set_address(virt_to_phys(
                            self.aspace_lvl_1[pd_index(virt)].as_ptr() as _
                        ));
                        lvl2.set_flags(PDEFlags::P | PDEFlags::RW);
                    }

                    let lvl1 = &mut self.aspace_lvl_1[pd_index(virt)][pt_index(virt)];
                    lvl1.set_frame(frame);
                    lvl1.set_flags(PTEFlags::P | PTEFlags::RW);
                }
                self.num_aspaces += 1;
            }

            virt += (num_spaces * layout.size()) as u64;
            num -= num_spaces as u16;
        }

        // TODO: Is a panic necessary here? We could keep going with the spaces we have and just not use
        // the remaining cores.
        if num != 0 {
            panic!(
                "Failed to allocate address spaces! (allocated {}/{})",
                self.num_aspaces,
                self.num_aspaces + num
            );
        }

        let raw = slice::from_raw_parts_mut(
            linker::ASPACE_WINDOW_START as *mut u8,
            self.num_aspaces as usize * layout.size(),
        );

        // Zero the memory just in case...
        raw.fill(0u8);
    }

    /// Map the given virtual address to the given physical address for the given size.
    ///
    /// Mapping a virtual address outside of [KernelMap::KERNEL_ADDRESS_RANGE] is not allowed and will
    /// cause a panic. Furthermore, remapping of virtual addresses to a different physical address is also
    /// not allowed and will panic.
    /// Mapping an existing virtual address to the same physical address is allowed.
    fn map_kern(&mut self, mut virt: u64, mut phys: u64, len: usize) {
        fn is_range_valid(range: &Range<u64>) -> bool {
            range.start >= KernelMap::KERN_VIRT_ADDR_RANGE.start
                && range.start <= KernelMap::KERN_VIRT_ADDR_RANGE.end
                && range.end >= KernelMap::KERN_VIRT_ADDR_RANGE.start
                && range.end <= KernelMap::KERN_VIRT_ADDR_RANGE.end
        }

        // Make sure the requested range fits in the kernel map.
        let range = virt..virt + len as u64;

        assert!(
            is_range_valid(&range),
            "KernelMap: requested map ({:?}) has an invalid range.",
            range
        );

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

        let frames = align_up::<{ paging::LARGE_PAGE }>(len as u64) / paging::LARGE_PAGE as u64;
        for _ in 0..frames {
            // Point the level-2 entry to the correct frame. Panics in case the entry is already mapped to a different frame.
            let entry = &mut self.kern_lvl_2[paging::pd_index(virt)];
            if *entry == PDE::NULL {
                entry.set_address(phys);
                entry.set_flags(PDEFlags::P | PDEFlags::RW | PDEFlags::PS)
            } else if entry.address() != phys {
                panic!("KernelMap: requested map ({:#018x} -> {:#018x}) is already mapped to a different address ({:#018x})", virt, phys, entry.address());
            }

            phys += paging::LARGE_PAGE as u64;
            virt += paging::LARGE_PAGE as u64;
        }
    }
}

/// Convert a virtual (kernel) address to a physical address.
///
/// If the given address is not in the kernel address space range, a panic will occur.
#[inline]
pub const fn virt_to_phys(virt: u64) -> u64 {
    // waiting on const_range_bounds to become stable. (#108082)
    assert!(
        KernelMap::KERN_VIRT_ADDR_RANGE.start <= virt
            && virt <= KernelMap::KERN_VIRT_ADDR_RANGE.end
    );
    virt - linker::VIRT_OFFSET as u64
}

/// Returns true if `addr` is aligned on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn is_aligned<const ALIGNMENT: usize>(addr: u64) -> bool {
    assert!(ALIGNMENT.is_power_of_two());
    addr & ((1 << ALIGNMENT.trailing_zeros()) - 1) == 0
}

/// Align `addr` down on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn align_down<const ALIGNMENT: usize>(addr: u64) -> u64 {
    assert!(ALIGNMENT.is_power_of_two());
    addr & !((1 << ALIGNMENT.trailing_zeros()) - 1)
}

/// Align `addr` up on `ALIGNMENT`.
///
/// `ALIGNMENT` should be a power of two.
#[inline]
pub const fn align_up<const ALIGNMENT: usize>(addr: u64) -> u64 {
    assert!(ALIGNMENT.is_power_of_two());
    (addr + ALIGNMENT as u64 - 1) & !((1 << ALIGNMENT.trailing_zeros()) - 1)
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
pub unsafe fn init_early(boot_info: &BootInfo) {
    // TODO:
    // - setup KernelMap, with a temporary PML4
    // - use temp shit to setup address spaces.
    KERNEL_MAP.init();

    // setup the temporary TOP
    TOP[511] = PML4E::new(
        virt_to_phys(KERNEL_MAP.kernel_lvl_3().as_ptr() as _),
        PML4EFlags::P,
    );

    // switch over to temporary mappings
    cr3_write(virt_to_phys(TOP.as_ptr() as _));

    // This is just POC, `num` should be the number of cores discovered with ACPI.
    KERNEL_MAP.allocate_aspaces(&boot_info.mem_descriptors, 4);
}
