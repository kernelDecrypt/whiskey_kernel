.section .text.entry
.global _start

_start:
    # STACK POINTER
    # QEMU's VIRT puts RAM starting at 0x80000000.
    la sp, boot_stack_top
    csrw mscratch, x0

    #  machine-mode trap vector
    la t0, trap_handler
    csrw mtvec, t0


    # delegate exceptions &/r interrupts to Supervisor mode
    li t0, 0xffff
    csrw medeleg, t0        # hands off illegal instructions, page faults, ecall fromU etc
    li t0, 0x222
    csrw mideleg, t0        # bits cant be delegated

    # set up Supervisor mode own interrupt enable bits aot
    li t0, 0x222
    csrw sie, t0

    # set mpp to supervisor so mret drops us there
    li t0, 0x1800           # mask for mpp
    csrc mstatus, t0
    li t0, 0x800            # 01 << 11
    csrs mstatus, t0

    # tell mret where to land
    la t0, s_mode_entry
    csrw mepc, t0

    # Grant S-mode (and U-mode) full access to all of physical memory
    li t0, 0x3fffffffffffff   # pmpaddr0: top of address range (TOR mode)
    csrw pmpaddr0, t0
    li t0, 0x0f                # pmpcfg0: R=1 W=1 X=1, A=TOR(01)
    csrw pmpcfg0, t0

    li t0, 0x80          # MTIE = bit 7
    csrs mie, t0

    mret # isolation boundary
    # we dont tail rust main directly it jumps to a new symbol see s_entry.s

    # 4. Jump to function
    tail rust_main

.section .bss.stack
.global boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 4  # 16KB stack space
.global boot_stack_top
boot_stack_top:

.section .bss.trap_stack
.global trap_stack_lower_bound
trap_stack_lower_bound:
    .space 4096 * 4  # 16KB trap stack
.global trap_stack_top
trap_stack_top:

.section .bss.strap_stack
.global strap_stack_lower_bound
strap_stack_lower_bound:
    .space 4096 * 4  # 16KB S-mode trap stack
.global strap_stack_top
strap_stack_top: