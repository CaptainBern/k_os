/// Register values saved on entering kernel through an interrupt. They will be
/// restored upon returning to userspace (or caller).
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct Regs {
    // Preserved registers
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,

    // Scratch registers
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rdi: u64,
    pub rsi: u64,
}

/// Interrupt Stack Frame. On an interrupt, the processor will push these
/// values on the stack. After the interrupt is handled, the processor will
/// resume to rip.
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct IRetStack {
    /// The return instruction pointer.
    pub rip: u64,
    /// The return code-segment selector.
    pub cs: u64,
    /// A copy of the RFLAGS register. The upper 32 bits are written as zeros.
    pub rflags: u64,
    /// The return stack-pointer.
    pub rsp: u64,
    /// The return stack-segment selector.
    pub ss: u64,
}

/// Interrupt frame, every interrupt handler has access to these values.
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct Frame {
    pub regs: Regs,
    pub iret: IRetStack,
}

#[doc(hidden)]
#[macro_export]
macro_rules! __interrupt_handler_internal {
    (
        @paranoid=$paranoid:literal
        @has_error=$has_error:literal
        $(#[$($attrs:tt)*])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_ty:ty $(, $error_code:ident: u64)?) $code:block
    ) => {
        $(
            #[$($attrs)*]
        )*
        #[naked]
        $vis unsafe extern "C" fn $name() {
            #[allow(dead_code)]
            extern "C" fn rust($frame: &mut $frame_ty $(, $error_code: u64)?) {
                let _: &mut $crate::idt::handler::Frame = $frame; // type guard.

                {
                    $code
                }
            }
            core::arch::asm!(
                "   
                    cld

                    // Save the registers
                .if {has_error}
                    // Swap rsi with error code.
                    xchg    %rsi, (%rsp) 
                .else
                    push    %rsi
                .endif

                    push    %rdi
                    push    %rdx
                    push    %rcx
                    push    %rax
                    push    %r8
                    push    %r9
                    push    %r10
                    push    %r11
                    push    %rbx
                    push    %rbp
                    push    %r12
                    push    %r13
                    push    %r14
                    push    %r15

                    // Using 'xorl' is faster than 'xorq', while still zero-ing
                    // all 64 bits.
                .if !{has_error}
                    // rsi does not contain an error code, so zero it.
                    xorl    %esi, %esi
                .endif
                    xorl    %edi, %edi
                    xorl    %edx, %edx
                    xorl    %ecx, %ecx
                    xorl    %eax, %eax
                    xorl    %r8d, %r8d
                    xorl    %r9d, %r9d
                    xorl    %r10d, %r10d
                    xorl    %r11d, %r11d
                    xorl    %ebx, %ebx
                    xorl    %ebp, %ebp
                    xorl    %r12d, %r12d
                    xorl    %r13d, %r13d
                    xorl    %r14d, %r14d
                    xorl    %r15d, %r15d
        
                    // Stack contains a full frame now. If there is an error
                    // code, it is in rsi already.
                    mov     %rsp, %rdi

                .if {paranoid}
                    // TODO
                .else
                    // Did we come from userspace?
                    testb   $0b11, (16*8)(%rsp)
                    jz      1f
                    swapgs
                .endif
                1:
                    // Call rust
                    call    {rust}

                    // Check if we are returning to userspace.
                .if {paranoid}
                .else
                    testb   $0b11, (16*8)(%rsp)
                    jz      1f
                    swapgs
                .endif
                1:
                    // Restore registers
                    pop     %r15
                    pop     %r14
                    pop     %r13
                    pop     %r12
                    pop     %rbp
                    pop     %rbx
                    pop     %r11
                    pop     %r10
                    pop     %r9
                    pop     %r8
                    pop     %rax
                    pop     %rcx
                    pop     %rdx
                    pop     %rdi
                    pop     %rsi
        
                    // Return to caller.
                    iretq
                ",
                rust = sym rust,
                paranoid = const($paranoid),
                has_error = const($has_error),
                options(att_syntax, noreturn)
            );
        }
    };
}

/// Define a paranoid interrupt handler.
#[macro_export(local_inner_macros)]
macro_rules! paranoid_interrupt_handler {
    (
        $(#[$($attrs:tt)*])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_ty:ty, $error_code:ident: u64) $code:block
    ) => {
        __interrupt_handler_internal! {
            @paranoid=1
            @has_error=1
            $(#[$($attrs)*])*
            $vis fn $name($frame: &mut $frame_ty $(, $error_code: u64)?) $code
        }
    };

    (
        $(#[$($attrs:tt)*])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_ty:ty) $code:block
    ) => {
        __interrupt_handler_internal! {
            @paranoid=1
            @has_error=0
            $(#[$($attrs)*])*
            $vis fn $name($frame: &mut $frame_ty) $code
        }
    };
}

/// Define an interrupt handler.
#[macro_export(local_inner_macros)]
macro_rules! interrupt_handler {
    (
        $(#[$($attrs:tt)*])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_ty:ty, $error_code:ident: u64) $code:block
    ) => {
        __interrupt_handler_internal! {
            @paranoid=0
            @has_error=1
            $(#[$($attrs)*])*
            $vis fn $name($frame: &mut $frame_ty, $error_code: u64) $code
        }
    };

    (
        $(#[$($attrs:tt)*])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_ty:ty) $code:block
    ) => {
        __interrupt_handler_internal! {
            @paranoid=0
            @has_error=0
            $(#[$($attrs)*])*
            $vis fn $name($frame: &mut $frame_ty) $code
        }
    };
}
