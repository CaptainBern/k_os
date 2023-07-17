/*
 * Boot code for APs.
 */
.section ".text.boot16", "ax", @progbits
.balign 4096

BOOTSTRAP_GDT_PTR_OFFSET = 0;
BOOTSTRAP_KERNEL_TOP_OFFSET = 42;
BOOTSTRAP_STACK_OFFSET = 50;
BOOTSTRAP_PERCPU_OFFSET = 58;

.code16
.global _start16
_start16:
    cli
    cld

    /* 
     * Keep track of %cs. We use it to calculate offsets relative to the 
     * location we're running at.
     */
    movw    %cs, %si
    shll    $4, %esi

    /* 
     * Bootstrap data is located in the next 4K frame, relative to `%cs`. 
     * Update %ds and %ss so we have access to it. We need to load these
     * to have access to the GDT and other bootstrap data.
     */
    mov     %cs, %dx
    add     $0x100, %dx
    movw    %dx, %ds

    /*
     * Manually fixup the protected mode jump address. We have to this because
     * the effective address is different from the link-time address.
     */
    lea     (protected_mode - _start16)(%esi), %eax
    mov     %eax, %cs:(.Lpm_entry - _start16)
    
    /*
     * Do the same for the long mode jump address. 
     */
    lea     (long_mode - _start16)(%esi), %eax
    mov     %eax, %cs:(.Llm_entry - _start16)

    /* Load bootstrap GDT. */
    lgdtl   %ds:BOOTSTRAP_GDT_PTR_OFFSET

    /* Enable CR0.PE, entering protected mode. */
    mov     %cr0, %eax
    or      $1, %eax
    mov     %eax, %cr0

    .byte   0x66, 0xea # ljmp opcode
.Lpm_entry:    
    .long   0          # offset
    .word   (1 << 3)   # segment

protected_mode:
    /* Enable PAE and PGE. */
    mov     %cr4, %eax
    or      $(1 << 7 | 1 << 5), %eax
    mov     %eax, %cr4

    /* Load the boot PML4 table into cr3. */
    mov     $(BOOT_PML4 - VIRT_OFFSET), %eax
    mov     %eax, %cr3

    /* Enable long mode and NXE. */
    mov     $0xc0000080, %ecx
    rdmsr
    or      $(1 << 11 | 1 << 8), %eax
    wrmsr

    /* Enable CR0.PG */
    mov     %cr0, %eax
    or      $(1 << 31), %eax
    mov     %eax, %cr0

    .byte   0x66, 0xea # ljmp opcode
.Llm_entry:
    .long   0          # offset
    .word   (2 << 3)   # segment

.code64
long_mode:
    /*
     * At this point we're running in longmode, but still with identity pages
     * and at whatever physical address we were placed by the AP bootstrap
     * setup. From here we jump into high virtual memory, so we can start
     * running in kernel space.
     */
    movabs  $transition, %rax
    jmp     *%rax

.text
.extern     boot_ap
transition:
    /* 
     * Before clearing %ds, use it to find the data provided by the bootstrap
     * setup.
     */
    mov     %ds, %esi
    shl     $4, %esi

    /* Clear segment registers. */
    xorl    %eax, %eax
    movl    %eax, %ds
    movl    %eax, %ss
    movl    %eax, %es
    movl    %eax, %gs
    movl    %eax, %fs

    /* Switch to the proper kernel space. */
    mov    BOOTSTRAP_KERNEL_TOP_OFFSET(%rsi), %rax
    mov    %rax, %cr3

    /* Use per-cpu stack. */
    mov     BOOTSTRAP_STACK_OFFSET(%rsi), %rdi
    mov     %rdi, %rsp
    mov     %rdi, %rbp

    /* Pass stack- and per-CPU offsets to boot_ap. */
    mov     BOOTSTRAP_PERCPU_OFFSET(%rsi), %rsi

    call    boot_ap

    /* boot_ap is never supposed to return, so just hang. */
1:
    hlt
    jmp     1b
