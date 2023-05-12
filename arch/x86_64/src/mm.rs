//! Kernel memory management.

pub mod desc;
pub mod map;
pub mod memory;
pub mod paging;

use core::{alloc::Layout, mem, ops::Range};

use heapless::Vec;
use spin::Once;
use x86::{controlregs::cr3_write, current::segmentation::wrgsbase, fence::mfence};

use crate::{linker, mm::map::PdMapper, percpu::PerCpu, BootInfo};

use self::{
    map::{Mapper, PdptMapper},
    memory::Memory,
    paging::{
        num_tables, pd_index, pdpt_index, pml4_index, pt_index, PDEFlags, PDPTEFlags, PML4EFlags,
        PTEFlags, PD, PDPT, PML4, PT,
    },
};

const PERCPU_WINDOW_SIZE: usize = mem::size_of::<PerCpu>() * linker::MAX_CPUS;

const NUM_PERCPU_PDS: usize = paging::num_tables::<{ paging::PD_COVERAGE }>(PERCPU_WINDOW_SIZE);
const NUM_PERCPU_PTS: usize = paging::num_tables::<{ paging::PT_COVERAGE }>(PERCPU_WINDOW_SIZE);
const NUM_PHYS_PDPTS: usize =
    paging::num_tables::<{ paging::PDPT_COVERAGE }>(linker::MAX_PHYS_MEMORY);

/// A list of memory regions that are free to allocate memory in.
static mut MEMORY: Once<Memory<{ crate::MAX_MEM_REGIONS }>> = Once::new();

static mut TOP: PML4 = PML4::zero();
static mut KERNEL_PDPT: PDPT = PDPT::zero();
static mut KERNEL_PD: PD = PD::zero();
static mut KERNEL_PT: [PT; 512] = [PT::zero(); 512];
static mut PERCPU_PDS: [PD; NUM_PERCPU_PDS] = [PD::zero(); NUM_PERCPU_PDS];
static mut PERCPU_PTS: [PT; NUM_PERCPU_PTS] = [PT::zero(); NUM_PERCPU_PTS];
static mut PHYS_PDPTS: [PDPT; NUM_PHYS_PDPTS] = [PDPT::zero(); NUM_PHYS_PDPTS];

/// Map the kernel window.
unsafe fn map_kernel_window<const LINK_OFFSET: usize>(
    mapper: &mut Mapper<LINK_OFFSET>,
) -> PdptMapper<LINK_OFFSET> {
    unsafe fn map_range<const LINK_OFFSET: usize>(
        mapper: &mut PdMapper<LINK_OFFSET>,
        range: Range<u64>,
        rw: bool,
        xd: bool,
    ) {
        // Sanity-check the input. Should be fine either way...
        assert!(range.start >= LINK_OFFSET as u64 && range.start <= range.end);
        assert!(range.end <= (LINK_OFFSET + linker::KERNEL_SIZE) as u64);

        let pd_flags = {
            let mut flags = PDEFlags::P | PDEFlags::PS;
            flags.set(PDEFlags::RW, rw);
            flags.set(PDEFlags::XD, xd);
            flags
        };

        let pt_flags = {
            let mut flags = PTEFlags::P;
            flags.set(PTEFlags::RW, rw);
            flags.set(PTEFlags::XD, xd);
            flags
        };

        // The virtual address offset.
        let mut virt = range.start;
        loop {
            if virt >= range.end {
                break;
            }

            if (range.end - virt) as usize >= paging::MEGA_PAGE {
                mapper.map(pd_index(virt), virt - LINK_OFFSET as u64, pd_flags);
                virt += paging::MEGA_PAGE as u64;
            } else {
                // Map using 4K pages.
                mapper
                    .pt(pd_index(virt), &mut KERNEL_PT[pd_index(virt)], PDEFlags::P)
                    .map(pt_index(virt), virt - LINK_OFFSET as u64, pt_flags);
                virt += paging::BASE_PAGE as u64;
            }
        }
    }

    let text = linker::_text()..linker::_etext();
    let rodata = linker::_rodata()..linker::_erodata();
    let data = linker::_data()..linker::_ebss();

    let mut kernel_pdpt = mapper.pdpt(
        pml4_index(LINK_OFFSET as u64),
        &mut KERNEL_PDPT,
        PML4EFlags::P,
    );

    let mut kernel_pd = kernel_pdpt.pd(
        pdpt_index(LINK_OFFSET as u64),
        &mut KERNEL_PD,
        PDPTEFlags::P,
    );

    map_range(&mut kernel_pd, text, false, false);
    map_range(&mut kernel_pd, rodata, false, true);
    map_range(&mut kernel_pd, data, true, true);

    kernel_pdpt
}

