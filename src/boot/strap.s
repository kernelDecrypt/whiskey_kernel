.global strap_handler
strap_handler:
    csrrw t6, sscratch, sp
    la sp, trap_stack_top
    addi sp, sp, -256

    sd ra, 248(sp)
    sd t0, 240(sp)
    sd t1, 232(sp)
    sd t2, 224(sp)
    sd t3, 216(sp)
    sd t4, 208(sp)
    sd t5, 200(sp)
    sd t6, 192(sp)
    sd a0, 184(sp)
    sd a1, 176(sp)
    sd a2, 168(sp)
    sd a3, 160(sp)
    sd a4, 152(sp)
    sd a5, 144(sp)
    sd a6, 136(sp)
    sd a7, 128(sp)
    sd s0, 120(sp)
    sd s1, 112(sp)
    sd s2, 104(sp)
    sd s3, 96(sp)
    sd s4, 88(sp)
    sd s5, 80(sp)
    sd s6, 72(sp)
    sd s7, 64(sp)
    sd s8, 56(sp)
    sd s9, 48(sp)
    sd s10, 40(sp)
    sd s11, 32(sp)
    sd gp, 24(sp)
    sd tp, 16(sp)

    csrr t0, scause
    csrr t1, sepc
    csrr t2, stval

    srli t4, t0, 63
    bnez t4, s_handle_interrupt

    andi t3, t0, 0x7ff
    li t4, 8
    beq t3, t4, s_exception_syscall   # ecall from Uuser mode
    j s_handle_exception

    s_handle_interrupt:
        andi t3, t0, 0x7ff
        li t4, 1
        beq t3, t4, s_software_interrupt   # SSIP timer tick
        li t4, 9
        beq t3, t4, s_external_interrupt   # SEIE uart irq
        j s_restore_and_return

    s_software_interrupt:
        li t3, 0x2
        csrc sip, t3            # clear ssip
        call handle_timer_interrupt   
        j s_restore_and_return

    s_external_interrupt:
        call handle_external_interrupt  
        j s_restore_and_return

    s_handle_exception:
        mv a0, t0
        mv a1, t1
        mv a2, t2
        addi a3, sp, 16
        call rust_exception_handler
        j s_restore_and_return

    s_exception_syscall:
        ld a0, 184(sp)
        ld a1, 176(sp)
        ld a2, 168(sp)
        ld a3, 160(sp)
        ld a4, 152(sp)
        mv a5, t1
        call rust_syscall_handler
        sd a0, 184(sp)
        addi t1, t1, 4
        csrw sepc, t1
        j s_restore_and_return

    s_restore_and_return:
        ld tp, 16(sp)
        ld gp, 24(sp)
        ld s11, 32(sp)
        ld s10, 40(sp)
        ld s9, 48(sp)
        ld s8, 56(sp)
        ld s7, 64(sp)
        ld s6, 72(sp)
        ld s5, 80(sp)
        ld s4, 88(sp)
        ld s3, 96(sp)
        ld s2, 104(sp)
        ld s1, 112(sp)
        ld s0, 120(sp)
        ld a7, 128(sp)
        ld a6, 136(sp)
        ld a5, 144(sp)
        ld a4, 152(sp)
        ld a3, 160(sp)
        ld a2, 168(sp)
        ld a1, 176(sp)
        ld a0, 184(sp)
        ld t6, 192(sp)
        ld t5, 200(sp)
        ld t4, 208(sp)
        ld t3, 216(sp)
        ld t2, 224(sp)
        ld t1, 232(sp)
        ld t0, 240(sp)
        ld ra, 248(sp)
        csrrw sp, sscratch, x0
        sret               # mret more like sret