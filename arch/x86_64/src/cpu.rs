use spin::Once;
use x86::{
    controlregs::{cr0, cr0_write, cr4, cr4_write, Cr0, Cr4},
    cpuid::{
        self, ExtendedFeatures, ExtendedProcessorFeatureIdentifiers, ExtendedStateInfo,
        FeatureInfo, ProcessorBrandString, VendorInfo,
    },
    msr::{self, rdmsr, wrmsr, IA32_EFER},
};

use crate::println;

/// A wrapper over the CpuId type provided by the x86 crate.
#[derive(Debug)]
pub struct CpuId {
    pub vendor_info: VendorInfo,
    pub features: FeatureInfo,
    pub ext_state: ExtendedStateInfo,
    pub ext_features: ExtendedFeatures,
    pub ext_proc_feature_ids: ExtendedProcessorFeatureIdentifiers,
    pub brand_string: ProcessorBrandString,
}

impl CpuId {
    pub fn read() -> Self {
        let cpuid = cpuid::CpuId::new();

        Self {
            vendor_info: cpuid.get_vendor_info().expect("Failed to get vendor info"),
            features: cpuid
                .get_feature_info()
                .expect("Failed to get feature info"),
            ext_state: cpuid
                .get_extended_state_info()
                .expect("Failed to get extended state info"),
            ext_features: cpuid
                .get_extended_feature_info()
                .expect("Failed to get extended features"),
            ext_proc_feature_ids: cpuid
                .get_extended_processor_and_feature_identifiers()
                .expect("Failed to get extended processor and feature identifiers"),
            brand_string: cpuid
                .get_processor_brand_string()
                .expect("Failed to get processor brand string"),
        }
    }
}

/// Query CPU information.
pub fn cpuid() -> &'static CpuId {
    static CPUID: Once<CpuId> = Once::new();
    CPUID.call_once(CpuId::read)
}

/// Enable essential CPU features.
///
/// These are the bare minimum features required. They should be enabled before
/// switching to the kernel pages (as the kernel pages depend on them).
pub fn pre_mm_init() -> Result<(), &'static str> {
    // We use 1G pages to map all the physical memory.
    if !cpuid().ext_proc_feature_ids.has_1gib_pages() {
        return Err("1G pages");
    }

    if !cpuid().features.has_apic() || !cpuid().features.has_x2apic() {
        return Err("apic");
    }

    if cpuid().ext_proc_feature_ids.has_execute_disable() {
        const NXE: u64 = 0x800;
        unsafe {
            let mut efer = rdmsr(IA32_EFER);
            efer |= NXE;
            wrmsr(IA32_EFER, efer);
        }
    } else {
        return Err("execute disable");
    }

    if cpuid().features.has_pat() {
        unsafe {
            let mut cr0 = cr0();
            cr0.set(Cr0::CR0_NOT_WRITE_THROUGH, false);
            cr0.set(Cr0::CR0_CACHE_DISABLE, false);
            cr0.set(Cr0::CR0_WRITE_PROTECT, true);
            cr0_write(cr0);
        }
    } else {
        return Err("pat");
    }

    if cpuid().features.has_pge() {
        unsafe {
            let mut cr4 = cr4();
            cr4 |= Cr4::CR4_ENABLE_GLOBAL_PAGES;
            cr4_write(cr4);
        }
    } else {
        return Err("pge");
    }

    Ok(())
}

/// Initialise the CPU.
pub unsafe fn init_cpu() {
    let features = CpuId::read();

    let mut cr4 = cr4();
    if features.features.has_pcid() {
        println!("Enabling PCID");
        //cr4 |= Cr4::CR4_ENABLE_PCID;
    }

    if features.features.has_fxsave_fxstor() {
        println!("Enabling OS_XSAVE");
        //cr4 |= Cr4::CR4_ENABLE_OS_XSAVE;
    }

    if features.ext_features.has_fsgsbase() {
        println!("Enabling FSGSBASE");
        //cr4 |= Cr4::CR4_ENABLE_FSGSBASE;
    }

    if features.ext_features.has_smap() {
        println!("Enabling SMAP");
        //cr4 |= Cr4::CR4_ENABLE_SMAP;
    }

    if features.ext_features.has_smep() {
        println!("Enabling SMEP");
        //cr4 |= Cr4::CR4_ENABLE_SMEP;
    }

    if features.features.has_pge() {
        println!("Enabling PGE");
        //cr4 |= Cr4::CR4_ENABLE_GLOBAL_PAGES;
    }

    cr4_write(cr4);

    let mut efer = msr::rdmsr(IA32_EFER);

    if features.ext_proc_feature_ids.has_execute_disable() {
        println!("Enabling NXE");
        //efer |= 0x800;
    }

    wrmsr(IA32_EFER, efer);

    /* FEATURES.with_borrow_mut(|f| {
        let _ = f.insert(features);
    }); */
}

/// Start the CPU.
pub fn start_cpu() -> ! {
    // initialise the cpu first.

    println!("Starting cpu...");

    // TODO: scheduler start shit, spin forever.
    loop {}
}
