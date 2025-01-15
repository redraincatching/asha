use crate::instructions::*;

/// # instruction decoding
/// TODO: explain this
fn disassemble(instruction: Instruction) -> Option<InstructionType> {
    // check if the instruction is valid
    if (instruction == 0) || (instruction == 0xFFFFFFFF) || (instruction & 0b11 != 0b11) {
        return None
    } 

    // determine which function it is
    if let Some((name, i_type)) = determine_name(&instruction) {
        // TODO: match on instruction type and change register operands using the enum

        match i_type {
            IT::R => Some(InstructionType::R { 
                name, 
                rd: ABIRegister::from(retrieve!(rd instruction) as u8),
                rs1: ABIRegister::from(retrieve!(rs1 instruction) as u8),
                rs2: ABIRegister::from(retrieve!(rs2 instruction) as u8) 
            }),
            IT::I => Some(InstructionType::I { 
                name, 
                rd: ABIRegister::from(retrieve!(rd instruction) as u8),
                rs1: ABIRegister::from(retrieve!(rs1 instruction) as u8),
                imm: retrieve!(iimm instruction) as u16
            }),
            IT::S => Some(InstructionType::S { 
                name, 
                rs1: ABIRegister::from(retrieve!(rs1 instruction) as u8),
                rs2: ABIRegister::from(retrieve!(rs2 instruction) as u8),
                imm: retrieve!(simm instruction) as u16
            }),
            IT::B => Some(InstructionType::B { 
                name, 
                rs1: ABIRegister::from(retrieve!(rs1 instruction) as u8),
                rs2: ABIRegister::from(retrieve!(rs2 instruction) as u8),
                imm: retrieve!(bimm instruction) as u16
            }),
            IT::U => Some(InstructionType::U { 
                name, 
                rd: ABIRegister::from(retrieve!(rd instruction) as u8),
                imm: retrieve!(uimm instruction)
            }),
            IT::J => Some(InstructionType::J { 
                name, 
                rd: ABIRegister::from(retrieve!(rd instruction) as u8),
                imm: retrieve!(jimm instruction)
            })
        }        
    } else { 
        None 
    }
}

/// # Determine the function name
/// TODO: better documentation for all of these, honestly
/// TODO: account for R4 instructions here too, could just do it in the table with correct masks?
fn determine_name(instruction: &Instruction) -> Option<(String, IT)> {
    // this function was revealed to me in a dream

    // what i actually want to do here is combine the three fields into a single binary number
    // and get which one it is with a jump table
    // something like (funct7.unwrap_or(0) << 8 | funct3.unwrap_or(0) << 5 | opcode as u32)
    let opcode: u8 = retrieve!(opcode instruction).try_into().unwrap();
    let funct3: u8 = retrieve!(funct3 instruction).try_into().unwrap();
    let funct7: u8 = retrieve!(funct7 instruction).try_into().unwrap();

    // use the from_bits to convert
    if let Some((name, i_type)) = from_bits(opcode, funct3, funct7) {
        Some((name, i_type))
    } else {
        None
    } 
}

/// Convert from bit fields to instruction
/// TODO: swap this from a match statement to a phf, see https://github.com/rust-phf/rust-phf
/// also of note, https://github.com/abonander/mime_guess/pull/14/files
fn from_bits(opcode: u8, funct3: u8, funct7: u8) -> Option<(String, IT)> {
    // convert to single value 
    match (opcode, funct3, funct7) {
        // RV32I
        (0b01100, 0b000, 0b0000000) => Some(("add".to_string(), IT::R)),
        _ => None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decoding() {
        // TODO: replace these with ones with nontrivial operands
        let r_type = 0x4000503b;  // sraw
        let u_type = 0x00000037;  // lui
        let i_type = 0x00002003;  // lw
        let b_type = 0x00000063;  // beq
        let s_type = 0x4000503b;  // sraw
        let j_type = 0x00003023;  // sd

        assert_eq!(
            disassemble(r_type), 
            // TODO: test versions with ABI and raw identifiers?
            // or maybe not
            Some(InstructionType::R { 
                name: "sraw".to_string(),
                rd: ABIRegister::zero, 
                rs1: ABIRegister::zero,
                rs2: ABIRegister::zero 
            }));
    }
}
