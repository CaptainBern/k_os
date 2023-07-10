use crate::{linker, percpu};

percpu! {
    static NMI_STACK: IrqStack = IrqStack::zero();
    static DF_STACK: IrqStack = IrqStack::zero();
    static MC_STACK: IrqStack = IrqStack::zero();
}

#[repr(C, align(16))]
pub struct IrqStack(pub [u8; linker::INTERRUPT_STACK_SIZE]);

impl IrqStack {
    pub const fn zero() -> Self {
        Self([0; linker::INTERRUPT_STACK_SIZE])
    }

    pub fn top(&self) -> u64 {
        self.0.as_ptr() as u64 + linker::INTERRUPT_STACK_SIZE as u64
    }
}

/// Returns the top of the NMI stack for the current CPU.
pub fn nmi_stack_top() -> u64 {
    NMI_STACK.with(IrqStack::top)
}

/// Returns the top of the DF stack for the current CPU.
pub fn df_stack_top() -> u64 {
    DF_STACK.with(IrqStack::top)
}

/// Returns the top of the MC stack for the current CPU.
pub fn mc_stack_top() -> u64 {
    MC_STACK.with(IrqStack::top)
}
