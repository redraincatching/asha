/// # Every Supported Opcode Enumerated
/// NOTEThis may not be necessary
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

// -------------------------
// Macros to retrieve fields
// -------------------------

/// retrieve specified fields from raw instruction bytes
// NOTE: this isn't totally correct, I don't think
// need to write tests to make sure these all work, especially the immediates
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
        // CHECK MY IMPLEMENTATION OF THIS AGAINST ANOTHER TOOL
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
        // CHECK MY IMPLEMENTATION OF THIS AGAINST ANOTHER TOOL
        (($inst >> 11)  &   0b10000000000000000000) + // 31:       imm[20]
        ( $inst         &    0b1111111100000000000) + // 12-19:    imm[19:12]
        (($inst >> 9)   &            0b10000000000) + // 20:       imm[11]
        (($inst >> 21)  &             0b1111111111)   // 21-30:    imm[10:1]
    }
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
