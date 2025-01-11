use std::fmt::Debug;

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
#[derive(Debug, PartialEq)]
enum InstructionType {
    R{opcode: u8, rd: u8, funct3: u8, rs1: u8, rs2: u8, funct7: u8},
    I{opcode: u8, rd: u8, funct3: u8, rs1: u8, imm: u16},            // may need a shifted variant
    S{opcode: u8, imm: u16, funct3: u8, rs1: u8, rs2: u8},
    B{opcode: u8, imm: u16, funct3: u8, rs1: u8, rs2: u8},
    U{opcode: u8, rd: u8, imm: u32},
    J{opcode: u8, rd: u8, imm: u32},
    Unknown
}

/// alias u32 to instruction
type Instruction = u32;

/// # instruction decoding
/// the process is as follows:
/// - check that the function is not all 1 or 0 and therefore invalid
/// - take in the 32-bit instruction
/// - check the bits [6:2] of the opcode with a mask
///     - if the instruction can be determined from here, return as InstructionType U or J
/// - check the bits [14:12] for funct3 with a mask
///     - if the instruction can be determined, return as InstructionType I, S, or B
/// - the function is either an R type or invalid
fn disassemble(instruction: Instruction) -> Option<InstructionType> {
    // check if the instruction is valid
    if (instruction == 0) || (instruction == 0xFFFFFFFF) {
        return None
    } 

    // get the required bits to check the opcode
    // if it isn't a known code, return None
    // TODO: TESTING! this may get messed up by endianness
    let opcode: u8 = retrieve!(opcode instruction).try_into().unwrap();
    // TODO: extend this table to cover everything I'm supporting
    // maybe move the comparison into another function and match on the result of that would be cleaner
    // use the lookup table
    let result = match opcode {
        0b01100 | 0b01110 => {
            InstructionType::R { 
                opcode, 
                rd: retrieve!(rd instruction).try_into().unwrap(),
                funct3: retrieve!(funct3 instruction).try_into().unwrap(), 
                rs1: retrieve!(rs1 instruction).try_into().unwrap(), 
                rs2: retrieve!(rs2 instruction).try_into().unwrap(), 
                funct7: retrieve!(funct7 instruction).try_into().unwrap()
            }
        },
        0b00101 | 0b001101 => {
            InstructionType::U { 
                opcode, 
                rd: retrieve!(rd instruction).try_into().unwrap(), 
                imm: retrieve!(uimm instruction)
            }
        },
        0b00100 | 0b00000 => {
            InstructionType::I { 
                opcode, 
                rd: retrieve!(rd instruction).try_into().unwrap(), 
                funct3: retrieve!(funct3 instruction).try_into().unwrap(), 
                rs1: retrieve!(rs1 instruction).try_into().unwrap(), 
                imm: retrieve!(iimm instruction).try_into().unwrap()
            }
        },
        0b11000 => {
            InstructionType::B { 
                opcode, 
                imm: retrieve!(bimm instruction).try_into().unwrap(),
                funct3: retrieve!(funct3 instruction).try_into().unwrap(), 
                rs1: retrieve!(rs1 instruction).try_into().unwrap(), 
                rs2: retrieve!(rs2 instruction).try_into().unwrap()
            }
        },
        0b01000 => {
            InstructionType::S { 
                opcode, 
                imm: retrieve!(simm instruction).try_into().unwrap(),
                funct3: retrieve!(funct3 instruction).try_into().unwrap(), 
                rs1: retrieve!(rs1 instruction).try_into().unwrap(), 
                rs2: retrieve!(rs2 instruction).try_into().unwrap()
            }
        },
        0b11011 => {
            InstructionType::J { 
                opcode, 
                rd: retrieve!(rd instruction).try_into().unwrap(), 
                imm: retrieve!(jimm instruction)
            }
        },
        _ => {
            InstructionType::Unknown
        }
    };

    //let opcode_decoded: Opcode = opcode.into();
    //if opcode_decoded == Opcode::Unknown { return None }
   
    // here we return the raw bits of the instruction, sorted into fields
    // we can then, in another function, determine what instruction it is
    // and then print out

    // maybe use the tooling that generates the matches automatically?
    // i feel like i could write a procedural macro for this

    Some(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decoding() {
        let r_type = 0x4000503b;  // sraw
        let u_type = 0x00000037;  // lui
        let i_type = 0x00002003;  // lw
        let b_type = 0x00000063;  // beq
        let s_type = 0x4000503b;  // sraw
        let j_type = 0x00003023;  // sd

        assert_eq!(
            disassemble(r_type), 
            Some(InstructionType::R { 
                opcode: 0b01110, 
                rd: 0b00000, 
                funct3: 0b101, 
                rs1: 0b0 ,
                rs2: 0b0, 
                funct7: 0b0100000 
            }));
    }
}
