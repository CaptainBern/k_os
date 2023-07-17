//! Kernel memory management.

mod consts;
pub mod desc;
pub mod map;
pub mod memory;
pub mod paging;

use core::{cell::OnceCell, ops::Range, slice};

use heapless::Vec;
use spin::{Mutex, Once};
use x86::controlregs::cr3_write;

use crate::linker;

use self::{
    consts::{NUM_PERCPU_PDS, NUM_PERCPU_PTS, NUM_PHYS_PDPTS},
    desc::MemoryDescriptor,
    map::{Flags, Mapper, PdMapper, PdptMapper},
    memory::Memory,
    paging::{
        num_tables, pd_index, pdpt_index, pml4_index, pt_index, PDEFlags, PDPTEFlags, PML4EFlags,
        PTEFlags, PD, PDPT, PML4, PT,
    },
};

/// Top level 4 table. All other tables live in here.
static mut TOP: PML4 = PML4::zero();

/// Kernel space level 3 table. The top half of the virtual address space is reserved
/// for kernel use. Kernel text/data, per-cpu variables, and kernel devices are mapped
/// in the top 512G, using this table.
static mut KERNEL_PDPT: PDPT = PDPT::zero();

/// Level 2 kernel table. Used to map a 512M window containing kernel text and data.
/// Where possible, kernel sections are mapped using 2M pages, else we fall back to 4K
/// pages for more finegrained control using [KERN_PT].
static mut KERN_PD: PD = PD::zero();

/// Level 1 kernel table. Makes finegrained mappings possible.
static mut KERN_PT: [PT; 256] = [PT::zero(); 256];

/// Level 2 table for kernel devices.
static mut KDEV_PD: PD = PD::zero();

/// Level 1 table for kernel devices.
static mut KDEV_PT: PT = PT::zero();

/// Level 2 table for per-cpu data.
static mut PERCPU_PDS: [PD; NUM_PERCPU_PDS] = [PD::zero(); NUM_PERCPU_PDS];

/// Level 1 table for per-cpu data. All per-cpu data is mapped using 4K pages, even if
/// we could map using 2M pages. The reason is that per-cpu data is allocated at runtime,
/// and just mapping a 4K page to a 4K frame makes it a little bit easier.
static mut PERCPU_PTS: [PT; NUM_PERCPU_PTS] = [PT::zero(); NUM_PERCPU_PTS];

/// A huge table used to map all physical memory to a predefined offset. To map all
/// physical memory, we use 1G pages. This has the advantage that we don't actually
/// occupy more memory between 0 and 512G of physical memory.
static mut PHYS_PDPTS: [PDPT; NUM_PHYS_PDPTS] = [PDPT::zero(); NUM_PHYS_PDPTS];

/// Keep track of free frames.
static MEMORY: Mutex<OnceCell<Memory<{ crate::MAX_MEM_REGIONS }>>> = Mutex::new(OnceCell::new());

/// Container to keep track of per-cpu data.
#[derive(Debug)]
pub struct PerCpuInfo {
    /// The virtual start address of the per-cpu storage.
    pub storage: u64,

    /// The stack.
    pub stack: &'static mut [u8; linker::STACK_SIZE],
}

/// Map the kernel window.
///
/// This function maps the kernel with the correct permisisons in the page tables. For it
/// to work properly, the following must hold:
/// - CR0.WP = 1
/// - CR0.NW = 0
/// - CR0.CD = 0
/// - CR4.PGE = 1
/// - EFER.NXE = 1
/// (in addition to the bits required for longmode of course).
unsafe fn map_kernel_window<const LINK_OFFSET: usize>(mapper: &mut PdptMapper<LINK_OFFSET>) {
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
                mapper.map(
                    pd_index(virt),
                    virt - LINK_OFFSET as u64,
                    Flags::Enable(pd_flags),
                );
                virt += paging::MEGA_PAGE as u64;
            } else {
                mapper
                    .pt(
                        pd_index(virt),
                        &mut KERN_PT[pd_index(virt) - pd_index(linker::VIRT_OFFSET)],
                        Flags::Enable(PDEFlags::P | PDEFlags::RW),
                    )
                    .map(
                        pt_index(virt),
                        virt - LINK_OFFSET as u64,
                        Flags::Enable(pt_flags),
                    );
                virt += paging::BASE_PAGE as u64;
            }
        }
    }

    let text = linker::_text()..linker::_etext();
    let rodata = linker::_rodata()..linker::_erodata();
    let data = linker::_data()..linker::_ebss();

    let mut pd = mapper.pd(
        pdpt_index(LINK_OFFSET as u64),
        &mut KERN_PD,
        Flags::Enable(PDPTEFlags::P | PDPTEFlags::RW),
    );

    map_range(&mut pd, text, false, false);
    map_range(&mut pd, rodata, false, true);
    map_range(&mut pd, data, true, true);
}

