use std::fmt::{self, Display};

use phf::phf_map;

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
/// TODO: note R4 instructions
#[derive(Debug, PartialEq, Clone)]
pub enum InstructionType {
    R{name: &'static str, rd: ABIRegister, rs1: ABIRegister, rs2: ABIRegister},
    I{name: &'static str, rd: ABIRegister, rs1: ABIRegister,                   imm: u16},
    S{name: &'static str,                  rs1: ABIRegister, rs2: ABIRegister, imm: u16},
    B{name: &'static str,                  rs1: ABIRegister, rs2: ABIRegister, imm: i16},
    U{name: &'static str, rd: ABIRegister,                                     imm: u32},
    J{name: &'static str, rd: ABIRegister,                                     imm: i32}
}

impl InstructionType {
    pub fn get_name(&self) -> &'static str {
        match *self {
            InstructionType::R {name, ..} | 
            InstructionType::I {name, ..} | 
            InstructionType::S {name, ..} | 
            InstructionType::B {name, ..} | 
            InstructionType::U {name, ..} | 
            InstructionType::J {name, ..}
                => name
        }
    }
}

impl fmt::Display for InstructionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InstructionType::R {name, rd, rs1, rs2} =>  write!(f, "{} {}, {}, {}", name, rd, rs1, rs2),
            InstructionType::I {name, rd, rs1, imm} =>  write!(f, "{} {}, {}, {}", name, rd, rs1, imm),
            InstructionType::S {name, rs1, rs2, imm} => write!(f, "{} {}, {}, {}", name, rs1, rs2, imm),
            InstructionType::B {name, rs1, rs2, imm} => write!(f, "{} {}, {}, {}", name, rs1, rs2, imm),
            InstructionType::U {name, rd, imm} =>       write!(f, "{} {}, {}", name, rd, imm),
            InstructionType::J {name, rd, imm} =>       write!(f, "{} {}, {}", name, rd, imm),
        }
    }
}

/// InstructionType identifier enum
#[derive(Debug, PartialEq)]
pub enum IT {
    R, I, S, B, U, J
}

/// Bitfield representation of the opcode
/// for use with type determination
#[repr(C)]
pub struct OpcodeBitfield {
    pub op4: bool,
    pub op3: bool,
    pub op2: bool,
    pub op1: bool,
    pub op0: bool
}

impl OpcodeBitfield {
    pub fn from_opcode(opcode: u8) -> Self {
        Self {
            op4:  (opcode >> 4)       == 1,
            op3: ((opcode >> 3) & 1)  == 1,
            op2: ((opcode >> 2) & 1)  == 1,
            op1: ((opcode >> 1) & 1)  == 1,
            op0:  (opcode       & 1)  == 1
        }
    }
}

/// alias u32 to instruction
pub type Instruction = u32;

// TODO: add lifting to pseudoinstructions

/// Enum to translate registers from binary value to ABI name
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone)]
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
            0 =>  ABIRegister::zero,
            1 =>  ABIRegister::ra,
            2 =>  ABIRegister::sp,
            3 =>  ABIRegister::gp,
            4 =>  ABIRegister::tp,
            5 =>  ABIRegister::t0,
            6 =>  ABIRegister::t1,
            7 =>  ABIRegister::t2,
            8 =>  ABIRegister::s0,
            9 =>  ABIRegister::s1,
            10 => ABIRegister::a0,
            11 => ABIRegister::a1,
            12 => ABIRegister::a2,
            13 => ABIRegister::a3,
            14 => ABIRegister::a4,
            15 => ABIRegister::a5,
            16 => ABIRegister::a6,
            17 => ABIRegister::a7,
            18 => ABIRegister::s2,
            19 => ABIRegister::s3,
            20 => ABIRegister::s4,
            21 => ABIRegister::s5,
            22 => ABIRegister::s6,
            23 => ABIRegister::s7,
            24 => ABIRegister::s8,
            25 => ABIRegister::s9,
            26 => ABIRegister::s10,
            27 => ABIRegister::s11,
            28 => ABIRegister::t3,
            29 => ABIRegister::t4,
            30 => ABIRegister::t5,
            31 => ABIRegister::t6,
            _   => ABIRegister::Unknown
        }
    }
}

