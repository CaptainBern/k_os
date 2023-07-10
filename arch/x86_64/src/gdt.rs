use core::{cell::RefCell, mem};

use x86::{
    dtables::{lgdt, DescriptorTablePointer},
    segmentation::{load_cs, SegmentSelector},
    task::load_tr,
    Ring,
};

use crate::{
    desc::{
        Access, CodeSegmentBits, DataSegmentBits, DescriptorFlags, SystemDescriptor,
        SystemDescriptorType, Tss, UserDescriptor, UserDescriptorType, IOPB_BYTES,
    },
    percpu,
};

pub const NMI_IST_INDEX: u8 = 1;
pub const DF_IST_INDEX: u8 = 2;
pub const MC_IST_INDEX: u8 = 3;

percpu! {
    /// Per-cpu GDT.
    static GDT: RefCell<KernelGdt> = RefCell::new(KernelGdt::new());

    /// Per-cpu TSS.
    static KERNEL_TSS: RefCell<KernelTss> = RefCell::new(KernelTss::zero());
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct KernelTss {
    tss: Tss,
    bitmap: [u8; IOPB_BYTES + 1],
}

impl KernelTss {
    pub const fn zero() -> Self {
        let mut bitmap = [0u8; IOPB_BYTES + 1];

        // Last entry must be 0xff according to the Intel and AMD manuals.
        bitmap[IOPB_BYTES] = 0xff;

        Self {
            tss: Tss::new_with_base(0, mem::offset_of!(KernelTss, bitmap) as u16),
            bitmap,
        }
    }
}

/// The kernel GDT.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct KernelGdt {
    pub null: UserDescriptor,
    pub kernel_code: UserDescriptor,
    pub kernel_data: UserDescriptor,
    pub user_code_32: UserDescriptor,
    pub user_code_64: UserDescriptor,
    pub user_data: UserDescriptor,
    pub kernel_tss: SystemDescriptor,
}

impl KernelGdt {
    pub const fn new() -> Self {
        KernelGdt {
            /* 0 */
            null: UserDescriptor::NULL,
            /* 1 */
            kernel_code: UserDescriptor::new(
                0,
                0,
                UserDescriptorType::Code(CodeSegmentBits::EX_ONLY),
                Access::KERNEL_USR,
                DescriptorFlags::L,
            ),
            /* 2 */
            kernel_data: UserDescriptor::new(
                0,
                0,
                UserDescriptorType::Data(DataSegmentBits::READ_WRITE),
                Access::KERNEL_USR,
                DescriptorFlags::L,
            ),
            /* 3 */
            user_code_32: UserDescriptor::new(
                0,
                0,
                UserDescriptorType::Code(CodeSegmentBits::empty()),
                Access::empty(),
                DescriptorFlags::empty(),
            ),
            /* 4 */
            user_code_64: UserDescriptor::new(
                0,
                0,
                UserDescriptorType::Code(CodeSegmentBits::EX_ONLY),
                Access::from_bits_truncate(
                    Access::P.bits() | Access::S.bits() | Access::DPL_3.bits(),
                ),
                DescriptorFlags::L,
            ),
            /* 5 */
            user_data: UserDescriptor::new(
                0,
                0,
                UserDescriptorType::Data(DataSegmentBits::READ_WRITE),
                Access::from_bits_truncate(
                    Access::P.bits() | Access::S.bits() | Access::DPL_3.bits(),
                ),
                DescriptorFlags::L,
            ),
            /* 6 */
            kernel_tss: SystemDescriptor::NULL,
        }
    }

    pub fn set_tss(&mut self, tss: SystemDescriptor) {
        self.kernel_tss = tss;
    }
}

/// Set the given RSP in the TSS to `stack`.
fn set_tss_rsp(rsp: u8, stack: u64) {
    KERNEL_TSS.with_borrow_mut(|tss| tss.tss.rsp[rsp as usize] = stack);
}

/// Set the given IST in the TSS to `stack`.
fn set_tss_ist(ist: u8, stack: u64) {
    KERNEL_TSS.with_borrow_mut(|tss| tss.tss.ist[ist as usize - 1] = stack);
}

/// Setup the GDT and TSS structures.
pub unsafe fn init(kstack: u64, nmi_stack: u64, df_stack: u64, mc_stack: u64) {
    set_tss_rsp(0, kstack);

    // Make sure these interrupts always execute on a known good stack.
    set_tss_ist(NMI_IST_INDEX, nmi_stack);
    set_tss_ist(DF_IST_INDEX, df_stack);
    set_tss_ist(MC_IST_INDEX, mc_stack);

    GDT.with_borrow_mut(|gdt| {
        let base = KERNEL_TSS.with(RefCell::as_ptr);
        gdt.set_tss(SystemDescriptor::new(
            base as u64,
            (mem::size_of::<KernelTss>() - 1) as u32,
            SystemDescriptorType::Tss,
            Access::KERNEL_SYS,
            DescriptorFlags::G,
        ))
    });
}

/// Load the GDT.
pub unsafe fn load() {
    GDT.with(|gdt| {
        let ptr = DescriptorTablePointer {
            limit: (mem::size_of::<KernelGdt>() - 1) as u16,
            base: gdt.as_ptr(),
        };

        unsafe {
            lgdt(&ptr);
            load_cs(SegmentSelector::new(1, Ring::Ring0));
            load_tr(SegmentSelector::new(6, Ring::Ring0));
        }
    });
}
