use core::fmt;
use std::sync::Arc;
use std::collections::BTreeMap;

use crate::instructions::InstructionType;

// List of aims
// - format for storing instructions
// - format for storing graph (could use library, will probably just use Rc)
//
// note: see the paper "No More Gotos"

#[derive(Clone, Debug, PartialEq)]
enum BranchType { Conditional, Unconditional }

#[derive(Clone)]
pub struct InstructionSection {
    id: usize,
    instructions: BTreeMap<u64, InstructionType>,
    branches: Vec<Arc<InstructionSection>>,
    branch_type: Option<BranchType>,
    start: u64,                                     // lower bound of block addresses
    end: u64                                        // upper bound for block addresses
}

impl InstructionSection {
    fn new(id: usize) -> Self {
        InstructionSection {
            id,      
            instructions: BTreeMap::new(),
            branches: Vec::new(),
            branch_type: None,
            start: 0,
            end: 0
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    fn push(&mut self, address: u64, instruction: InstructionType) {
        self.instructions.insert(address, instruction);
    }

    fn set_branch_type(&mut self, branch_type: BranchType) {
        self.branch_type = Some(branch_type);
    }

    // might need reworking, we'll see
    fn add_branch(&mut self, section: Arc<InstructionSection>) {
        self.branches.push(section);
    }

    pub fn get_branches(&self) -> &[Arc<InstructionSection>] {
        &self.branches
    }

    /// extend range covered by codeblock
    fn add_to_range(&mut self, address: u64) {
        if self.start == 0 {
            self.start = address;
        }
        if address > self.end {
            self.end = address;
        }
    }

    /// determine if a given address is in this codeblock
    fn in_block(&self, address: u64) -> bool {
        (address >= self.start) && (address <= self.end)
    }
}

impl fmt::Display for InstructionSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut block_str = format!("section {}:\n", self.id);

        for (address, instruction) in self.instructions.iter() {
            block_str.push_str(&format!("{:>#8x}: {}\n", address, instruction));
        }

        if !self.branches.is_empty() {
            let branches = self.get_branches();
            if self.branch_type == Some(BranchType::Conditional) {
                block_str.push_str(&format!("\ttrue: jump to section {}\n", branches.first().unwrap().get_id()));
                block_str.push_str(&format!("\tfalse: jump to section {}\n", branches.get(1).unwrap().get_id()));
            } else {
                block_str.push_str(&format!("\tjump to section {}\n", branches.first().unwrap().get_id()));
            }
        }                

        write!(f, "{}", block_str) 
    }
}

/// # parse function
///
/// algorithm:
/// - read in instruction
/// - if instruction == jump
///     - if destination == jump
///          - concatenate jumps
///     - else
///          - add branch to section
/// - else if instruction == valid instruction
///     - add to current 
/// - else 
///     while !eof continue;
fn make_blocks(instructions: BTreeMap<u64, InstructionType>) -> Vec<InstructionSection> {
    let mut sections: Vec<InstructionSection> = Vec::new();
    let mut curr_section = InstructionSection::new(0);
    let mut section_id = 1;

    for address in instructions.keys() {
        let curr = instructions.get(address).unwrap();

        // B- and J-type instructions cause a branch
        match *curr {
            InstructionType::B {..} | InstructionType::J {..} => {
                // set branch_type based on whether this is B or J
                let branch_type = match curr {
                    InstructionType::B { .. } => BranchType::Conditional,
                    InstructionType::J { .. } => BranchType::Unconditional,
                    _ => unreachable!()
                };

                curr_section.set_branch_type(branch_type);

                // add jump to current section
                curr_section.push(*address, curr.clone());
                curr_section.add_to_range(*address);

                // branch to new section
                let new_section = InstructionSection::new(section_id);
                sections.push(curr_section.clone());    // save current
                curr_section = new_section;

                section_id += 1;

                // TODO: sequential jumps?
            }
            _ => {
                // normal instruction, add to current block
                curr_section.push(*address, curr.clone());
                curr_section.add_to_range(*address);
            }
        }
    }

    sections.push(curr_section);

    sections
}