/// Map [linker::MAX_PHYS_MEMORY] using 1G pages at [linker::PHYS_OFFSET].
///
/// The mapped window has the execute-disable bit set, and is read-only.
unsafe fn map_phys_window<const LINK_OFFSET: usize>(mapper: &mut Mapper<LINK_OFFSET>) {
    let num_frames = linker::MAX_PHYS_MEMORY / paging::GIGA_PAGE;
    let num_pdpts = num_frames / 512;

    for x in 0..num_pdpts {
        let delta = (x * paging::GIGA_PAGE) as u64;
        let phys_pdpt = &mut PHYS_PDPTS[x];
        let mut pdpt = mapper.pdpt(
            pml4_index(linker::PHYS_OFFSET + delta),
            phys_pdpt,
            Flags::Enable(PML4EFlags::P),
        );
        for y in 0..512 {
            pdpt.map(
                y,
                (y * paging::GIGA_PAGE) as u64 + delta,
                Flags::Enable(PDPTEFlags::P | PDPTEFlags::PS | PDPTEFlags::XD),
            );
        }
    }
}

/// Allocate and map the per-CPU structures.
unsafe fn allocate_per_cpus<const LINK_OFFSET: usize>(
    mapper: &mut PdptMapper<LINK_OFFSET>,
    memory: &mut Memory<{ crate::MAX_MEM_REGIONS }>,
    num: usize,
) -> Vec<PerCpuInfo, { linker::MAX_CPUS }> {
    fn map<const LINK_OFFSET: usize>(
        mapper: &mut PdptMapper<LINK_OFFSET>,
        pds: &mut [PD; NUM_PERCPU_PDS],
        pts: &mut [PT; NUM_PERCPU_PTS],
        virt: u64,
        frame: u64,
    ) {
        mapper
            .pd(
                pdpt_index(virt),
                &mut pds[pdpt_index(virt) - pdpt_index(linker::PERCPU_OFFSET)],
                Flags::Enable(PDPTEFlags::P | PDPTEFlags::RW),
            )
            .pt(
                pd_index(virt),
                &mut pts[pd_index(virt) - pd_index(linker::PERCPU_OFFSET)],
                Flags::Enable(PDEFlags::P | PDEFlags::RW),
            )
            .map(
                pt_index(virt),
                frame,
                Flags::Enable(PTEFlags::P | PTEFlags::RW | PTEFlags::XD),
            );
    }

    // Make sure we aren't allocating more than we can handle.
    assert!(num <= linker::MAX_CPUS);
    assert!(num >= 1);

    let mut info: Vec<PerCpuInfo, { linker::MAX_CPUS }> = Vec::new();

    let block_size = (linker::_epercpu_load() - linker::_percpu_load()) as usize;
    let frames_per_block = num_tables::<{ paging::BASE_PAGE }>(block_size);
    let frames_per_stack = num_tables::<{ paging::BASE_PAGE }>(linker::STACK_SIZE);

    let mut virt = linker::PERCPU_OFFSET;
    for _ in 0..num {
        // Map the contiguous per-cpu storage block first
        let storage = virt;
        for _ in 0..frames_per_block {
            let frame = memory.next().expect("Failed to retrieve frame");
            map(mapper, &mut PERCPU_PDS, &mut PERCPU_PTS, virt, frame);
            virt += paging::BASE_PAGE as u64;
        }

        // Stack guard hole.
        virt += linker::STACK_GUARD_SIZE as u64;

        // Map stack. The System V ABI dictates that the stack should be aligned on a 16 byte boundary.
        // Since ours sits on a page bounary, this is always the case.
        let stack = virt;
        for _ in 0..frames_per_stack {
            let frame = memory.next().expect("Failed to retrieve frame");
            map(mapper, &mut PERCPU_PDS, &mut PERCPU_PTS, virt, frame);
            virt += paging::BASE_PAGE as u64;
        }

        // Stack guard hole.
        virt += linker::STACK_GUARD_SIZE as u64;

        info.push(PerCpuInfo {
            storage,
            stack: (stack as *mut [u8; linker::STACK_SIZE]).as_mut().unwrap(),
        })
        .unwrap();
    }

    // Next copy over the per-cpu data for each cpu, we use the phys window here since the per-cpu region
    // isn't actually mapped.
    let data = unsafe {
        slice::from_raw_parts(
            ((linker::_percpu_load() - linker::VIRT_OFFSET) + linker::PHYS_OFFSET) as *const u8,
            block_size,
        )
    };

    for percpu in &mut info {
        // copy over the per-cpu data.
        let block = unsafe { slice::from_raw_parts_mut(percpu.storage as *mut u8, block_size) };
        block.copy_from_slice(&data);

        // zero the stack.
        percpu.stack.fill(0u8);
    }

    info
}

