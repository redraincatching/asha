use std::{fmt::Debug, ops::Deref};

/// # RISC-V Instruction Types
///
/// Instructions in RISC-V are a fixed 32-bit width, and must be aligned to 4-byte word boundaries
/// Immediates in the instructions are sign-extended from bit 31
///
/// Each instruction takes one of six forms:
/// - Register to register instructions (R type)
/// - Immediate instructions (I type)
/// - Store instructions (S type)
/// - Branch instructions (B type)
/// - Upper immediate instructions (U type)
/// - Jump instructions (J type)
///
/// These instructions formats have a different layout of fields, with each field being treated as its own unsigned integer
/// Each instruction encoding keeps the opcode, destination register, and first source register in the same place (if they exist)
/// It should be noted that the only difference between the S and B formats is that the 12-bit immediate field is used to encode branch offsets in multiples of 2 in the B format. Similarly, the only difference between the U and J formats is that the 20-bit immediate is shifted left by 12 bits to form U immediates, and by 1 to form J immediates.
/// NOTE: the zicsr and zifencei sets are also considered standard, may add them

/// # Instruction types
/// The fields are as follows: (not to scale)
/// | funct7                | rs2 | rs1 | funct3 | rd                   | opcode | R type |
/// | imm[11:0]                   | rs1 | funct3 | rd                   | opcode | I type |
/// | imm[11:5]             | rs2 | rs1 | funct3 | imm[4:1]             | opcode | S type |
/// | imm[12] | imm[10:5]   | rs2 | rs1 | funct3 | imm[4:1] | imm[11]   | opcode | B type |
/// | imm[31:12]                                 | rd                   | opcode | U type |
/// | imm[20] | imm[10:1] | imm[11] | imm[9:12]  | rd                   | opcode | J type |
///
/// the opcode, funct3, and funct7 bits combine to determine the function used
/// rd is a destination register, rs* are source registers
/// imm is an immediate value
/// in addition, inst[1:0] are 11 for all valid instructions. all 0s and all 1s are both invalid
/// MAYBE: use an enum that can take u8 or Register for the register fields
#[derive(Debug, PartialEq)]
enum InstructionType {
    R{name: Option<String>, opcode: u8, rd: u8, funct3: u8, rs1: u8, rs2: u8, funct7: u8},
    I{name: Option<String>, opcode: u8, rd: u8, funct3: u8, rs1: u8, imm: u16},            // may need a shifted variant
    S{name: Option<String>, opcode: u8, imm: u16, funct3: u8, rs1: u8, rs2: u8},
    B{name: Option<String>, opcode: u8, imm: u16, funct3: u8, rs1: u8, rs2: u8},
    U{name: Option<String>, opcode: u8, rd: u8, imm: u32},
    J{name: Option<String>, opcode: u8, rd: u8, imm: u32}
}

/// simpler representation of the instruction types
enum IT {
    R, I, S, B, U, J
}

/// alias u32 to instruction
type Instruction = u32;

/// # Determine type of instruction
/// Determine what kind of instruction we are attempting to decode.
/// Based on opcodes supported
/// TODO: throw this into logisim or verilog and simplify
/// TODO: finish this, honestly
fn determine_instruction_type(opcode: u8) -> Option<IT> {
    match opcode {
        0b01100 | 0b01110 
            => { Some(IT::R) },
        0b00100 | 0b00000
            => { Some(IT::I) },
        0b01000
            => { Some(IT::S) },
        0b11000
            => { Some(IT::B) },
        0b00101 | 0b001101
            => { Some(IT::U) },
        0b11011
            => { Some(IT::J) },
        _ => { None }
    }
}

/// make setting the name of an instruction cleaner
macro_rules! set_name {
    ($inst:expr, $val:expr) => (
        match $inst {
            $crate::instructions::InstructionType::R { name, .. }
            | $crate::instructions::InstructionType::I { name, .. }
            | $crate::instructions::InstructionType::S { name, .. }
            | $crate::instructions::InstructionType::B { name, .. }
            | $crate::instructions::InstructionType::U { name, .. }
            | $crate::instructions::InstructionType::J { name, .. } => {
                *name = Some($val.to_string());
            }
        }
    )
}

