# `asha` - a RISC-V decompiler written in Rust

[![License](https://img.shields.io/badge/License-MIT_Non--AI-blue)](#license)

`asha` is a decompiler that for RISC-V written for a final year project. 
A full writeup will be coming soon.

## Current RISC-V extension support 
- RV32/64I
- RV32/64M
- RV32/64A

----

## Output Examples
using an implementation of the fibonacci function as an example

### Disassembly
![](https://github.com/redraincatching/asha/raw/main/images/disasm_view.png)

### Control Flow Graph
![](https://github.com/redraincatching/asha/raw/main/images/cfg_view.png)

### Decompilation
![](https://github.com/redraincatching/asha/raw/main/images/decomp_view.png)

----

## Running the Program
Build the program with `cargo`
```
> cargo build --release
```
and run the executable, or simply run with cargo
```
> cargo run --release
```

----

###### License
Released under [MIT Non-AI](/license) by [@redraincatching](https://github.com/redraincatching).

----
![Neovim](https://img.shields.io/badge/NeoVim-%2357A143.svg?&style=for-the-badge&logo=neovim&logoColor=white)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
