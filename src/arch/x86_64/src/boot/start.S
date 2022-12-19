/* This symbol is known by the linker script. */
.global KERNEL_ENTRY
KERNEL_ENTRY = _start

.section ".phys.text", "ax", @progbits
.align 8
.code32
_start:
    cli
    cld

    /* Verify we were booted by a Multiboot 2 compliant bootloader. */
    cmpl    $0x36d76289, %eax
    jne     1f

    /* Manually zero the physical BSS. */
    xorl    %eax, %eax
    movl    $_phys_bss, %edi
    movl    $_ephys_bss, %ecx
    subl    %edi, %ecx
    shrl    $2, %ecx
    rep     stosl

    /* Setup a temporary stack. */
    leal    boot_stack_top, %esp

    /* Save multiboot info. */
    pushl   $0
    pushl   %ebx

    /* Reset EFLAGS. */
    pushl   $0
    popfl

    /* 
     * Verify that the CPU supports the features we need.
     * We do this by checking for CPUID support followed by checking for SSE
     * and longmode support.
     *
     * See vol 3A section 2.3.
     *
     * Check for CPUID support by attempting to flip EFLAGS.ID.
     */
    pushfl
    popl    %eax
    movl    %eax, %ebx
    xorl    $(1 << 21), %eax
    pushl   %eax
    popfl
    pushfl
    popl    %eax
    cmpl    %eax, %ebx
    jz      1f

    /* Check for SSE support. */
    movl    $0x1, %eax
    cpuid
    testl   $(1 << 25), %edx
    jz      1f

    /* Check for extended-mode (required to check for longmode). */
    movl    $0x80000000, %eax
    cpuid
    cmpl    $0x80000001, %eax
    jb      1f

    /* Check for longmode. */
    movl    $0x80000001, %eax
    cpuid
    testl   $(1 << 29), %edx
    jz      1f

.Lpaging_setup:
    /* 
     * Initialize the PML4.
     *
     * The boot PML4 contains two entries. The first entry is used to identity-
     * map the first 4G of physical memory. The second entry (the last entry in
     * the table) is used to map the top -2G of virtual memory to the first 2G
     * of physical memory (where the kernel is located).
     */
    movl    $BOOT_PML4, %edi

    movl    $BOOT_PDPT, %eax
    orl     $(1 << 2 | 1 << 1 | 1 << 0), %eax
    movl    %eax, (%edi)
    movl    %eax, 511 * 8(%edi)

    /* 
     * Initialize the PDPT. We can share the same PD table between the two PDP
     * tables but we will only use the first 2 PD tables for the higher-memory
     * map.
     */
    movl    $BOOT_PDPT, %edi
    movl    $BOOT_PDS, %eax
    orl     $(1 << 2 | 1 << 1 | 1 << 0), %eax

    movl    %eax, 0(%edi)
    movl    %eax, 510 * 8(%edi)
    
    addl    $(512 * 8), %eax
    movl    %eax, 8(%edi)
    movl    %eax, 511 * 8(%edi)

    addl    $(512 * 8), %eax
    movl    %eax, 16(%edi)

    addl    $(512 * 8), %eax
    movl    %eax, 24(%edi)

    /*
     * Initialise the PDs. We just cover the first 4G of physical memory since
     * that's all we need to be able to get to Rust. Once we are in Rust, we
     * can do some more fine-grained mapping.
     */
    movl    $BOOT_PDS, %edi
    movl    $(1 << 8 | 1 << 7 | 1 << 1 | 1 << 0), %eax
    movl    $(512 * 4), %ecx
0:
    movl    %eax, (%edi)
    addl    $0x200000, %eax
    addl    $8, %edi
    loop    0b

.Lenable_longmode:
    /* Disable PG. */
    movl    %cr0, %eax
    andl    $~(1 << 31), %eax
    movl    %eax, %cr0

    /* Load the early GDT. */
    lgdt    BOOT_GDT_PTR

    /* Enable PAE and PGE. */
    movl    %cr4, %eax
    orl     $(1 << 7 | 1 << 5), %eax
    movl    %eax, %cr4

    /* Load the PML4 table into cr3. */
    leal    BOOT_PML4, %eax
    movl    %eax, %cr3

    /* Enable long mode. */
    movl    $0xc0000080, %ecx
    rdmsr
    orl     $(1 << 8), %eax
    wrmsr

    /* Prepare to jump into long mode. */
    pushl   $8
    movl    $low_entry, %eax
    pushl   %eax

    /* Enable PG and PE */
    movl    $(1 << 31 | 1 << 0), %eax
    movl    %eax, %cr0

    /* Jump into long mode. */
    lret
1:
    hlt
    jmp     1b

.code64
low_entry:
    /*
     * At this point we're still running in a low 'physical' address.
     * Leave it behind and start running in the higher-half virtual address.
     */
    movabs  $high_entry, %rax
    jmp     *%rax

.text
.extern     boot
high_entry:
    /*
     * At this point we're running in a virtual address. All code here must be
     * pure PIC.
     */

    /* Clear segment registers. */
    xorl    %eax, %eax
    movl    %eax, %ds
    movl    %eax, %ss
    movl    %eax, %es
    movl    %eax, %gs
    movl    %eax, %fs

    /* Retrieve multiboot info from the boot stack. */
    pop     %rdi

    /* Load kernel stack. It is defined in `main.rs`. */
    leaq    KERNEL_STACK+0x4000(%rip), %rsp

    /* Jump into Rust. */
    call    boot

    /* boot is not supposed to return, but if it does, just hang on. */
1:
    hlt
    jmp     1b


/*
 * Initial boot stack.
 */
.section ".phys.bss"
.balign 16
boot_stack_bottom:
    .skip   8192
boot_stack_top:
