/// A copy of the general purpose registers.
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct Registers {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rax: usize,
    rbx: usize,
    rcx: usize,
    rdx: usize,
    rbp: usize,
    rdi: usize,
    rsi: usize,
}

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct IretStack {
    ip: usize,
    cs: usize,
    flags: usize,
    user_sp: usize,
    user_ss: usize,
}

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct Frame {
    regs: Registers,
    iret: IretStack,
}

/// This macro evaluates to 1 if an identifier is supplied.
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! __interrupt_handler__is_set {
    ($_:ident) => {
        1
    };

    () => {
        0
    };
}

/// Declare an interrupt handler function.
#[macro_export]
macro_rules! interrupt_handler {
    (
        #[paranoid]
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        interrupt_handler!{
            @paranoid=1
            $(#[$meta])*
            $vis fn $name($frame: &mut $frame_type $(, $error: usize)?) $code
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        interrupt_handler!{
            @paranoid=0
            $(#[$meta])*
            $vis fn $name($frame: &mut $frame_type $(, $error: usize)?) $code
        }
    };

    (
        @paranoid=$paranoid:literal
        $(#[meta:meta])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        #[naked]
        $vis unsafe extern "C" fn $name() {
            unsafe extern "C" fn inner($frame: &mut $frame_type $(, $error: usize)?) {
                let _: &mut $crate::interrupts::handler::Frame = $frame;
                $code
            }
            core::arch::asm!(
                "
                    cld

                    // Save the registers
                .if {error}
                    // Error-code is present -> swap the top of the stack with %rsi.
                    // %rsi now contains the error-code, while the stack contains the
                    // original value of %rsi.
                    xchg    %rsi, (%rsp) 
                .else
                    // Not expecting an error-code, just push %rsi on the stack.
                    push    %rsi
                .endif

                    push    %rdi
                    push    %rbp
                    push    %rdx
                    push    %rcx
                    push    %rbx
                    push    %rax
                    push    %r8
                    push    %r9
                    push    %r10
                    push    %r11
                    push    %r12
                    push    %r13
                    push    %r14
                    push    %r15
            
                    // Stack contains a full frame now.
                    mov     %rsp, %rdi

                .if {paranoid}
                    // This is a paranoid interrupt, we need to take extra care to handle it.
                .else
                    // Did we come from userspace?
                    testb   $0b11, (16*8)(%rsp)
                    jz      1f
                    swapgs
                .endif
                1:

                    // Call the handler.
                    call    {inner}

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
                    pop     %r11
                    pop     %r10
                    pop     %r9
                    pop     %r8
                    pop     %rax
                    pop     %rbx
                    pop     %rcx
                    pop     %rdx
                    pop     %rbp
                    pop     %rdi
            
                    // Restore interrupt stack frame to its original state.
                .if {error}
                    xchg    (%rsp), %rsi
                .else
                    pop     %rsi
                .endif
            
                    // Return to caller.
                    iretq
                ",
                inner = sym inner,
                error = const $crate::__interrupt_handler__is_set!($($error)?),
                paranoid = const($paranoid),
                options(noreturn, att_syntax)
            );
        }
    };
}
