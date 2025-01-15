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

/// alias u32 to instruction
pub type Instruction = u32;

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
/// TODO: note R4 instructions
#[derive(Debug, PartialEq)]
pub enum InstructionType {
    R{name: String, rd: ABIRegister, rs1: ABIRegister, rs2: ABIRegister},
    I{name: String, rd: ABIRegister, rs1: ABIRegister,                   imm: u16},
    S{name: String,                  rs1: ABIRegister, rs2: ABIRegister, imm: u16},
    B{name: String,                  rs1: ABIRegister, rs2: ABIRegister, imm: u16},
    U{name: String, rd: ABIRegister,                                     imm: u32},
    J{name: String, rd: ABIRegister,                                     imm: u32}
}

/// InstructionType identifier enum
pub enum IT {
    R, I, S, B, U, J
}

// TODO: add lifting to pseudoinstructions

/// Enum to translate registers from binary value to ABI name
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum ABIRegister {
    zero,   // hardcoded zero
    ra,     // return address
    sp,     // stack pointer
    gp,     // global pointer
    tp,     // thread pointer
    t0,     // temporary registers
    t1,
    t2,
    s0,     // saved registers
    s1,
    a0,     // argument registers
    a1,
    a2,
    a3,
    a4,
    a5,
    a6,
    a7,
    s2,     // saved registers
    s3,
    s4,
    s5,
    s6,
    s7,
    s8,
    s9,
    s10,
    s11,
    t3,     // temporary registers
    t4,
    t5,
    t6,
    Unknown
}

impl From<u8> for ABIRegister {
    fn from(value: u8) -> Self {
        match value {
            0x0 =>  ABIRegister::zero,
            0x1 =>  ABIRegister::ra,
            0x2 =>  ABIRegister::sp,
            0x3 =>  ABIRegister::gp,
            0x4 =>  ABIRegister::tp,
            0x5 =>  ABIRegister::t0,
            0x6 =>  ABIRegister::t1,
            0x7 =>  ABIRegister::t2,
            0x8 =>  ABIRegister::s0,
            0x9 =>  ABIRegister::s1,
            0x10 => ABIRegister::a0,
            0x11 => ABIRegister::a1,
            0x12 => ABIRegister::a2,
            0x13 => ABIRegister::a3,
            0x14 => ABIRegister::a4,
            0x15 => ABIRegister::a5,
            0x16 => ABIRegister::a6,
            0x17 => ABIRegister::a7,
            0x18 => ABIRegister::s2,
            0x19 => ABIRegister::s3,
            0x20 => ABIRegister::s4,
            0x21 => ABIRegister::s5,
            0x22 => ABIRegister::s6,
            0x23 => ABIRegister::s7,
            0x24 => ABIRegister::s8,
            0x25 => ABIRegister::s9,
            0x26 => ABIRegister::s10,
            0x27 => ABIRegister::s11,
            0x28 => ABIRegister::t3,
            0x29 => ABIRegister::t4,
            0x30 => ABIRegister::t5,
            0x31 => ABIRegister::t6,
            _   => ABIRegister::Unknown
        }
    }
}

// -------------------------
//          Macros
// -------------------------

/// retrieve specified fields from raw instruction bytes
// NOTE: this isn't totally correct, I don't think
// need to write tests to make sure these all work, especially the immediates
// TODO: check that all of this is necessary, think it should be?
// TODO: see if i need anything else for R4
macro_rules! retrieve {
    (opcode $inst:expr) => {
        (($inst >> 2) & 0b11111)
    };
    (rd $inst:expr) => {
        (($inst >> 7) & 0b11111)
    };
    (funct3 $inst:expr) => {
        (($inst >> 12) & 0b111) 
    };
    (funct7 $inst:expr) => {
        (($inst >> 25) & 0b1111111) 
    };
    (rs1 $inst:expr) => {
        (($inst >> 15) & 0b11111) 
    };
    (rs2 $inst:expr) => {
        (($inst >> 20) & 0b11111) 
    };
    (simm $inst:expr) => {
        // 11 - 7:  imm[4:0]
        // 31 - 25: imm[11:5]
        (($inst >> 20) & 0b111111100000) + (($inst >> 7) & 0b11111)
    };
    (bimm $inst:expr) => {
        // note: imm[0] is implicitly 0
        // TEST: CHECK MY IMPLEMENTATION OF THIS AGAINST ANOTHER TOOL
        (($inst >> 19)  &   0b100000000000) + // 31:       imm[12]
        (($inst << 4)   &    0b10000000000) + // 7:        imm[11]
        (($inst >> 15)  &     0b1111110000) + // 25-30:    imm[10:5]
        (($inst >> 8)   &           0b1111)   // 8-11:     imm[4:1]
    };
    (iimm $inst:expr) => {
        (($inst >> 20) & 0b111111111111)
    };
    (uimm $inst:expr) => {
        (($inst >> 12) & 0b11111111111111111111)
    };
    (jimm $inst:expr) => {
        // note: imm[0] is implicitly 0
        // TEST: CHECK MY IMPLEMENTATION OF THIS AGAINST ANOTHER TOOL
        (($inst >> 11)  &   0b10000000000000000000) + // 31:       imm[20]
        ( $inst         &    0b1111111100000000000) + // 12-19:    imm[19:12]
        (($inst >> 9)   &            0b10000000000) + // 20:       imm[11]
        (($inst >> 21)  &             0b1111111111)   // 21-30:    imm[10:1]
    }
}

/// make setting the name of an instruction cleaner
/// TODO: think this isn't necessary, remove after checking
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fields() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_immediate() {
        assert_eq!(1, 1);
    }
}