/// # determine what children each section has
/// - if a block can branch, add an edge to that destination, and another to the immediate next block (fallthrough)
/// - if a block always branches, and its child is within this function, add an edge to it
///     - if its child is not, it is assumed that it is a returning subfunction, and so add a fallthrough edge
/// - if none of the above applies, add a fallthrough edge
///
/// ## unconditional jump resolution
/// there are three jump instructions in RVI - `j`, `jal`, and `jalr`
/// ```riscv
/// j       imm             # pc += imm
/// jal     rd, imm         # rd = pc+4; pc += imm
/// jalr    rd, rs1, imm    # rd = pc+4; pc = rs1+imm
/// ```
/// `j` is a simple jump, `jal` and `jalr` are for function calls
/// the return address for these jumps are stored (the next function in order) in rd before updating pc
/// `jal` uses a 20-bit signed immediate for the jump destination
/// `jalr` uses a register plus a 12-bit signed offset
///
/// generally, we use `jal` to call an instruction, and `jalr` to return from them
/// the pseudoinstructions `call` and `ret` do this pretty nicely
/// 
/// it should also be noted that `j` is a pseudoinstruction, translated to `jal` with a return address of the zero register
fn resolve_jumps(sections: &mut [InstructionSection]) {
    let section_ptr = sections.as_mut_ptr();

    // because i missed writing c
    unsafe {
        for i in 0..sections.len() {
            if let Some(branch_type) = &(*section_ptr.add(i)).branch_type {
                match branch_type {
                    BranchType::Conditional => {
                        // get offset from last instruction
                        let last_inst = (*section_ptr.add(i)).instructions.values().last().unwrap();
                        if let InstructionType::B { imm, .. } = last_inst {
                            // find actual destination by adding the offset
                            let pc = (*section_ptr.add(i)).end;
                            let destination_addr = pc.checked_add_signed(*imm as i64).unwrap();

                            // if destination in within this function, add it as a branch
                            if let Some(target_index) = find_section(sections, destination_addr) {
                                let target_section = sections.get(target_index).unwrap();
                                (*section_ptr.add(i)).add_branch(Arc::new(target_section.clone()));
                            }
                            // in either case, add fallthrough edge if later blocks exist
                            if i + 1 < sections.len() {
                                let target_section = sections.get(i+1).unwrap();
                                (*section_ptr.add(i)).add_branch(Arc::new(target_section.clone()));
                            }
                        }
                    },
                    BranchType::Unconditional => {
                        // get offset from last instruction
                        let last_inst = (*section_ptr.add(i)).instructions.values().last().unwrap();
                        if let InstructionType::J { imm, .. } = last_inst {
                            // find actual destination by adding the offset
                            let pc = (*section_ptr.add(i)).end;
                            let destination_addr = pc.checked_add_signed(*imm as i64).unwrap();

                            // if destination in within this function, add it as a branch
                            if let Some(target_index) = find_section(sections, destination_addr) {
                                let target_section = sections.get(target_index).unwrap();
                                (*section_ptr.add(i)).add_branch(Arc::new(target_section.clone()));
                            } else if i + 1 < sections.len() {
                                // for unconditional branches, we only have one edge
                                let target_section = sections.get(i+1).unwrap();
                                (*section_ptr.add(i)).add_branch(Arc::new(target_section.clone()));
                            }
                        }
                    }
                }
            } else {
                // no branch, just fallthrough if not the last section
                if i + 1 < sections.len() {
                    let target_section = sections.get(i+1).unwrap();
                    (*section_ptr.add(i)).add_branch(Arc::new(target_section.clone()));
                }
            }
        }
    }
}

// see which section a given address exists in
fn find_section(sections: &[InstructionSection], address: u64) -> Option<usize> {
    for (i, section) in sections.iter().enumerate() {
        if section.in_block(address) {
            return Some(i)
        }
    }
    None
}

pub fn generate_cfg(instructions: BTreeMap<u64, InstructionType>) -> Vec<InstructionSection> {
    let mut sections = make_blocks(instructions);
    resolve_jumps(&mut sections);

    sections
}

// reduce cfg to abstract code
// - reduce sequential blocks to single blocks
// - reduce self-loop to while
// - reduce single-step branch to if-then
// - reduce "diamond" to if-else statement
// - reducing loops (see below)

// reducing loops
// multiple preparatory steps
// - find cyclic regions in a given graph
// - find all looping paths
// - combine looping paths to determine associated nodes for loop
// - determine and reduce break and continue statements
// - reduce loop body graph to single node
//
// we use an algorithm to find the "strongly connected components" in a graph
// we then find all of the "back edges" to see how many loop headers there are
// the cycle paths are then enumerated with johnson's algorithm
// then reduce any paths that only pass through one header as follows:
// - link back to header? continue node
// - is a child in loop body? not a tail node
// - is a child not in loop body? break node
// - parent not in body or header list? multi-entry, cannot be reduced immediately
// 
// we can also tell at this point that if the header is also a break, it's a while loop, otherwise it's do/while (for loops count as whiles here)
