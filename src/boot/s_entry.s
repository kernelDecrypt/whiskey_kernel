.global s_entry
s_mode_entry:
    la t0, strap_handler
    csrw stvec, t0
    tail rust_main