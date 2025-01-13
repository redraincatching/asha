// TODO: enumerate all supported codes

/// Enum to translate registers from binary value to ABI name
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Register {
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

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            0x0 =>  Register::zero,
            0x1 =>  Register::ra,
            0x2 =>  Register::sp,
            0x3 =>  Register::gp,
            0x4 =>  Register::tp,
            0x5 =>  Register::t0,
            0x6 =>  Register::t1,
            0x7 =>  Register::t2,
            0x8 =>  Register::s0,
            0x9 =>  Register::s1,
            0x10 => Register::a0,
            0x11 => Register::a1,
            0x12 => Register::a2,
            0x13 => Register::a3,
            0x14 => Register::a4,
            0x15 => Register::a5,
            0x16 => Register::a6,
            0x17 => Register::a7,
            0x18 => Register::s2,
            0x19 => Register::s3,
            0x20 => Register::s4,
            0x21 => Register::s5,
            0x22 => Register::s6,
            0x23 => Register::s7,
            0x24 => Register::s8,
            0x25 => Register::s9,
            0x26 => Register::s10,
            0x27 => Register::s11,
            0x28 => Register::t3,
            0x29 => Register::t4,
            0x30 => Register::t5,
            0x31 => Register::t6,
            _   => Register::Unknown
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
