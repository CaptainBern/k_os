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

#[doc(hidden)]
#[macro_export]
macro_rules! __interrupt_handler_emit_asm {
    (
        @paranoid=$paranoid:literal,
        @error=$error:literal,
        @inner=$inner:ident
    ) => {
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
                pop     %rsi
        
                // Return to caller.
                iretq
            ",
            paranoid = const($paranoid),
            error = const ($error),
            inner = sym $inner,
            options(noreturn, att_syntax)
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __interrupt_handler_emit_stub {
    (
        @paranoid=$paranoid:literal
        fn $inner:ident($frame:ident: &mut $frame_type:ty, $error:ident: usize) $code:block
    ) => {
        fn $inner($frame: &mut $frame_type, $error: usize) {
            // Check if frame is the correct type.
            let _: &mut $crate::interrupts::handler::Frame = $frame;
            $code
        }
        $crate::__interrupt_handler_emit_asm!(
            @paranoid=$paranoid,
            @error=1,
            @inner=$inner
        );
    };

    (
        @paranoid=$paranoid:literal
        fn $inner:ident($frame:ident: &mut $frame_type:ty) $code:block
    ) => {
        fn $inner($frame: &mut $frame_type) {
            // Check if frame is the correct type.
            let _: &mut $crate::interrupts::handler::Frame = $frame;
            $code
        }
        $crate::__interrupt_handler_emit_asm!(
            @paranoid=$paranoid,
            @error=0,
            @inner=$inner
        );
    };
}

/// Declare an interrupt handler.
/// Every handler should have a [Frame] as its first argument:
/// ```
/// interrupt_handler! {
///     pub fn breakpoint(frame: &mut Frame) {
///         println!("Received a breakpoint!");
///     }
/// }
/// ```
/// Some interrupts will push an additional error code on the stack. To handle
/// these cases, an additional argument of type [usize] should be added to the
/// handler function:
/// ```
/// interrupt_handler! {
///     pub fn page_fault(frame: &mut Frame, error: usize) {
///         println!("Got a pagefault with error: {}", error);
///     }
/// }
/// ```
///
/// It is possible to define attributes for the handler, and to change
/// the visibility modifier. However, the [Frame] argument *MUST* be mutable.
///
/// Finally, some interrupts might trigger before an earlier interrupt is
/// finished. These interrupts should be 'paranoid' in the way they check if
/// the interrupt came from user- or kernelspace. An interrupt handler can be
/// marked as 'paranoid' by adding a '\#\[paranoid\]' attribute:
/// ```
/// interrupt_handler! {
///     #[paranoid]
///     pub fn nmi(frame: &mut Frame) {
///         println!("Got a paranoid interrupt!");
///     }
/// }
/// ```
#[macro_export]
macro_rules! interrupt_handler {
    (
        @finalize
        paranoid=$paranoid:literal,
        attributes=$(#[$($attrs:tt)*])*,
        function=$vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        $(#[$($attrs)*])*
        #[naked]
        $vis unsafe extern "C" fn $name() {
            $crate::__interrupt_handler_emit_stub!(
                @paranoid=$paranoid
                fn inner($frame: &mut $frame_type $(, $error: usize)?) $code
            );
        }
    };

    // Recurse: no attributes defined.
    (
        @decl_handler_recursive
        head={},
        tail={},
        function=$vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        interrupt_handler! {
            @finalize
            paranoid=0,
            attributes=,
            function=$vis fn $name($frame: &mut $frame_type $(, $error: usize)?) $code
        }
    };

    // Recurse: list fully processed, there was no '#[paranoid]'.
    (
        @decl_handler_recursive
        head={
            $(#[$($attrs:tt)*])*
        },
        tail={},
        function=$vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        interrupt_handler! {
            @finalize
            paranoid=0,
            attributes=$(#[$($attrs)*])*,
            function=$vis fn $name($frame: &mut $frame_type $(, $error: usize)?) $code
        }
    };

    // Recurse: tail contains '#[paranoid]'. Since it should only be defined once, we
    // can stop here.
    (
        @decl_handler_recursive
        head={
            $(#[$($head:tt)*])*
        },
        tail={
            #[paranoid]
            $(#[$($tail:tt)*])*
        },
        function=$vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        interrupt_handler! {
            @finalize
            paranoid=1,
            attributes=$(#[$($head)*])* $(#[$($tail)*])*,
            function=$vis fn $name($frame: &mut $frame_type $(, $error: usize)?) $code
        }
    };

    // Recurse: tail contains at least one element, it is not '#[paranoid]', so
    // move it to head and continue processing tail (if there is anything left).
    (
        @decl_handler_recursive
        head={
            $(#[$($head:tt)*])*
        },
        tail={
            #[$($attr:tt)*]
            $(#[$($tail:tt)*])*
        },
        function=$vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        interrupt_handler! {
            @decl_handler_recursive
            head={
                $(#[$($head)*])*
                #[$($attr)*]
            },
            tail={
                $(#[$($tail)*])*
            },
            function=$vis fn $name($frame: &mut $frame_type $(, $error: usize)?) $code
        }
    };

    // Actual API.
    (
        $(#[$($attrs:tt)*])*
        $vis:vis fn $name:ident($frame:ident: &mut $frame_type:ty $(, $error:ident: usize)?) $code:block
    ) => {
        interrupt_handler! {
            @decl_handler_recursive
            head={},
            tail={
                $(#[$($attrs)*])*
            },
            function=$vis fn $name($frame: &mut $frame_type $(, $error: usize)?) $code
        }
    };
}