impl Display for ABIRegister{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ABIRegister::zero => write!(f, "zero"),
            ABIRegister::ra => write!(f, "ra"),
            ABIRegister::sp => write!(f, "sp"),
            ABIRegister::gp => write!(f, "gp"),
            ABIRegister::tp => write!(f, "tp"),
            ABIRegister::t0 => write!(f, "t0"),
            ABIRegister::t1 => write!(f, "t1"),
            ABIRegister::t2 => write!(f, "t2"),
            ABIRegister::s0 => write!(f, "s0"),
            ABIRegister::s1 => write!(f, "s1"),
            ABIRegister::a0 => write!(f, "a0"),
            ABIRegister::a1 => write!(f, "a1"),
            ABIRegister::a2 => write!(f, "a2"),
            ABIRegister::a3 => write!(f, "a3"),
            ABIRegister::a4 => write!(f, "a4"),
            ABIRegister::a5 => write!(f, "a5"),
            ABIRegister::a6 => write!(f, "a6"),
            ABIRegister::a7 => write!(f, "a7"),
            ABIRegister::s2 => write!(f, "s2"),
            ABIRegister::s3 => write!(f, "s3"),
            ABIRegister::s4 => write!(f, "s4"),
            ABIRegister::s5 => write!(f, "s5"),
            ABIRegister::s6 => write!(f, "s6"),
            ABIRegister::s7 => write!(f, "s7"),
            ABIRegister::s8 => write!(f, "s8"),
            ABIRegister::s9 => write!(f, "s9"),
            ABIRegister::s10 => write!(f, "s10"),
            ABIRegister::s11 => write!(f, "s11"),
            ABIRegister::t3 => write!(f, "t3"),
            ABIRegister::t4 => write!(f, "t4"),
            ABIRegister::t5 => write!(f, "t5"),
            ABIRegister::t6 => write!(f, "t6"),
            ABIRegister::Unknown => write!(f, "")
        }
    }
}

pub static INSTRUCTIONS: phf::Map<[u8; 3], &'static str> = phf_map! {
    // RV32I
    [0b00000, 0b000, 0b0000000] => "lb",
    [0b00000, 0b001, 0b0000000] => "lh",
    [0b00000, 0b010, 0b0000000] => "lw",
    [0b00000, 0b100, 0b0000000] => "lbu",
    [0b00000, 0b101, 0b0000000] => "lhu",
    [0b00100, 0b000, 0b0000000] => "addi",
    [0b00100, 0b001, 0b0000000] => "slli",
    [0b00100, 0b010, 0b0000000] => "slti",
    [0b00100, 0b011, 0b0000000] => "sltiu",
    [0b00100, 0b100, 0b0000000] => "xori",
    [0b00100, 0b101, 0b0000000] => "srli",
    [0b00100, 0b101, 0b0100000] => "srai",    // this is the only I-type to have a relevant funct7 (NOTE: WRONG! add an i-type with funct7 please)
    [0b00100, 0b110, 0b0000000] => "ori",
    [0b00100, 0b111, 0b0000000] => "andi",
    [0b00101, 0b000, 0b0000000] => "auipc",
    [0b01000, 0b000, 0b0000000] => "sb",
    [0b01000, 0b001, 0b0000000] => "sh",
    [0b01000, 0b010, 0b0000000] => "sw",
    [0b01100, 0b000, 0b0000000] => "add",
    [0b01100, 0b000, 0b0100000] => "sub",
    [0b01100, 0b001, 0b0000000] => "sll",
    [0b01100, 0b010, 0b0000000] => "slt",
    [0b01100, 0b011, 0b0000000] => "sltu",
    [0b01100, 0b100, 0b0000000] => "xor",
    [0b01100, 0b101, 0b0000000] => "srl",
    [0b01100, 0b101, 0b0100000] => "sra",
    [0b01100, 0b110, 0b0000000] => "or",
    [0b01100, 0b111, 0b0000000] => "and",
    [0b01101, 0b000, 0b0000000] => "lui",
    [0b11000, 0b000, 0b0000000] => "beq",
    [0b11000, 0b001, 0b0000000] => "bne",
    [0b11000, 0b100, 0b0000000] => "blt",
    [0b11000, 0b101, 0b0000000] => "bge",
    [0b11000, 0b110, 0b0000000] => "bltu",
    [0b11000, 0b111, 0b0000000] => "bgeu",
    [0b11001, 0b000, 0b0000000] => "jalr",
    [0b11011, 0b000, 0b0000000] => "jal",

    // RV64I
    [0b00000, 0b011, 0b0000000] => "ld",
    [0b00000, 0b110, 0b0000000] => "lwu",
    [0b00110, 0b000, 0b0000000] => "addiw",
    [0b00110, 0b001, 0b0000000] => "slliw",
    [0b00110, 0b101, 0b0000000] => "srliw",
    //[0b00110, 0b101, 0b0100000] => "addiw",   // sort this shit too
    [0b01000, 0b011, 0b0000000] => "sd",
    [0b01110, 0b000, 0b0000000] => "addw",
    [0b01110, 0b000, 0b0100000] => "subw",
    [0b01110, 0b001, 0b0000000] => "sllw",
    [0b01110, 0b101, 0b0000000] => "srlw",
    [0b01110, 0b101, 0b0100000] => "sraw",

    // RVM

    // RVA

    // RVD

    // RVZ

    // CSR (csr registers currently unimplemented)
    [0b11100, 0b000, 0b0000000] => "syscall",  // have to handle which instruction it actually is based on the immediate values, handle with pseudoinstructions
    [0b11100, 0b001, 0b0000000] => "csrrw",
    [0b11100, 0b010, 0b0000000] => "csrrs",
    [0b11100, 0b011, 0b0000000] => "csrrc",
    [0b11100, 0b101, 0b0000000] => "csrrwi",
    [0b11100, 0b110, 0b0000000] => "csrrsi",
    [0b11100, 0b111, 0b0000000] => "csrrci"
};


