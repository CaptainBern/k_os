/* This symbol is known by the linker script. */
.global KERNEL_ENTRY
KERNEL_ENTRY = _start

VIRT_OFFSET = 0xffffffff80000000;

.section ".text.boot", "ax", @progbits
.balign 8
.code32
_start:
    cli
    cld

    /* Verify we were booted by a Multiboot 2 compliant bootloader. */
    cmpl    $0x36d76289, %eax
    jne     1f

    /* Manually zero BSS, just in case. */
    xorl    %eax, %eax
    movl    $(_bss - VIRT_OFFSET), %edi
    movl    $(_ebss - VIRT_OFFSET), %ecx
    subl    %edi, %ecx
    shrl    $2, %ecx
    rep     stosl

    /* Setup bsp stack. */
    leal    ((BOOT_STACK+0x4000) - VIRT_OFFSET), %esp
    movl    %esp, %ebp

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

    /* Check for extended-mode. */
    movl    $0x80000000, %eax
    cpuid
    cmpl    $0x80000001, %eax
    jb      1f

    /* Check for longmode. */
    movl    $0x80000001, %eax
    cpuid
    testl   $(1 << 29), %edx
    jz      1f

    /* Check for XD. */
    movl    $0x80000001, %eax
    cpuid
    testl   $(1 << 20), %edx
    jz      1f

    /* Check for 1G pages. */
    movl    $0x80000001, %eax
    cpuid
    testl   $(1 << 26), %edx
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
    movl    $(BOOT_PML4 - VIRT_OFFSET), %edi

    movl    $(BOOT_PDPT - VIRT_OFFSET), %eax
    orl     $(1 << 2 | 1 << 1 | 1 << 0), %eax
    movl    %eax, (%edi)
    movl    %eax, 511 * 8(%edi)

    /* 
     * Initialize the PDPT. We can share the same PD table between the two PDP
     * tables but we will only use the first 2 PD tables for the higher-memory
     * map.
     */
    movl    $(BOOT_PDPT - VIRT_OFFSET), %edi
    movl    $(BOOT_PDS - VIRT_OFFSET), %eax
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
    movl    $(BOOT_PDS - VIRT_OFFSET), %edi
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
    sub     $6, %esp
    movw    (BOOT_GDT_PTR - VIRT_OFFSET), %ax
    movl    ((BOOT_GDT_PTR+2) - VIRT_OFFSET), %ebx
    subl    $VIRT_OFFSET, %ebx
    movw    %ax, -6(%esp)
    movl    %ebx, -4(%esp)
    lgdt    -6(%esp)

    /* Enable PAE and PGE. */
    movl    %cr4, %eax
    orl     $(1 << 7 | 1 << 5), %eax
    movl    %eax, %cr4

    /* Load the PML4 table into cr3. */
    leal    (BOOT_PML4 - VIRT_OFFSET), %eax
    movl    %eax, %cr3

    /* Enable long mode and NXE. */
    movl    $0xc0000080, %ecx
    rdmsr
    orl      $(1 << 11 | 1 << 8), %eax
    wrmsr

    /* Prepare to jump into long mode. */
    pushl   $(1 << 3)
    movl    $low_entry, %eax
    pushl   %eax

    /* Enable CR0.PG */
    movl    %cr0, %eax
    orl     $(1 << 31), %eax
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
.extern     pre_boot
high_entry:
    /*
     * At this point we're running in a virtual address. From now on, all code
     * must be PIC.
     */

    /* Clear segment registers. */
    xorl    %eax, %eax
    movl    %eax, %ds
    movl    %eax, %ss
    movl    %eax, %es
    movl    %eax, %gs
    movl    %eax, %fs

    /* Reload GDT with the proper, virtual, pointer. */
    lgdt    BOOT_GDT_PTR(%rip)

    /* We're on the proper GDT ptr now, so set stack to previous state. */
    addq    $6, %rsp

    /* Update stack offset to the correct, virtual, address. */
    addq    $VIRT_OFFSET, %rsp
    addq    $VIRT_OFFSET, %rbp

    /* Retrieve multiboot info from the boot stack. */
    pop     %rdi

    /* Jump into Rust. */
    call    pre_boot

    /* boot is not supposed to return, but if it does, just hang on. */
1:
    hlt
    jmp     1b