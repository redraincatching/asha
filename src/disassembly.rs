use crate::instructions::*;

/// # instruction decoding
/// TODO: explain this
pub fn disassemble(instruction: Instruction) -> Option<InstructionType> {
    // check if the instruction is valid
    if (instruction == 0) || (instruction == 0xFFFFFFFF) || (instruction & 0b11 != 0b11) {
        return None
    } 

    // determine which function it is
    if let Some((name, i_type)) = determine_name(&instruction) {
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
                imm: convert_to_signed(retrieve!(bimm instruction) as usize, 12) as i16
            }),
            IT::U => Some(InstructionType::U { 
                name, 
                rd: ABIRegister::from(retrieve!(rd instruction) as u8),
                imm: retrieve!(uimm instruction)
            }),
            IT::J => Some(InstructionType::J { 
                name, 
                rd: ABIRegister::from(retrieve!(rd instruction) as u8),
                imm: convert_to_signed(retrieve!(jimm instruction) as usize, 20) as i32
            })
            // TODO: extend for R4
        }        
    } else { 
        None 
    }
}

/// # Determine the function name
/// TODO: better documentation for all of these, honestly
/// TODO: account for R4 instructions here too
fn determine_name(instruction: &Instruction) -> Option<(&'static str, IT)> {
    // this function was revealed to me in a dream

    let opcode: u8 = retrieve!(opcode instruction).try_into().unwrap();
    let i_type = determine_type(opcode)?;

    let funct3: u8;
    let funct7: u8;

    // U and J-type only use opcode
    if !(i_type == IT::U || i_type == IT::J) {
        funct3 = retrieve!(funct3 instruction).try_into().unwrap();

        // only R uses funct7 (and also _one_ singular i-type)
        if i_type == IT::R || opcode == 0b00100 {
            funct7 = retrieve!(funct7 instruction).try_into().unwrap();
        } else {
            funct7 = 0;
        }

    } else {
        funct3 = 0;
        funct7 = 0;
    }

    // use the from_bits to convert
    from_bits(opcode, funct3, funct7).map(|name| (name, i_type))
}

/// Determine the type of instruction, and therefore which fields to match on
fn determine_type(opcode: u8) -> Option<IT> {
    let bf = OpcodeBitfield::from_opcode(opcode);
    
    if (bf.op4 && bf.op2) || (bf.op3 && bf.op2 && !bf.op0) {
        Some(IT::R)
    } else if (!bf.op4 && !bf.op3 && !bf.op2) || (!bf.op4 && !bf.op3 && !bf.op0) || (bf.op4 && bf.op3 && !bf.op1 && bf.op0) {
        Some(IT::I)
    } else if !bf.op4 && bf.op3 && !bf.op2 {
        Some(IT::S)
    } else if bf.op4 && bf.op3 && !bf.op0 {
        Some(IT::B)
    } else if bf.op2 && bf.op0 {
        Some(IT::U)
    } else if bf.op3 && !bf.op2 && bf.op1 {
        Some(IT::J)
    //} else if bf.op4 && !bf.op3 && !bf.op2 {
    //    Some(IT::R4)
    } else {
        None
    }
}

/// Convert from bit fields to instruction
fn from_bits(opcode: u8, funct3: u8, funct7: u8) -> Option<&'static str> {
    // convert to array so that the phf map can use it as a key
    let key: [u8; 3] = [opcode, funct3, funct7];
    INSTRUCTIONS.get(&key).cloned()
}

/// Convert from two's complement raw bits to isize
/// takes the number of bits operating on, as this is always an unusual amount, and is sign extended
fn convert_to_signed(value: usize, bits: usize) -> isize {
    // calculate the maximum value from the number of bits
    let max = (1_usize << (bits - 1)) - 1;

    // use a bitmask to extract only the relevant bits
    let mask = (1_usize << bits) - 1;
    let value = value & mask;

    if value <= max {
        // positive value, return unchanged
        value as isize
    } else {
        // negative, return as such
        (value as isize) - (1_usize << bits) as isize
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn twos_complement_conversion() {
        // i don't like the lack of an apostrophe here but whatever
        let value: usize = 0x7FF; // 12-bit max
        let result = convert_to_signed(value, 12);
        assert_eq!(result, 2047);

        let value: usize = 0x800; // 12-bit min
        let result = convert_to_signed(value, 12);
        assert_eq!(result, -2048);

        let value: usize = 0x7FFFF; // 20-bit max
        let result = convert_to_signed(value, 20);
        assert_eq!(result, 524287);

        let value: usize = 0x80000; // 20-bit min
        let result = convert_to_signed(value, 20);
        assert_eq!(result, -524288);

        let value: usize = 0; // zero value for 12 bits
        let result = convert_to_signed(value, 12);
        assert_eq!(result, 0);

        let value: usize = 0; // zero value for 20 bits
        let result = convert_to_signed(value, 20);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_decoding() {
        let r_type = 0x40c5d53b;  // sraw a0, a1, a2
        let u_type = 0x076192b7;  // lui t0, 30233
        let i_type = 0x05002083;  // lw ra, 80
        let b_type = 0x00928263;  // beq t0, s1, 4
        let s_type = 0x01103523;  // sd a7, 10
        let j_type = 0xfb5ff16f;  // jal sp, -76

        assert_eq!(
            disassemble(r_type), 
            Some(InstructionType::R { 
                name: "sraw",
                rd: ABIRegister::a0, 
                rs1: ABIRegister::a1,
                rs2: ABIRegister::a2 
            })
        );

        assert_eq!(
            disassemble(u_type), 
            Some(InstructionType::U { 
                name: "lui",
                rd: ABIRegister::t0,
                imm: 30233 
            })
        );

        assert_eq!(
            disassemble(i_type),
            Some(InstructionType::I { 
                name: "lw", 
                rd: ABIRegister::ra,
                rs1: ABIRegister::zero, 
                imm: 80 
            })
        );

        assert_eq!(
            disassemble(b_type),
            Some(InstructionType::B { 
                name: "beq",
                rs1: ABIRegister::t0,
                rs2: ABIRegister::s1,
                imm: 4 
            })
        );

        assert_eq!(
            disassemble(s_type),
            Some(InstructionType::S { 
                name: "sd",
                rs1: ABIRegister::zero,
                rs2: ABIRegister::a7,
                imm: 10 
            })
        );

        assert_eq!(
            disassemble(j_type),
            Some(InstructionType::J { 
                name: "jal",
                rd: ABIRegister::sp,
                imm: -76 
            })
        );
    }
}
