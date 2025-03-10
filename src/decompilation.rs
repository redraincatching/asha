use std::sync::Arc;
use std::collections::BTreeMap;

use crate::instructions::InstructionType;

// List of aims
// - format for storing instructions
// - format for storing graph (could use library, will probably just use Rc)
//
// note: see the paper "No More Gotos"

#[derive(Clone)]
pub struct InstructionSection {
    id: usize,
    instructions: BTreeMap<u64, InstructionType>,
    branches: Vec<Arc<InstructionSection>>  // NOTE: this may be unnecessarily convoluted
}

impl InstructionSection {
    fn new(id: usize) -> Self {
        InstructionSection {
            id,      
            instructions: BTreeMap::new(),
            branches: Vec::new()
        }
    }

    fn push(&mut self, address: u64, instruction: InstructionType) {
        self.instructions.insert(address, instruction);
    }

    // might need reworking, we'll see
    fn add_branch(&mut self, section: &InstructionSection) {
        self.branches.push(Arc::new(section.clone()));
    }
}

// parse function
//
// note: distinguish between conditional and unconditional jumps
//
// algorithm:
// - read in instruction
// - if instruction == jump
//     - if destination == jump
//          - concatenate jumps
//     - else
//          - add branch to section
// - else if instruction == valid instruction
//     - add to current 
// - else 
//     while !eof continue;
// TODO: store the sections on the heap? and return the first one? or something
// TODO: labels, i guess
pub fn parse_jumps(instructions: BTreeMap<u64, InstructionType>) -> Vec<InstructionSection> {
    let mut sections: Vec<InstructionSection> = Vec::new();
    let mut curr_section = InstructionSection::new(0);
    let mut section_id = 1;

    for address in instructions.keys() {
        let curr = instructions.get(address).unwrap();
        // B- and J-type instructions cause a branch
        match *curr {
            InstructionType::B {..} | InstructionType::J {..} => {
                // TODO: concat jumps somehow
                // TODO: distinguish between conditional and unconditional jumps here?

                // branch to new section
                let new_section = InstructionSection::new(section_id);
                sections.push(curr_section.clone());    // save current
                // TODO: does this handle cycles at all?
                curr_section = new_section;

                section_id += 1;
            }
            _ => {
                // normal instruction, add to current block
                curr_section.push(*address, curr.clone());
            }
        }
    }

    sections.push(curr_section);

    sections
}

// TEMP FUNCTION, REMOVE
pub fn blocks_to_strings(instructions: BTreeMap<u64, InstructionType>) -> Vec<String> {
    let sections = parse_jumps(instructions);

    let mut result: Vec<String> = Vec::new();

    for section in sections {
        let mut block_str = format!("Section {}:\n", section.id);

        for (address, instruction) in section.instructions {
            block_str.push_str(&format!("{:>#8x}: {:?}\n", address, instruction));
        }

        result.push(block_str);
    }

    result
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
// we can also tell at this point that id the header is also a break, it's a while loop, otherwise it's do/while (for loops count as whiles here)
