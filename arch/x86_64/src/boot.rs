use core::{
    panic,
    sync::atomic::{compiler_fence, AtomicBool, AtomicU32, Ordering},
};

use acpi::{
    madt::{ApicStructureKind, LocalApicFlags},
    AcpiTables, TableKind,
};
use heapless::Vec;
use multiboot2::{MemoryAreaType, MemoryMapTag};
use spin::Once;

use crate::{
    idt, include_asm, linker,
    mm::{
        self,
        desc::{MemoryDescriptor, Region},
    },
    pic, println, smp,
};

mod early;
pub mod serial_console;

include_asm! {
    "src/boot/header.S",
    "src/boot/start16.S",
    "src/boot/start.S"
}

/// The vector at which the AP bootcode is mapped.
const AP_BOOTCODE: u8 = (0x8000 >> 12) as u8;

static CPU_INFO: Once<Vec<CpuInfo, { linker::MAX_CPUS }>> = Once::new();

#[derive(Debug)]
pub struct CpuInfo {
    pub apic_id: u32,
    pub stack: u64,
    pub percpu_offset: u64,
}

#[derive(Debug)]
pub struct ApicInfo {
    pub local_apic_address: u64,
    pub apic_ids: Vec<u32, { linker::MAX_CPUS }>,
    pub io_apics: Vec<u32, { linker::MAX_IOAPICS }>,
}

impl ApicInfo {
    pub fn bsp_id(&self) -> u32 {
        assert!(self.apic_ids.len() > 0);
        self.apic_ids.as_slice()[0]
    }

    pub fn num_cpus(&self) -> usize {
        self.apic_ids.len()
    }
}

/// Parse the memory map provided by multiboot2 into our own descriptors.
fn parse_memory_map(mmap: &MemoryMapTag) -> Vec<MemoryDescriptor, { crate::MAX_MEM_REGIONS }> {
    Vec::from_iter(mmap.all_memory_areas().map(|area| MemoryDescriptor {
        kind: match area.typ() {
            MemoryAreaType::Available => mm::desc::MemoryKind::Usable,
            MemoryAreaType::Reserved => mm::desc::MemoryKind::Reserved,
            MemoryAreaType::AcpiAvailable => mm::desc::MemoryKind::AcpiReclaimable,
            MemoryAreaType::ReservedHibernate => mm::desc::MemoryKind::AcpiNvs,
            MemoryAreaType::Defective => mm::desc::MemoryKind::Defective,
        },
        region: Region {
            base: area.start_address(),
            length: area.size() as usize,
        },
    }))
}

/// Parse the ACPI tables (at least the ones we use).
fn parse_acpi(acpi_tables: &AcpiTables) -> ApicInfo {
    let madt = acpi_tables
        .iter()
        .find_map(|table| {
            if let TableKind::Madt(madt) = table {
                Some(madt)
            } else {
                None
            }
        })
        .expect("MADT not present!");

    let mut cpu_ids: Vec<u32, { linker::MAX_CPUS }> = Vec::new();
    let mut add_cpu = |flags: LocalApicFlags, apic_id| {
        if !cpu_ids.is_full() && flags.bits() == LocalApicFlags::ENABLED.bits() {
            cpu_ids.push(apic_id).unwrap();
        }
    };

    let mut local_apic_address = madt.local_apic_address as u64;

    let mut io_apics = Vec::new();

    // ACPI spec dictates the BSP is the first entry in the table. Additionally,
    // the lists contains the first logical processor of each of the possible
    // individual multithreaded processors.
    for ics in madt.iter() {
        match ics {
            // If an override address is present, we MUST use that instead.
            ApicStructureKind::LocalApicAddressOverride(address) => {
                local_apic_address = address.local_apic_address;
            }
            ApicStructureKind::ProcessorLocalApic(apic) => add_cpu(apic.flags, apic.apic_id as u32),
            ApicStructureKind::ProcessorLocalX2Apic(xapic) => add_cpu(xapic.flags, xapic.x2apic_id),
            ApicStructureKind::IoApic(ioapic) => {
                // TODO: Check if we're exceeding max, and just continue with however many we can support.
                io_apics
                    .push(ioapic.io_apic_address)
                    .expect("Failed to push ioapic");
            }
            _ => {}
        }
    }

    ApicInfo {
        local_apic_address,
        apic_ids: cpu_ids,
        io_apics,
    }
}

/// Used to keep track of booting APs.
static AP_BUSY: AtomicBool = AtomicBool::new(false);

/// Used to keep track of the number of active APs in the system.
static AP_COUNT: AtomicU32 = AtomicU32::new(0);

/// Boot an AP.
#[no_mangle]
extern "C" fn boot_ap(stack: u64, percpu_offset: u64) {
    let ap_id = AP_COUNT.fetch_add(1, Ordering::Relaxed);

    // We got our ID, so the next AP can be booted up.
    AP_BUSY.store(false, Ordering::SeqCst);

    println!(
        "Hello from rust (storage: {:#018x})! I am core #{}",
        percpu_offset, ap_id
    );

    // Init the CPU.
    crate::init(stack, percpu_offset);

    // Ready to start doing work.
    crate::start();
}

