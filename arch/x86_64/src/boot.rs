use heapless::Vec;
use multiboot2::{load, MemoryAreaType};

use crate::{
    include_asm,
    mm::{
        self,
        desc::{MemoryDescriptor, Region},
    },
};

mod early;
pub mod serial_console;

include_asm!("src/boot/header.S", "src/boot/start.S");

/// Continue the boot process.
///
/// At this point the ifrst 4G of physical memory is identity mapped, as well
/// as the first 2G to the virtual top -2G. From here on we setup the memory
/// descriptors, and parse all other data so the boot process can continue.
#[no_mangle]
extern "C" fn boot(multiboot_info_ptr: usize) -> ! {
    serial_console::init();

    let multiboot_info = unsafe { load(multiboot_info_ptr).unwrap() };

    let memory_map = multiboot_info
        .memory_map_tag()
        .expect("Memory map not ptovided by bootloader!");

    let mem_descriptors: Vec<MemoryDescriptor, 32> =
        Vec::from_iter(memory_map.all_memory_areas().map(|area| MemoryDescriptor {
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
        }));

    let acpi_tables = if let Some(xsdt) = multiboot_info.rsdp_v2_tag() {
        unsafe { acpi::AcpiTables::from_xsdt(xsdt.xsdt_address()).unwrap() }
    } else if let Some(rsdp) = multiboot_info.rsdp_v1_tag() {
        unsafe { acpi::AcpiTables::from_rsdt(rsdp.rsdt_address()).unwrap() }
    } else {
        panic!("No RSDP found!");
    };

    crate::startup_system(crate::BootInfo {
        mem_descriptors,
        acpi_tables,
    })
}