/// Initialise the kernel page tables.
///
/// This function maps the kernel and physical memory windows. It does not
/// activate the kernel pages! This function should only be called once.
pub fn init() {
    static INIT: Once<()> = Once::new();
    INIT.call_once(|| unsafe {
        let mut mapper: Mapper<{ linker::VIRT_OFFSET as usize }> = Mapper::new(&mut TOP);

        let mut pdpt = mapper.pdpt(
            pml4_index(linker::KERNEL_START),
            &mut KERNEL_PDPT,
            Flags::Set(PML4EFlags::P | PML4EFlags::RW),
        );

        // Map the kernel.
        map_kernel_window(&mut pdpt);

        // Map the physical memory window.
        map_phys_window(&mut mapper);
    });
}

/// Switches the CPU to the kernel page tables.
///
/// # Safety
/// This function should only be called after a call to [init_once].
#[inline]
pub unsafe fn switch_to_kernel() {
    let ptr = TOP.table.as_ptr();
    cr3_write(ptr as u64 - linker::VIRT_OFFSET);
}

/// Return the physical address of the kernel top table.
pub fn kernel_top() -> u64 {
    unsafe { TOP.table.as_ptr() as u64 - linker::VIRT_OFFSET }
}

/// Setup the bootcode for the APs at the given vector.
///
/// This function depends on the physical window being available, so it should
/// only be called after the kernel pages are active.
pub fn setup_ap_bootcode(vector: u8) {
    unsafe fn map_bootcode(phys: u64, virt: u64, len: usize) {
        // This memory range should be free for us to use.
        const LOW_MEM_START: u64 = 0x500;
        const LOW_MEM_END: u64 = 0x7ffff;
        const LOW_MEM_LEN: usize = (LOW_MEM_END - LOW_MEM_START) as usize;

        assert!(len <= LOW_MEM_LEN);
        assert!(phys >= LOW_MEM_START && (phys + len as u64) <= LOW_MEM_END);

        let mut mapper: Mapper<{ linker::VIRT_OFFSET as usize }> = Mapper::new(&mut TOP);
        let mut pd = mapper
            .pdpt(
                pml4_index(virt),
                &mut KERNEL_PDPT,
                Flags::Enable(PML4EFlags::P | PML4EFlags::RW),
            )
            .pd(
                pdpt_index(virt),
                &mut KERN_PD,
                Flags::Enable(PDPTEFlags::P | PDPTEFlags::RW),
            );

        let num_frames = num_tables::<{ paging::BASE_PAGE }>(len);
        for i in 0..num_frames {
            let delta = (i * paging::BASE_PAGE) as u64;
            pd.pt(
                pd_index(virt + delta),
                &mut KERN_PT[pd_index(virt + delta) - pd_index(linker::VIRT_OFFSET)],
                Flags::Enable(PDEFlags::P | PDEFlags::RW),
            )
            .map(
                pt_index(virt + delta),
                phys + delta,
                Flags::Set(PTEFlags::P | PTEFlags::RW),
            );
        }
    }

    // The physical address at which we'll map the bootcode.
    let phys = (vector as u64) << 12;

    let len = (linker::_eboot16() - linker::_boot16()) as usize;
    let virt = linker::VIRT_OFFSET + linker::_boot16();

    // Make sure the code fits in a single page, this is because the code
    // itself depends on data located at `((vector << 12) + 0x1000)`
    assert!(len <= paging::BASE_PAGE);

    unsafe {
        // Make the lower-memory bootcode region available.
        // TODO: map extra pages for the data we pass to it.
        map_bootcode(phys, virt, 0x3000);

        // zero
        slice::from_raw_parts_mut((linker::VIRT_OFFSET + linker::_boot16()) as *mut u8, 0x3000)
            .fill(0);

        let src =
            slice::from_raw_parts((linker::PHYS_OFFSET + linker::_boot16()) as *const u8, len);
        let dst = slice::from_raw_parts_mut(virt as *mut u8, len);

        dst.copy_from_slice(src);
    }
}

