//! Kernel virtual memory management.

use core::{mem::MaybeUninit, panic};

use x86::controlregs::cr3_write;

use core::{alloc::Layout, cmp, ops::Range, slice};

use heapless::Vec;
use itertools::Itertools;

use crate::{
    linker,
    mm::{
        desc::{MemoryDescriptor, Region},
        paging::{
            self, align_down, align_up, is_aligned, pd_index, pdpt_index, pml4_index, pt_index,
            PDEFlags, PDPTEFlags, PML4EFlags, PTEFlags, PD, PDE, PDPT, PDPTE, PML4, PML4E, PT, PTE,
        },
    },
    println,
};

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
        assert!(
            is_aligned::<{ paging::GIGABYTE }>(linker::ASPACE_WINDOW_SIZE as u64),
            "ASPACE_WINDOW_SIZE must be a multiple of paging::GIGABYTE!",
        );

        let bios_mem = linker::VIRT_OFFSET as usize..linker::KERNEL_START as usize;
        let text = unsafe { linker::_text() as usize..linker::_etext() as usize };
        let rodata = unsafe { linker::_rodata() as usize..linker::_erodata() as usize };
        let data = unsafe { linker::_data() as usize..linker::_edata() as usize };
        let bss = unsafe { linker::_bss() as usize..linker::_ebss() as usize };
        let cpulocal = unsafe {
            linker::_cpulocal_load_addr() as usize..linker::_ecpulocal_load_addr() as usize
        };

        self.map(bios_mem.start as u64, 0, bios_mem.len());

        self.map(
            text.start as u64,
            self.virt_to_phys(text.start as u64).unwrap(),
            text.len(),
        );

        self.map(
            rodata.start as u64,
            self.virt_to_phys(rodata.start as u64).unwrap(),
            rodata.len(),
        );

        self.map(
            data.start as u64,
            self.virt_to_phys(data.start as u64).unwrap(),
            data.len(),
        );

        self.map(
            bss.start as u64,
            self.virt_to_phys(bss.start as u64).unwrap(),
            bss.len(),
        );

        // TODO: Map this to the BSP.
        self.map(
            cpulocal.start as u64,
            self.virt_to_phys(cpulocal.start as u64).unwrap(),
            cpulocal.len(),
        );
    }

    /// Return the physical address of the kernel PDPT.
    #[inline]
    pub fn kernel_lvl_3(&self) -> u64 {
        self.virt_to_phys(self.lvl_3.as_ptr() as u64).unwrap()
    }

    /// Translate the given virtual address to its physical address.
    ///
    /// This function only works for addresses that are within the kernel map. It
    /// will return `None` in case the given virtual address is not part of the
    /// kernel map, or is not mapped.
    ///
    /// Note that `self.map` counts on this functions ability to translate virtual
    /// kernel addresses (so not ASPACE_WINDOW addresses) without using tables!
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
    /// Address spaces will be allocated within the given memory regions. In memory, each address space
    /// is followed by a block used for CPU locals. Furthermore, each address space and its CPU local
    /// block is allocated as contiguous physical memory region:
    ///
    /// ```
    ///   +--------------------+--------------------+--------------------+--------------------+...
    ///   |     AddressSpace   |      CPU Local     |     AddressSpace   |      CPU Local     |
    ///   +--------------------+--------------------+--------------------+--------------------+...
    ///   \_________________________________________/                    |
    ///   |                    |         `- contiguous block of physical memory
    ///   \____________________/                    |                    |
    ///    |                                        \____________________/
    ///     `- starts at [linker::ASPACE_WINDOW_START] `- [linker::ASPACE_WINDOW_START] + sizeof(AddressSpace)
    /// ```
    ///
    /// In virtual memory, the spaces are mapped one after the other, starting at [linker::ASPACE_WINDOW_START].
    /// Note that this function only *allocates* the requested number of address spaces. It does not perform
    /// any initialization.
    ///
    /// # Safety
    /// The kernel map should be mapped at index 511 of the PML4! The allocation routine counts on mapped
    /// virtual blocks to be available immediately, and the 'virt_to_phys' function.
    ///
    /// * `mem_info` - A *sorted* vec of [Regions](Region).
    /// * `num` - The number of spaces that should be allocated.
    pub unsafe fn allocate_aspaces(&mut self, mem_info: &Vec<MemoryDescriptor, 32>, mut num: u16) {
        let mut virt = linker::ASPACE_WINDOW_START;
        let layout = Layout::new::<AddressSpace>();

        // Do a bunch of checks to make sure everything is configured properly.
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
        assert!(
            layout.align() == paging::BASE_PAGE,
            "AddressSpace has wrong alignment!"
        );
        assert!(
            (layout.size() * num as usize) <= linker::ASPACE_WINDOW_SIZE,
            "ASPACE_WINDOW_SIZE is too small!"
        );
        assert!(
            is_aligned::<{ paging::BASE_PAGE }>(linker::_cpulocal_load_addr()),
            ".cpulocal_load_addr is not properly aligned!"
        );
        assert!(
            is_aligned::<{ paging::BASE_PAGE }>(linker::_ecpulocal_load_addr()),
            ".ecpulocal_load_addr is not properly aligned!"
        );

        // The physical kernel region. We count on `base` being 0 further down when
        // coalescing the memory regions.
        let kernel_region = &Region {
            base: 0,
            length: self.virt_to_phys(linker::_end()).unwrap(),
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

        let cpu_local_len =
            (linker::_ecpulocal_load_addr() - linker::_ecpulocal_load_addr()) as usize;
        let total_phys_len = layout.size() + cpu_local_len;

        // The number of frames needed to map an address space. Note that we skip over the
        // CPU local block here, since they are not mapped into the global kernel map. Instead,
        // each address space maps the cpu local block itself.
        let num_frames = layout.size() / paging::BASE_PAGE;

        let num_lvl_3 = align_up::<{ paging::PD_COVERAGE }>(total_phys_len as u64 * num as u64)
            as usize
            / paging::PD_COVERAGE;

        (0..num_lvl_3).for_each(|i| {
            let virt = virt + (i * paging::PD_COVERAGE) as u64;
            let address = self
                .virt_to_phys(self.aspace_lvl_2[pdpt_index(virt)].as_ptr() as _)
                .unwrap();
            self.lvl_3[pdpt_index(virt)] = PDPTE::new(address, PDPTEFlags::P);
        });

        for region in regions.iter() {
            if num == 0 {
                break;
            }

            if (region.length as usize) < total_phys_len {
                continue;
            }

            let num_spaces = cmp::min(num as usize, region.length as usize / total_phys_len);
            let frame_base = align_up::<{ paging::BASE_PAGE }>(region.base);
            for x in 0..num_spaces {
                let virt_offset = (x * layout.size()) as u64;
                let frame_offset = (x * total_phys_len) as u64;

                for y in 0..num_frames {
                    let delta = (y * paging::BASE_PAGE) as u64;
                    let virt = virt + virt_offset + delta;
                    let frame = frame_base + frame_offset + delta;

                    if self.aspace_lvl_2[pdpt_index(virt)][pd_index(virt)] == PDE::NULL {
                        let address = self
                            .virt_to_phys(self.aspace_lvl_1[pd_index(virt)].as_ptr() as _)
                            .unwrap();
                        self.aspace_lvl_2[pdpt_index(virt)][pd_index(virt)] =
                            PDE::new(address, PDEFlags::P | PDEFlags::RW);
                    }

                    self.aspace_lvl_1[pd_index(virt)][pt_index(virt)] =
                        PTE::new(frame, PTEFlags::P | PTEFlags::RW);
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
    fn map(&mut self, mut virt: u64, mut phys: u64, len: usize) {
        #[inline]
        fn is_range_valid(range: &Range<u64>) -> bool {
            range.start >= KernelMap::KERN_VIRT_ADDR_RANGE.start
                && range.start <= KernelMap::KERN_VIRT_ADDR_RANGE.end
                && range.end >= KernelMap::KERN_VIRT_ADDR_RANGE.start
                && range.end <= KernelMap::KERN_VIRT_ADDR_RANGE.end
        }

        // Make sure we can actually map the request. This also ensures that calls to `self.virt_to_phys`
        // will actually resolve.
        assert!(
            is_range_valid(&(virt..virt + len as u64)),
            "KernelMap: requested map has an invalid range.",
        );

        // Fixup the addresses so they are aligned on the correct page boundary.
        virt = align_down::<{ paging::PT_COVERAGE }>(virt);
        phys = align_down::<{ paging::PT_COVERAGE }>(phys);

        // Point the level-3 entry to the level-2 table. Panics if the entry is pointing to the wrong table (which would be a bug).
        let address = self.virt_to_phys(self.kern_lvl_2.as_ptr() as _).unwrap();
        if self.lvl_3[pdpt_index(virt)] == PDPTE::NULL {
            self.lvl_3[pdpt_index(virt)] = PDPTE::new(address, PDPTEFlags::P | PDPTEFlags::RW);
        } else if self.lvl_3[pdpt_index(virt)].address() != address {
            panic!("KernelMap: level-3 entry pointing to an unexpected level-2 table ({:#018x})! This is a bug.", self.lvl_3[pdpt_index(virt)].address());
        }

        let frames = align_up::<{ paging::PT_COVERAGE }>(len as u64) / paging::PT_COVERAGE as u64;
        for _ in 0..frames {
            // Point the level-2 entry to the correct frame. Panics in case the entry is already mapped to a different frame.
            if self.kern_lvl_2[pd_index(virt)] == PDE::NULL {
                self.kern_lvl_2[pd_index(virt)] =
                    PDE::new(phys, PDEFlags::P | PDEFlags::RW | PDEFlags::PS);
            } else if self.kern_lvl_2[pd_index(virt)].address() != phys {
                let entry = self.kern_lvl_2[pd_index(virt)];
                panic!("KernelMap: requested map ({:#018x} -> {:#018x}) is already mapped to a different address ({:#018x})", virt, phys, entry.address());
            }

            phys += paging::PT_COVERAGE as u64;
            virt += paging::PT_COVERAGE as u64;
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
pub struct AddressSpace<'a> {
    /// The top of the address space for the owning CPU.
    top: PML4,

    /// Page tables for local per-CPU storage. We know this will fit in a single
    /// PDPT for sure (since the size of the *entire* kernel is at most 512M).
    local_lvl_3: PDPT,

    /// As with the level-3 table, a single level-2 will suffice.
    local_lvl_2: PD,

    /// The number of level-1 tables depends on the configured physical offset
    /// in the linker. By default this gives us 8 tables.
    local_lvl_1: [PT; linker::KERNEL_PHYS_START as usize / paging::PT_COVERAGE],

    /// Refrence to the global read-only kernel map.
    kernel_map: MaybeUninit<&'a KernelMap>,

    /// The apic ID of the CPu this address space belongs to.
    apic_id: u32,

    /// Pointer to self.
    this: MaybeUninit<&'a AddressSpace<'a>>,
}

impl<'a> AddressSpace<'a> {
    pub fn init(&mut self, kernel_map: &'a KernelMap, apic_id: u16) {
        // Make kernel map available.
        self.top[pml4_index(linker::KERNEL_START)] =
            PML4E::new(kernel_map.kernel_lvl_3(), PML4EFlags::P);

        // Begin the setup of our local per-CPU data.
        self.top[pml4_index(linker::ASPACE_LOCAL_START)] = PML4E::new(
            kernel_map
                .virt_to_phys(self.local_lvl_3.as_ptr() as u64)
                .unwrap(),
            PML4EFlags::P,
        );

        self.local_lvl_3[pdpt_index(linker::ASPACE_LOCAL_START)] = PDPTE::new(
            kernel_map
                .virt_to_phys(self.local_lvl_2.as_ptr() as _)
                .unwrap(),
            PDPTEFlags::P,
        );

        let cpu_local_len = align_up::<{ paging::BASE_PAGE }>(unsafe {
            linker::_ecpulocal_load_addr() - linker::_cpulocal_load_addr()
        });

        let num_pt = cmp::max(1, cpu_local_len / paging::PT_COVERAGE as u64);
        (0..num_pt).for_each(|i| {
            let virt = linker::ASPACE_LOCAL_START + (i * paging::PT_COVERAGE as u64);
            let address = kernel_map
                .virt_to_phys(self.local_lvl_1[pd_index(virt)].as_ptr() as _)
                .unwrap();
            self.local_lvl_2[pd_index(virt)] = PDE::new(address, PDEFlags::P);
        });

        let frame_start = kernel_map.virt_to_phys(self.top.as_ptr() as _).unwrap()
            + Layout::new::<AddressSpace>().size() as u64;

        let num_frames = cpu_local_len / paging::BASE_PAGE as u64;
        (0..num_frames).for_each(|i| {
            let offset = i * paging::BASE_PAGE as u64;
            let virt = linker::ASPACE_LOCAL_START + offset;
            let frame = frame_start + offset;
            self.local_lvl_1[pd_index(virt)][pt_index(virt)] =
                PTE::new(frame, PTEFlags::P | PTEFlags::RW);
        });

        self.apic_id = apic_id as u32;
        self.kernel_map.write(kernel_map);
    }

    pub fn init_cpu_local(&self) {
        // TODO: clean this up
        // The gist of it is that the address space needs to be flushed in order for its local
        // memory to be available (since per-cpu memory is not globally mapped, but instead only
        // available to the owning CPU's address space). This creates an issue because this function
        // should be called immediately after flushing, so, init, flush, init_cpu_local, which is
        // rather ugly. So, for now, it just serves to show that the per-cpu storage *does* work,
        // but the setup phase should be improved...

        let cpu_local_len = align_up::<{ paging::BASE_PAGE }>(unsafe {
            linker::_ecpulocal_load_addr() - linker::_cpulocal_load_addr()
        });

        let from = unsafe {
            slice::from_raw_parts(
                linker::_cpulocal_load_addr() as *const u8,
                cpu_local_len as usize,
            )
        };

        let to = unsafe {
            slice::from_raw_parts_mut(
                linker::ASPACE_LOCAL_START as *mut u8,
                cpu_local_len as usize,
            )
        };
        to.copy_from_slice(&from);
    }

    /// Install the address space on the current CPU. This operation consumes self,
    /// at which point the only way to access the address space, is through [AddressSpace::current()].
    pub unsafe fn install(self) {
        // We flush ourselves, so we have immediate access to the virtual memory.
    }

    #[inline]
    pub fn apic_id(&self) -> u32 {
        self.apic_id
    }

    /// Flush the address space.
    ///
    /// # Safety
    /// The address space and kernel map need to be initialised!
    pub unsafe fn flush(&self) {
        cr3_write(
            self.kernel_map
                .assume_init()
                .virt_to_phys(self.top.as_ptr() as _)
                .unwrap(),
        );
    }
}

impl<'a> Drop for AddressSpace<'a> {
    fn drop(&mut self) {
        panic!("AddressSpace not supposed to be dropped!");
    }
}
