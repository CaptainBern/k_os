use x86::bits64::paging::{PML4, PML4Entry, PDPT, PDPTEntry, PD, PDEntry};

#[no_mangle]
static mut KERNEL_PML4: PML4 = [PML4Entry(0); 512];

#[no_mangle]
static mut KERNEL_PDPT: PDPT = [PDPTEntry(0); 512];

#[no_mangle]
static mut KERNEL_PDS: [PD; 4] = [
    [PDEntry(0); 512],
    [PDEntry(0); 512],
    [PDEntry(0); 512],
    [PDEntry(0); 512]
];