# Whiskey Kernel
![Rust Badge](https://img.shields.io/badge/Language-Rust-orange)
![GPL Badge](https://img.shields.io/badge/License-GPL_3.0-blue)

<br>


<img src="https://images2.imgbox.com/33/c8/8EtZjavM_o.png" alt="whiskey chan" style="vertical-align: middle; margin-right: 10px;" width="250">
<span>I approve of this project. - An alcoholic anime girl I drew in five minutes.</span>

#### Whiskey is a RISC-V Kernel built in Rust and Assembly.
#### It features literally nothing useful so far other than a cool yellow shell.

whiskey_kernel is a heavy work in progress, and aims to develop into a skeleton that can be built
upon by other fellow developers.

I guess you could call it GNU/whiskey. Yes, that was a funny joke.

<br>

#### Roadmap
1 -> Finish buddy allocator (DONE)

2 -> define Userspace ABI (primarily a tiny syscall interface) (IN PROGRESS)

3 -> memory isolation, simple page table setup

4 -> replace tasks.rs with better process structure

5 -> storage support (read from a disk image)

5.5 -> simple read-only filesystem (w/dierctory table, some files)

6 -> add an elf loader to parse simple elf binaries

7 -> add syscall layer for writing, exiting (+ reading later)

<br>


#### Does this project use AI?
Yes and no. kernelDecrypt (Head developer) makes use of AI as advisory, and primarily to explain 
RISC-V assembly code. For other purposes, kernelDecrypt frowns upon it. This is for learning.

<br>
<br>

## Prerequisites 
#### nightly Rust toolchain with the riscv64gc-unknown-none-elf target
#### rust-src component installed
#### RISC-V emulator qemu-system-riscv64

## Quick start
#### make to build
#### make run to launch QEMU

<br>
<br>

## LICENSE


### This project is licensed under the GNU General Public License v3.0 or later