/// Boot the BSP.
extern "C" fn boot_bsp(stack: u64, percpu_offset: u64) -> ! {
    crate::init(stack, percpu_offset);

    let aps = CPU_INFO
        .get()
        .expect("CPU info not present!")
        .iter()
        .skip(1);

    let bootstrap = unsafe {
        smp::Bootstrap::new(
            (linker::VIRT_OFFSET + linker::_boot16()) as *mut [u8; 0x2000],
            0x8000,
            mm::kernel_top(),
        )
    };

    // It's up to us to bring up the APs.
    for ap in aps {
        AP_BUSY.store(true, Ordering::SeqCst);

        unsafe { bootstrap.try_start_ap(ap.apic_id, ap.stack, ap.percpu_offset) }

        // Wait until the AP is done setting up.
        while AP_BUSY.load(Ordering::SeqCst) {
            core::hint::spin_loop();
        }
    }

    crate::start();
}

/// Switch stack and jump into [`boot_bsp`].
///
/// # Safety
/// This function updates the current stack pointer.
unsafe fn switch_stack_and_boot(new_stack: u64, percpu_offset: u64) -> ! {
    core::arch::asm!(
        "
        movq    {stack}, %rdi
        movq    %rdi, %rsp
        movq    %rdi, %rbp
        movq    {percpu_offset}, %rsi
        leaq    {boot}(%rip), %rax
        pushq   %rax
        retq
        ",
        stack = in(reg) new_stack,
        percpu_offset = in(reg) percpu_offset,
        boot = sym boot_bsp,
        options(att_syntax, noreturn)
    );
}

/// Prepare for booting the BSP.
///
/// We come here straight from assembly with the following:
/// - 4G of physical memory identity mapped using 2M pages.
/// - First 2G of physical memory mapped to -2G, so higher-half memory access works.
/// - A simple GDT without TSS
/// - No IDT, interrupts disabled.
/// - A boot stack without guard pages.
///
/// This function continues the setup process by properly remapping the kernel, setting
/// up a serial console, installing an IDT etc.
#[no_mangle]
#[link_section = ".text"]
unsafe extern "C" fn pre_boot(multiboot_info_ptr: u64) {
    // Setup some form of output ASAP.
    serial_console::init();

    // Prepare for switching to proper page tables, and allocating per-cpu
    // structures.
    mm::init();

    // We're using x(2)APIC, so disable the PIC before we enable APIC.
    pic::remap(0x20, 0x28);
    pic::disable();

    // Now we can handle exceptions.
    idt::init();

    // Switch to the proper kernel pages.
    mm::switch_to_kernel();

    // From this point on we depend on the proper kernel maps to be available.
    compiler_fence(Ordering::SeqCst);

    // Physical memory should be availabl at [linker::PHYS_OFFSET] now, so we
    // can safely
    // load the boot info.
    let boot_info = unsafe {
        multiboot2::load_with_offset(multiboot_info_ptr as usize, linker::PHYS_OFFSET as usize)
            .expect("Failed to read multiboot2 info!")
    };

    // Now the ACPI tables are available as well. We access them through the
    // physical memory window.
    let acpi_address = if let Some(xsdt) = boot_info.rsdp_v2_tag() {
        xsdt.xsdt_address()
    } else if let Some(rsdp) = boot_info.rsdp_v1_tag() {
        rsdp.rsdt_address()
    } else {
        panic!("No ACPI info!")
    };

    let acpi_tables = unsafe {
        AcpiTables::from_address(acpi_address, linker::PHYS_OFFSET as usize)
            .expect("Failed to read the ACPI tables!")
    };

    // Find the APIC info. We need it to find out how many cores are available.
    let apic_info = parse_acpi(&acpi_tables);

    // There must be at least 1 CPU. If there isn't, something is wrong.
    assert!(apic_info.num_cpus() >= 1);

    if apic_info.num_cpus() > 1 {
        // Setup the AP bootcode in case there's more than one processor.
        mm::setup_ap_bootcode(AP_BOOTCODE);
    }

    // Map local APIC. If X2APIC is available, we use that instead, but the APIC mmio
    // is mapped either way.
    mm::map_apic(apic_info.local_apic_address, &apic_info.io_apics);

    // Translate the memory descriptors provided by the bootloader into a
    // format we understand.
    let mem_descriptors = parse_memory_map(
        boot_info
            .memory_map_tag()
            .expect("Memory map not provided by bootloader!"),
    );

    // Setup available memory for per-CPU data.
    mm::init_memory(&mem_descriptors);

    // Allocate memory for every core.
    let per_cpus = mm::allocate_percpus(apic_info.num_cpus());

    CPU_INFO.call_once(|| {
        apic_info
            .apic_ids
            .into_iter()
            .zip(per_cpus.into_iter())
            .map(|(apic_id, percpu)| CpuInfo {
                apic_id,
                stack: percpu.stack.as_ptr() as u64 + percpu.stack.len() as u64,
                percpu_offset: percpu.storage,
            })
            .collect::<Vec<CpuInfo, { linker::MAX_CPUS }>>()
    });

    // Make ACPI tables available to everyone.
    crate::ACPI_TABLES.call_once(|| acpi_tables);

    unsafe {
        let bsp = CPU_INFO.get_unchecked().first().unwrap();
        switch_stack_and_boot(bsp.stack, bsp.percpu_offset)
    }
}
