//! # asha - A library for RISC-V disassembly and decompilation
//! 
//! this was made for a final year project in the University of Galway
//! hopefully this gets changed to say something useful eventually 
//! TODO: change this layout to be similar to egui's, it's a good example

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

use object::{Object, ObjectSection};

#[macro_use]
mod instructions;
mod disassembly;
mod decompilation;
mod app;

pub fn launch_app() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(app::AshaApp::new(cc)))),
    )
}

/// Read in an executable file and return it as bytes
pub fn read_compiled(filepath: &str) -> Vec<u8> {
    fs::read(filepath).expect("error reading object file")

    // TODO: error handling
    // TODO: separate them in a useful way
}

pub fn disassemble_file(bytes: Vec<u8>) -> Result<BTreeMap<u64, instructions::InstructionType>, Box<dyn Error>> {
    let file = object::File::parse(&*bytes)?;
    let mut out = BTreeMap::new();

    // find the .text section
    let text = file.sections()
        .find(|s| s.name() == Ok(".text"))
        .ok_or("no .text section found")?;

    let mut address : u64 = text.address();
    
    for row in text.data()?.chunks_exact(4) {
        let raw = u32::from_le_bytes([row[0], row[1], row[2], row[3]]);
        
        if let Some(instruction) = disassembly::disassemble(raw) {
            out.insert(address, instruction);
        }

        address += 4;
    }
    Ok(out)
}

/// Output the raw bytes as 4-byte hex words, the address of the current 32-bit word, and the disassembled instructions
// TODO: refactor this to take a vector disassembled instructions
pub fn output_assembly(bytes: Vec<u8>) -> Result<String, Box<dyn Error>> {
    let file = object::File::parse(&*bytes)?;
    let mut out = String::new();

    // find the .text section
    let text = file.sections()
        .find(|s| s.name() == Ok(".text"))
        .ok_or("no .text section found")?;

    // TODO: labels

    out.push_str("----- dissassembly of .text section -----\n");
    let mut address : u64 = text.address();
    
    for row in text.data()?.chunks_exact(4) {
        // TODO: pretty-print the addresses
        out.push_str(&format!("  {:>#8x}: ", address));
        address += 4;

        let raw = u32::from_le_bytes([row[0], row[1], row[2], row[3]]);
        // TODO: print bigendian with leading zeroes
        out.push_str(&format!("{:0>8x}", raw));
        
        if let Some(instruction) = disassembly::disassemble(raw) {
            out.push_str(&format!("    {}\n", instruction));
        } else {
            out.push('\n');
        }
    }
    Ok(out)
}

// TODO: tests
