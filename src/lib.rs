//! # asha - A library for RISC-V disassembly and decompilation
//! 
//! this was made for a final year project in the University of Galway
//! hopefully this gets changed to say something useful eventually 

use object::{Object, ObjectSection};
use std::error::Error;
use std::fs;

mod opcodes;
mod instructions;

/// Read in an executable file and return it as bytes
pub fn read_compiled(filepath: &str) -> Vec<u8> {
    // could get this to return an iterator?
    // TODO: error handling
    let bytes = fs::read(filepath).expect("error reading object file");

    // TODO: separate them in a useful way
    // maybe look for useful sections
    bytes
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
    let mut address : u64 = 0;
    let file = object::File::parse(&*bytes)?;
    
    for row in bytes.chunks_exact(4) {
        // TODO: pretty-print the addresses
        print!("{:<#10x}", address);
        address += 4;

        for byte in row.iter() {
            print!("{:0>2x}", byte);
        }

        println!("    opcode goes here");
    }
    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
