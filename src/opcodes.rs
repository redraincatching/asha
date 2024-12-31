/// # Every Supported Opcode Enumerated

// TODO: remove any of these that I don't support
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Opcode {
    LOAD,
    LOAD_FP,
    MISC_MEM,
    OP_IMM,
    AUIPC,
    OP_IMM_32,
    STORE,
    STORE_FP,
    AMO,
    OP,
    LUI,
    OP_32,
    MADD,
    MSUB,
    NMSUB,
    NMADD,
    OP_FP,
    OP_V,
    BRANCH,
    JALR,
    JAL,
    SYSTEM,
    OP_VE,
    Unknown     // anything else
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        match value {
            0b00000 => Opcode::LOAD,
            0b00001 => Opcode::LOAD_FP,
            // custom, ignored
            0b00011 => Opcode::MISC_MEM,
            0b00100 => Opcode::OP_IMM,
            0b00101 => Opcode::AUIPC,
            0b00110 => Opcode::OP_IMM_32,
            0b01000 => Opcode::STORE,
            0b01001 => Opcode::STORE_FP,
            // custom, ignored
            0b01011 => Opcode::AMO,
            0b01100 => Opcode::OP,
            0b01101 => Opcode::LUI,
            0b01110 => Opcode::OP_32,
            0b10000 => Opcode::MADD,
            0b10001 => Opcode::MSUB,
            0b10010 => Opcode::NMADD,
            0b10011 => Opcode::NMSUB,
            0b10100 => Opcode::OP_FP,
            0b10101 => Opcode::OP_V,
            // custom, ignored
            0b11000 => Opcode::BRANCH,
            0b11001 => Opcode::JALR,
            // reserved
            0b11011 => Opcode::JAL,
            0b11100 => Opcode::SYSTEM,
            0b11101 => Opcode::OP_VE,
            // custom, ignored
            _       => Opcode::Unknown
        }
    }
}