/// Map [linker::MAX_PHYS_MEMORY] using 1G pages at [linker::PHYS_OFFSET].
unsafe fn map_phys_window<const LINK_OFFSET: usize>(mapper: &mut Mapper<LINK_OFFSET>) {
    let num_frames = linker::MAX_PHYS_MEMORY / paging::GIGA_PAGE;
    let num_pdpts = num_frames / 512;

    for x in 0..num_pdpts {
        let delta = (x * paging::GIGA_PAGE) as u64;
        let mut pdpt = mapper.pdpt(
            pml4_index(linker::PHYS_OFFSET + delta),
            &mut PHYS_PDPTS[x],
            PML4EFlags::P,
        );
        for y in 0..512 {
            pdpt.map(
                y,
                (y * paging::GIGA_PAGE) as u64 + delta,
                PDPTEFlags::P | PDPTEFlags::RW | PDPTEFlags::PS | PDPTEFlags::XD, // TODO: should this be RW?
            );
        }
    }
}

/// Allocate and map the per-CPU structures.
unsafe fn allocate_per_cpus<const LINK_OFFSET: usize>(
    mapper: &mut PdptMapper<LINK_OFFSET>,
    memory: &mut Memory<{ crate::MAX_MEM_REGIONS }>,
    apic_ids: &Vec<u32, { linker::MAX_CPUS }>,
    num: usize,
) {
    let layout = Layout::new::<PerCpu>().pad_to_align();
    let total_frames = num_tables::<{ paging::BASE_PAGE }>(layout.size()) * num;

    for i in 0..total_frames {
        let frame = memory.next().expect("Failed to allocate per-CPU!");
        let delta = (i * paging::BASE_PAGE) as u64;
        let virt = linker::PERCPU_OFFSET as u64 + delta;
        mapper
            .pd(
                pdpt_index(virt),
                &mut PERCPU_PDS[pdpt_index(virt) - pdpt_index(linker::PERCPU_OFFSET)],
                PDPTEFlags::P,
            )
            .pt(
                pt_index(virt),
                &mut PERCPU_PTS[pd_index(virt) - pd_index(linker::PERCPU_OFFSET)],
                PDEFlags::P,
            )
            .map(
                pt_index(virt),
                frame,
                PTEFlags::P | PTEFlags::RW | PTEFlags::XD,
            );
    }
}

/// Initialise the early memory management.
pub unsafe fn init_early(boot_info: &BootInfo) {
    MEMORY.call_once(|| Memory::new(&boot_info.mem_descriptors));

    let mut mapper: Mapper<{ linker::VIRT_OFFSET as usize }> = Mapper::new(&mut TOP);
    map_phys_window(&mut mapper);
    let mut kernel_pdpt = map_kernel_window(&mut mapper);
    {
        allocate_per_cpus(
            &mut kernel_pdpt,
            MEMORY.get_mut_unchecked(),
            &Vec::default(),
            16, // placeholder
        );
    }

    mfence();

    cr3_write(TOP.table.as_ptr() as u64 - linker::VIRT_OFFSET);

    // Just testing...
    wrgsbase(linker::PERCPU_OFFSET);
    let ptr = linker::PERCPU_OFFSET as *mut PerCpu;
    ptr.write(PerCpu::new(15));
}