// -------------------------
//          Macros
// -------------------------

/// retrieve specified fields from raw instruction bytes
// TODO: see if i need anything else for R4
// NOTE: i don't actually handle sign extension, may change that
macro_rules! retrieve {
    (opcode $inst:expr) => {
        (($inst >> 2) & 0x1f)
    };
    (rd $inst:expr) => {
        (($inst >> 7) & 0x1f)
    };
    (funct3 $inst:expr) => {
        (($inst >> 12) & 0x7) 
    };
    (funct7 $inst:expr) => {
        (($inst >> 25) & 0x7f) 
    };
    // TODO: funct2? gotta handle r4 somehow
    (rs1 $inst:expr) => {
        (($inst >> 15) & 0x1f) 
    };
    (rs2 $inst:expr) => {
        (($inst >> 20) & 0x1f) 
    };
    (simm $inst:expr) => {
        // 11 - 7:  imm[4:0]
        // 31 - 25: imm[11:5]
        (($inst >> 20)  & 0b111111100000) | 
        (($inst >> 7)   &        0b11111)
    };
    (bimm $inst:expr) => {
        // note: imm[0] is implicitly 0
        (($inst >> 19)  &   0b1000000000000) | // 31:       imm[12]
        (($inst << 4)   &    0b100000000000) | // 7:        imm[11]
        (($inst >> 20)  &     0b11111100000) | // 25-30:    imm[10:5]
        (($inst >> 7)   &           0b11110)   // 8-11:     imm[4:1]
    };
    (iimm $inst:expr) => {
        (($inst >> 20) & 0xfff)
    };
    (uimm $inst:expr) => {
        (($inst >> 12) & 0xfffff)
    };
    (jimm $inst:expr) => {
        // note: imm[0] is implicitly 0
        (($inst >> 11)  &   0b100000000000000000000) | // 31:       imm[20]
        ( $inst         &    0b11111111000000000000) | // 12-19:    imm[19:12]
        (($inst >> 9)   &            0b100000000000) | // 20:       imm[11]
        (($inst >> 20)  &             0b11111111110)   // 21-30:    imm[10:1]
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn test_immediate() {
        // I-type immediate extraction
        let i_instruction = 0b0000_1010_1101_0101_1000_0101_0001_1011;  // addiw a0, a1, 173
        assert_eq!(173, retrieve!(iimm i_instruction));

        // U-type immediate extraction
        let u_instruction = 0b0000_0000_0100_1011_0001_0111_1011_0111;  // lui a5, 1201
        assert_eq!(1201, retrieve!(uimm u_instruction));

        // S-type immediate extraction
        let s_instruction = 0b0000_0100_0111_0000_0011_1100_0010_0011;  // sd t2, 88
        assert_eq!(88, retrieve!(simm s_instruction));

        // B-type immediate extraction
        let b_instruction = 0b0100_0000_1101_0110_0101_1100_0110_0011;  // bge a2, a3, 1048
        assert_eq!(1048, retrieve!(bimm b_instruction));

        // J-type immediate extraction
        let j_instruction = 0b0010_0000_1011_0000_0001_0101_1110_1111;  // jal a1, 6666
        assert_eq!(6666, retrieve!(jimm j_instruction));
    }
}