/// Map the given local APIC MMIO address to [linker::KDEV_OFFSET].
///
/// The kernel tables do not have to be active for this operation to succeed. Because
/// the MMIO region is relative to each CPU, this function should only be called once.
pub fn map_apic(local_apic_address: u64, io_apics: &Vec<u32, { linker::MAX_IOAPICS }>) {
    assert!(linker::LOCAL_APIC_ADDRESS >= linker::KDEV_OFFSET);
    assert!(
        linker::LOCAL_APIC_ADDRESS + paging::BASE_PAGE as u64
            <= linker::KDEV_OFFSET + (paging::GIGA_PAGE as u64 - 1)
    );

    // TODO: Clean up
    unsafe {
        let mut mapper: Mapper<{ linker::VIRT_OFFSET as usize }> = Mapper::new(&mut TOP);
        let mut pt = mapper
            .pdpt(
                pml4_index(linker::LOCAL_APIC_ADDRESS),
                &mut KERNEL_PDPT,
                Flags::Enable(PML4EFlags::P | PML4EFlags::RW),
            )
            .pd(
                pdpt_index(linker::LOCAL_APIC_ADDRESS),
                &mut KDEV_PD,
                Flags::Enable(PDPTEFlags::P | PDPTEFlags::RW),
            )
            .pt(
                pd_index(linker::LOCAL_APIC_ADDRESS),
                &mut KDEV_PT,
                Flags::Enable(PDEFlags::P | PDEFlags::RW),
            );

        pt.map(
            pt_index(linker::LOCAL_APIC_ADDRESS),
            local_apic_address,
            Flags::Enable(PTEFlags::P | PTEFlags::PCD | PTEFlags::PWT | PTEFlags::RW),
        );

        let mut virt = linker::IO_APIC_OFFSET;
        for io_apic in io_apics {
            pt.map(
                pt_index(virt),
                *io_apic as u64,
                Flags::Enable(PTEFlags::P | PTEFlags::PCD | PTEFlags::PWT | PTEFlags::RW),
            );
            virt += 0x1000;
        }
    }
}

/// Initialise the available physical memory.
///
/// Any regions below 1M are filtered out, since we use that region to bootstrap
/// APs.
pub fn init_memory(mem: &Vec<MemoryDescriptor, { crate::MAX_MEM_REGIONS }>) {
    MEMORY
        .lock()
        .set(Memory::new(mem))
        .expect("Memory already set!");
}

/// Allocate per-cpu data.
///
/// # Safety
/// This function may only be called after the kernel tables are active!
pub fn allocate_percpus(num: usize) -> Vec<PerCpuInfo, { linker::MAX_CPUS }> {
    let mut memory = MEMORY.lock();

    unsafe {
        let mut mapper: Mapper<{ linker::VIRT_OFFSET as usize }> = Mapper::new(&mut TOP);
        let mut pdpt = mapper.pdpt(
            pdpt_index(linker::PERCPU_OFFSET),
            &mut KERNEL_PDPT,
            Flags::Enable(PML4EFlags::P | PML4EFlags::RW),
        );

        allocate_per_cpus(
            &mut pdpt,
            memory.get_mut().expect("Memory not initialised"),
            num,
        )
    }
}
