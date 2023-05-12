use x86::{
    controlregs::{cr4, cr4_write, Cr4},
    cpuid, fs_deref, gs_deref,
    msr::{self, rdmsr, wrmsr, IA32_EFER},
};

use crate::{gdt, interrupts, println};

/// Initialise the CPU.
pub unsafe fn init_cpu() -> u8 {
    let cpuid = cpuid::CpuId::new();

    let features = cpuid
        .get_feature_info()
        .expect("Failed to read CPU features!");

    let ext_features = cpuid
        .get_extended_feature_info()
        .expect("Failed to read CPU extended features!");

    let proc_feature_ids = cpuid
        .get_extended_processor_and_feature_identifiers()
        .expect("Failed to get extended processor feature identifiers");

    let mut cr4 = cr4();
    if features.has_pcid() {
        println!("Enabling PCID");
        cr4 |= Cr4::CR4_ENABLE_PCID;
    }

    if features.has_fxsave_fxstor() {
        println!("Enabling OS_XSAVE");
        cr4 |= Cr4::CR4_ENABLE_OS_XSAVE;
    }

    if ext_features.has_fsgsbase() {
        println!("Enabling FSGSBASE");
        cr4 |= Cr4::CR4_ENABLE_FSGSBASE;
    }

    if ext_features.has_smap() {
        println!("Enabling SMAP");
        cr4 |= Cr4::CR4_ENABLE_SMAP;
    }

    if ext_features.has_smep() {
        println!("Enabling SMEP");
        cr4 |= Cr4::CR4_ENABLE_SMEP;
    }

    if features.has_pge() {
        println!("Enabling PGE");
        cr4 |= Cr4::CR4_ENABLE_GLOBAL_PAGES;
    }

    cr4_write(cr4);

    let mut efer = msr::rdmsr(IA32_EFER);

    if proc_feature_ids.has_execute_disable() {
        println!("Enabling NXE");
        efer |= 0x800;
    }

    wrmsr(IA32_EFER, efer);

    // VMX
    // Timer
    // kernel space cpu mappings

    0
}

/// Initialise the MSRs.
unsafe fn init_msrs() {
    // TODO: fs/gs base
}

#[inline]
pub fn is_bsc() -> bool {
    let msr = unsafe { rdmsr(msr::APIC_BASE) };
    (msr & (1 << 8)) == (1 << 8)
}

/// Start the CPU.
pub fn start_cpu() -> ! {
    // initialise the cpu first.

    println!("Starting cpu...");

    // TODO: scheduler start shit, spin forever.
    loop {}
}
