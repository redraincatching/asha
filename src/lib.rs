//! # asha - A library for RISC-V disassembly and decompilation
//! 
//! this was made for a final year project in the University of Galway
//! hopefully this gets changed to say something useful eventually 
//! TODO: change this layout to be similar to egui's, it's a good example

mod app;
pub use app::AshaApp;

use object::{Object, ObjectSection};
use std::error::Error;
use std::fs;

#[macro_use]
mod instructions;
mod disassembly;

/// Read in an executable file and return it as bytes
pub fn read_compiled(filepath: &str) -> Vec<u8> {
    fs::read(filepath).expect("error reading object file")

    // TODO: error handling
    // TODO: separate them in a useful way
    // maybe look for useful sections
}

/// read in a file and display the name of each section
pub fn sections(binary_data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let file = object::File::parse(&*binary_data)?;
    for section in file.sections() {
        println!("{}", section.name()?);
    }
    Ok(())
}

/// read in a file and display the symbol table
pub fn symbols(binary_data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let file = object::File::parse(&*binary_data)?;
    for symbol in file.symbols() {
        println!("{:?}", symbol);
    }
    Ok(())
}

/// Output the raw bytes as 4-byte hex words, the address of the current 32-bit word, and the disassembled instructions
pub fn output_assembly(bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let file = object::File::parse(&*bytes)?;

    // find the .text section
    let text = file.sections()
        .find(|s| s.name() == Ok(".text"))
        .ok_or("no .text section found")?;

    // TODO: labels

    println!("----- dissassembly of .text section -----");
    let mut address : u64 = text.address();
    
    for row in text.data()?.chunks_exact(4) {
        // TODO: pretty-print the addresses
        print!("  {:>#8x}: ", address);
        address += 4;

        let raw = u32::from_le_bytes([row[0], row[1], row[2], row[3]]);
        // TODO: print bigendian with leading zeroes
        print!("{:0>8x}", raw);
        
        if let Some(instruction) = disassembly::disassemble(raw) {
            println!("    {}", instruction);
        } else {
            println!()
        }
    }
    Ok(())
}