/// # Determine the function name
/// TODO: better documentation for all of these, honestly
/// TODO: account for R4 instructions here too, could just do it in the table with correct masks?
fn determine_name(instruction: &mut InstructionType) {
    // extract fields relevant for matching
    let (opcode, funct3, funct7) = match instruction {
        InstructionType::R { opcode, funct3, funct7, .. } => (
            *opcode,
            Some(*funct3),
            Some(*funct7)
        ),
        InstructionType::I { opcode, funct3, .. }
        | InstructionType::S { opcode, funct3, .. }
        | InstructionType::B { opcode, funct3, .. } => (
            *opcode,
            Some(*funct3),
            None
        ),
        InstructionType::U { opcode, .. }
        | InstructionType::J { opcode, .. } => (
            *opcode,
            None,
            None
        )
    };

    // TODO: actually match based on an enum
    // for now, though
    set_name!(instruction, "addi");

    // what i actually want to do here is combine the three fields into a single binary number
    // and get which one it is with a jump table
    // something like (funct7.unwrap_or(0) << 8 | funct3.unwrap_or(0) << 5 | opcode as u32)
}

/// # instruction decoding
/// TODO: explain this
/// TODO: maybe change the output type, would a string slice work?
/// Or maybe extend to use optional names and other fields
/// might need InstructionType to have a constructor in that case
fn disassemble(instruction: Instruction) -> Option<InstructionType> {
    // check if the instruction is valid
    if (instruction == 0) || (instruction == 0xFFFFFFFF) || (instruction & 0b11 != 0b11) {
        return None
    } 

    // get the required bits to check the opcode
    // if it isn't a known code, return None
    // TODO: TESTING! this may get messed up by endianness or my other mistakes
    let opcode: u8 = retrieve!(opcode instruction).try_into().unwrap();

    // determine instruction type
    let instruction_type = determine_instruction_type(opcode);

    // TODO: extend this table to cover everything I'm supporting
    // maybe move the comparison into another function and match on the result of that would be cleaner
    // use the lookup table
    // TODO: switch this up so that the registers are of type register
    let mut result = match instruction_type {
        Some(IT::R) => {
            Some(InstructionType::R { 
                name: None,
                opcode, 
                rd: retrieve!(rd instruction).try_into().unwrap(),
                funct3: retrieve!(funct3 instruction).try_into().unwrap(), 
                rs1: retrieve!(rs1 instruction).try_into().unwrap(), 
                rs2: retrieve!(rs2 instruction).try_into().unwrap(), 
                funct7: retrieve!(funct7 instruction).try_into().unwrap()
            })
        },
        Some(IT::U) => {
            Some(InstructionType::U { 
                name: None,
                opcode, 
                rd: retrieve!(rd instruction).try_into().unwrap(), 
                imm: retrieve!(uimm instruction)
            })
        },
        Some(IT::I) => {
            Some(InstructionType::I { 
                name: None,
                opcode, 
                rd: retrieve!(rd instruction).try_into().unwrap(), 
                funct3: retrieve!(funct3 instruction).try_into().unwrap(), 
                rs1: retrieve!(rs1 instruction).try_into().unwrap(), 
                imm: retrieve!(iimm instruction).try_into().unwrap()
            })
        },
        Some(IT::B) => {
            Some(InstructionType::B { 
                name: None,
                opcode, 
                imm: retrieve!(bimm instruction).try_into().unwrap(),
                funct3: retrieve!(funct3 instruction).try_into().unwrap(), 
                rs1: retrieve!(rs1 instruction).try_into().unwrap(), 
                rs2: retrieve!(rs2 instruction).try_into().unwrap()
            })
        },
        Some(IT::S) => {
            Some(InstructionType::S { 
                name: None,
                opcode, 
                imm: retrieve!(simm instruction).try_into().unwrap(),
                funct3: retrieve!(funct3 instruction).try_into().unwrap(), 
                rs1: retrieve!(rs1 instruction).try_into().unwrap(), 
                rs2: retrieve!(rs2 instruction).try_into().unwrap()
            })
        },
        Some(IT::J) => {
            Some(InstructionType::J { 
                name: None,
                opcode, 
                rd: retrieve!(rd instruction).try_into().unwrap(), 
                imm: retrieve!(jimm instruction)
            })
        },
        _ => {
            return None
        }
    };

    // now here we determine what function it actually is
    determine_name(result.as_mut()?);

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decoding() {
        // TODO: replace these with ones with nontrivial arguments
        let r_type = 0x4000503b;  // sraw
        let u_type = 0x00000037;  // lui
        let i_type = 0x00002003;  // lw
        let b_type = 0x00000063;  // beq
        let s_type = 0x4000503b;  // sraw
        let j_type = 0x00003023;  // sd

        assert_eq!(
            disassemble(r_type), 
            Some(InstructionType::R { 
                name: Some("sraw".to_string()),
                opcode: 0b01110, 
                rd: 0b00000, 
                funct3: 0b101, 
                rs1: 0b0 ,
                rs2: 0b0, 
                funct7: 0b0100000 
            }));
    }
}
