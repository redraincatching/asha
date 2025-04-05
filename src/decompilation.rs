use core::fmt;
use std::sync::Arc;
use std::collections::{BTreeMap, VecDeque, HashSet};

use log::{info, log_enabled, Level};

use petgraph::graph::DiGraph;

use crate::instructions::InstructionType;

// ----------------------------------------
// structures and methods
// ----------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum BranchType { Conditional, Unconditional }

#[derive(Clone, Debug, PartialEq)]
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

    pub fn get_instructions(&self) -> BTreeMap<u64, InstructionType> {
        self.instructions.clone()
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

// ----------------------------------------

/// map of instruction sections
pub type SectionMap = BTreeMap<usize, InstructionSection>;
/// map of abstract sections
pub type AbstractMap = BTreeMap<usize, AbstractSection>;

// TODO: find a way to nest them
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbstractSectionType {
    Unbranching,    // a sequence of instructions with no logical branches
    If,             // if-then construct
    IfElse,         // if-else construct
    SingleWhile,    // a single-concrete-section while loop
    While,          // a more complex while
    DoWhile,        // a do-while loop
    Break,          // a section that breaks from a while loop
    Continue,       // a section that continues to the next part of a while loop
    Acyclic         // an acyclic single-entry, single-exit complex section
}

#[derive(Clone, Debug)]
pub struct AbstractSection {
    section_type: AbstractSectionType,
    concrete_section: usize,                                // index of the concrete instruction section that this represents
    abstract_sections: Vec<AbstractSection>                 // for nesting purposes
}

impl AbstractSection {
    fn new(section_type: AbstractSectionType, idx: usize) -> Self {
        AbstractSection {
            section_type,
            concrete_section: idx,                        
            abstract_sections: Vec::new()
        }
    }

    fn add_section(&mut self, section: usize) {
        self.concrete_section = section;
    }

    fn nest_section(&mut self, section: AbstractSection) {
        self.abstract_sections.push(section);
    }

    fn get_nested_sections(&self) -> Vec<AbstractSection> {
        self.abstract_sections.clone()
    }

    fn get_type(&self) -> AbstractSectionType {
        self.section_type
    }

    /// get the id of the corresponding concrete section
    fn get_id(&self) -> usize {
        self.concrete_section
    }

    /// set the type of the section
    /// this is for printing purposes for the high-level pseudocode
    /// using Unbranching as the default
    /// if the node contains more subsections than its type expects, they are subsequent unbranching sections
    fn set_type(&mut self, section_type: AbstractSectionType) {
        if section_type != AbstractSectionType::Unbranching {
            self.section_type = section_type;
        }
    }
}

// ----------------------------------------

/// # Graph of all AbstractSections
/// simply a map of the sections corresponding to vertices, and a list of the edges
pub struct AbstractGraph {
    vertices: AbstractMap,
    edges: Vec<(usize, usize)>
}

enum Direction {
    Incoming,
    Outgoing,
    All
}

impl AbstractGraph {
    fn new() -> Self {
        AbstractGraph {
            vertices: BTreeMap::new(),
            edges: Vec::new()
        }
    }

    fn get_no_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn get_vertices(&self) -> AbstractMap {
        self.vertices.clone()
    }

    fn get_no_edges(&self) -> usize {
        self.edges.len()
    }

    /// check whether the graph contains any edges to or from the specified node
    /// returns the indices of the edges
    fn get_edges(&self, index: usize, dir: Direction) -> Vec<usize> {
        match dir {
            // incoming edge, (_, index)
            Direction::Incoming => {
                self.edges.iter()
                    .enumerate()
                    .filter(|&(_, (_, dst))| *dst == index )
                    .map(|(edge_index, _)| edge_index)
                    .collect()
            },
            // outgoing edge, (index, _)
            Direction::Outgoing => {
                self.edges.iter()
                    .enumerate()
                    .filter(|&(_, (src, _))| *src == index )
                    .map(|(edge_index, _)| edge_index)
                    .collect()
            },
            Direction::All => {
                self.edges.iter()
                    .enumerate()
                    .filter(|&(_, (src, dst))| *src == index || *dst == index )
                    .map(|(edge_index, _)| edge_index)
                    .collect()
            }
        }
    }

    /// get an edge by index
    fn get_edge(&self, idx: usize) -> Option<&(usize, usize)> {
        self.edges.get(idx)
    }

    fn contains_edge(&self, src: usize, dst: usize) -> bool {
        self.edges.contains(&(src, dst))
    }

    /// # delete an existing node
    /// add the current abstract section to its parent as a nested subsection
    /// if the node has a child, redirect any incoming edges to the child
    /// else, delete those edges
    fn reduce_node(&mut self, id: usize, parent_id: usize, section_type: AbstractSectionType) {
        let to_reduce = self.vertices.remove(&id).unwrap();
        let parent = self.vertices.get_mut(&parent_id).unwrap();

        // add section as subsection in the parent
        parent.nest_section(to_reduce);

        // set the parent section type
        parent.set_type(section_type);

        // if outgoing edge exists in set (node has a child)
        // if i was reducing the graph properly, there would only be one edge, but just to be sure
        for edge in self.get_edges(id, Direction::Outgoing) {
            // get edge from to_reduce to child
            let curr_edge = self.edges.get_mut(edge).unwrap();

            // mutate its parent
            curr_edge.0 = parent_id;
        }

        // delete parent edge to to_reduce
        // if any others still exist at the printing step, use a goto
        for edge in self.get_edges(id, Direction::Incoming) {
            let (src, dest) = self.edges.get(edge).unwrap();
            if *src == parent_id && *dest == id {
                self.edges.remove(edge);
                break;
            }
        }

        // delete any self-edges to the parent, these occur when creating while loops
        for edge in self.get_edges(parent_id, Direction::Outgoing) {
            let (src, dest) = self.edges.get(edge).unwrap();
            if *src == *dest {
                self.edges.remove(edge);
                break;
            }
        }

        // deduplicate these, edges are unique
        self.edges.dedup();
    }

    fn traverse(&self) -> ReverseInorderIterator {
        ReverseInorderIterator::new(self)
    }
}

/// iterator to allow for traversal of the graph in reverse-inorder
struct ReverseInorderIterator<'a> {
    graph: &'a AbstractGraph,
    stack: VecDeque<usize>,             // stack for dfs frontier
    visited: HashSet<usize>,            // set of previously visited nodes
    traversal_order: VecDeque<usize>    // resulting traversal order
}

impl<'a> ReverseInorderIterator<'a> {
    fn new(graph: &'a AbstractGraph) -> Self {
        let mut stack = VecDeque::new();

        // push root node onto the stack
        stack.push_back(0);

        ReverseInorderIterator {
            graph,
            stack,
            visited: HashSet::new(),
            traversal_order: VecDeque::new()
        }
    }

    fn traverse_node(&mut self, node: usize) {
        if self.visited.contains(&node) {
            return;
        }

        self.visited.insert(node);

        let outgoing_edges = self.graph.get_edges(node, Direction::Outgoing);

        for index in outgoing_edges.iter().rev() {
            let (_, child) = self.graph.edges[*index];
            if !self.visited.contains(&child) {
                self.stack.push_back(child);
            }
            self.traverse_node(child);
        }

        // add self after traversing children
        self.traversal_order.push_front(node);
    }
}

impl <'a> Iterator for ReverseInorderIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // if no nodes left to process, return
        if self.stack.is_empty() {
            return None;
        }

        let node = self.stack.pop_back().unwrap();
        self.traverse_node(node);

        self.traversal_order.pop_back()
    }
}

impl fmt::Debug for AbstractGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AbstractGraph")
            .field("vertices", &self.vertices)
            .field("edges", &self.edges)
            .finish()
    }
}

// -----------------------------------
// functions
// -----------------------------------

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
/// 
/// > aside: uninterruptible sections
/// > a block of code, B, is uninterruptible if
/// > - no instruction jumps to an address in B other than the first
/// > - no instruction in B, other than the last one, jumps
/// > 
/// > we assume that the first condition is true for all blocks identified here
/// > this limits the code that we can decompile to single-entry, single-exit sections
/// > this logic can and has been extended to multi-entry code sections, but this has not been implemented here at present
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
                        println!("{}", last_inst);
                        if let InstructionType::J { imm, .. } = last_inst {
                            // find actual destination by adding the offset
                            let pc = (*section_ptr.add(i)).end;
                            println!("pc: {}, imm: {}", pc, *imm);

                            // this can crash if labels get involved
                            let destination_addr = pc.checked_add_signed(*imm as i64).unwrap_or(0);

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

// MAYBE: change the name of this idk
/// generate the control-flow graph of the program
pub fn generate_sections(instructions: BTreeMap<u64, InstructionType>) -> SectionMap {
    let mut sections = make_blocks(instructions);
    resolve_jumps(&mut sections);

    let mut graph: SectionMap = BTreeMap::new();
    for s in sections.into_iter() {
        graph.insert(s.get_id(), s);
    }

    graph
}

// ----------------------------------------

/// build a directed graph of the cfg
fn build_graph(sections: &SectionMap) -> Option<(DiGraph<usize, ()>, Vec<petgraph::prelude::NodeIndex>)> {
    if sections.is_empty() {
        return None;
    }

    // the inidices are becasue the graph uses NodeIndex rather than integer indices
    let mut graph: DiGraph<usize, ()> = DiGraph::new();
    let mut indices: Vec<petgraph::prelude::NodeIndex> = Vec::new();

    // loop through sections and populate the graph
    for section in sections.iter() {
        // this iterates as tuples, so the index is just field 0
        indices.push(graph.add_node(*section.0));
    }

    // now that it's populated, loop through again and set the edges
    for section in sections {
        let id: usize = *section.0;
        for branch in section.1.get_branches() {
            graph.add_edge(indices[id], indices[branch.get_id()], ());
        }
    }

    Some((graph, indices))
}

// ----------------------------------------

/// construct a representation of the cfg as abstract nodes
/// starts off as an exact replica, and then gets continually reduced
/// this 1-1 mapping, with static keys for the vertices, allows us to retrieve concrete sections using the same index
fn build_abstract_graph(sections: &SectionMap) -> Option<AbstractGraph> {
    if sections.is_empty() {
        return None;
    }

    let mut graph = AbstractGraph::new();

    // insert vertices
    for (id, _) in sections.iter() {
        graph.vertices.insert(*id, AbstractSection::new(AbstractSectionType::Unbranching, *id));
    }

    // insert edges
    for section in sections.values() {
        for edge in section.get_branches() {
            graph.edges.push((section.get_id(), edge.get_id()));
        }
    }

    Some(graph)
}

// ----------------------------------------
// reduction functions
// r_ prefix used
// ----------------------------------------

/// # reduce all sequential, unbranching blocks
fn r_sequential_blocks(abstract_sections: &mut AbstractGraph) -> bool {
    // if a section has one child, and that child has only one parent and <= 1 child, concatenate them

    for a_section in abstract_sections.traverse() {
        // if this section only has one child, and that only has one parent (this current section), concatenate them
        let branches = abstract_sections.get_edges(a_section, Direction::Outgoing); 

        if branches.len() == 1 {
            let child = abstract_sections.edges[*branches.first().unwrap()].1;              // (_, dst)

            let no_children = abstract_sections.get_edges(child, Direction::Outgoing).len();
            let no_parents = abstract_sections.get_edges(child, Direction::Incoming).len();
                // no need to check that the parent is this, as we know it is a child of a_section already

            if no_parents == 1 && no_children <= 1 {
                abstract_sections.reduce_node(child, a_section, AbstractSectionType::Unbranching);

                if log_enabled!(Level::Info) {
                    info!("reduced parent {} and child {} to concatenated single block", a_section, child);
                }

                return true;
            }
        }
    }

    false
}

/// reduce all single-block while loops
/// for more complex loop reductions, see the paper "no more gotos"
fn r_single_block_while(abstract_sections: &mut AbstractGraph) -> bool {
    // if block_0 has two children,
    // if one child has exactly one parent and child, both being block_0, they are the contents of its while loop

    // traverse reverse-inorder
    for a_section in abstract_sections.traverse() {
        let children = abstract_sections.get_edges(a_section, Direction::Outgoing);

        if children.len() == 2 {
            let block_1 = abstract_sections.edges[*children.first().unwrap()].1;
            let block_2 = abstract_sections.edges[*children.last().unwrap()].1;

            let no_parents_1 = abstract_sections.get_edges(block_1, Direction::Incoming).len();
            let no_parents_2 = abstract_sections.get_edges(block_2, Direction::Incoming).len();
            let no_children_1 = abstract_sections.get_edges(block_1, Direction::Outgoing).len();
            let no_children_2 = abstract_sections.get_edges(block_2, Direction::Outgoing).len();

            // have to check recursively, no guarantee as to which block is the loop contents
            if no_parents_1 == 1 && no_children_1 == 1 &&
               abstract_sections.contains_edge(block_1, a_section) {

                abstract_sections.reduce_node(block_1, a_section, AbstractSectionType::SingleWhile);
                if log_enabled!(Level::Info) {
                    info!("reduced parent {} and child {} to single while block", a_section, block_1);
                }

                return true;
            }
            else if 
                no_parents_2 == 1 && no_children_2 == 1 &&
                abstract_sections.contains_edge(block_2, a_section) {

                abstract_sections.reduce_node(block_2, a_section, AbstractSectionType::SingleWhile);
                if log_enabled!(Level::Info) {
                    info!("reduced parent {} and child {} to single while block", a_section, block_2);
                }

                return true;
            }
        }
    }

    false
}

// reduce if-then constructs to a single abstract section
fn r_if_then(abstract_sections: &mut AbstractGraph) -> bool {
    // if block_0 has two children, block_1 and block_2
    // block_1 has one parent and one child
    // block_1's child is block_2
    // block_2 != block_0

    for a_section in abstract_sections.traverse() {
        let children = abstract_sections.get_edges(a_section, Direction::Outgoing);

        if children.len() == 2 {
            // have to check recursively, no guarantee as to which block is the if
            let block_1 = abstract_sections.edges[*children.first().unwrap()].1;
            let block_2 = abstract_sections.edges[*children.last().unwrap()].1;
            
            let no_parents_1 = abstract_sections.get_edges(block_1, Direction::Incoming).len();
            let no_parents_2 = abstract_sections.get_edges(block_2, Direction::Incoming).len();
            let no_children_1 = abstract_sections.get_edges(block_1, Direction::Outgoing).len();
            let no_children_2 = abstract_sections.get_edges(block_2, Direction::Outgoing).len();

            // test if block_1 is the if inner
            if 
                no_parents_1 == 1 && no_children_1 <= 1 &&
                abstract_sections.contains_edge(block_1, block_2) && 
                block_2 != a_section
            {
                abstract_sections.reduce_node(block_1, a_section, AbstractSectionType::If);

                if log_enabled!(Level::Info) {
                    info!("reduced parent {} and child {} to if-then", a_section, block_1);
                }

                return true;
            }
            // and then block_2
            else if 
                no_parents_2 == 1 && no_children_2 <= 1 &&
                abstract_sections.contains_edge(block_2, block_1) && 
                block_1 != a_section
            {
                abstract_sections.reduce_node(block_2, a_section, AbstractSectionType::If);

                if log_enabled!(Level::Info) {
                    info!("reduced parent {} and child {} to if-then", a_section, block_2);
                }

                return true;
            }
        }
    }

    false
}

/// reduce an if-else "diamond" to a single abstract block
fn r_if_else(abstract_sections: &mut AbstractGraph) -> bool {
    // if block_0 has 2 children
    // block_1 and block_2 have block_0 as their only parent
    // block_1 and block_2 have the same child (if any)

    for a_section in abstract_sections.traverse() {
        let children = abstract_sections.get_edges(a_section, Direction::Outgoing);

        if children.len() == 2 {
            // no reflexivity here
            let block_1 = abstract_sections.edges[*children.first().unwrap()].1;
            let block_2 = abstract_sections.edges[*children.last().unwrap()].1;

            let no_parents_1 = abstract_sections.get_edges(block_1, Direction::Incoming).len();
            let no_parents_2 = abstract_sections.get_edges(block_2, Direction::Incoming).len();
            let children_1 = abstract_sections.get_edges(block_1, Direction::Outgoing);
            let no_children_1 = children_1.len();
            let children_2 = abstract_sections.get_edges(block_2, Direction::Outgoing);
            let no_children_2 = children_2.len();
            
            if
                no_parents_1 == 1 && no_parents_2 == 1 &&
                (no_children_1 == 0 && no_children_2 == 0) || 
                (no_children_1 == 1 && no_children_2 == 1 && abstract_sections.edges[*children_1.first().unwrap()].1 == abstract_sections.edges[*children_2.first().unwrap()].1) {
                    abstract_sections.reduce_node(block_1, a_section, AbstractSectionType::IfElse);
                    abstract_sections.reduce_node(block_2, a_section, AbstractSectionType::IfElse);

                    if log_enabled!(Level::Info) {
                        info!("reduced parent {}, child_1 {}, and child_2 {} to if-else", a_section, block_1, block_2);
                    }

                    return true;
            }
        }
    }

    false
}

// ----------------------------------------

/// # iteratively reduce the control-flow graph to nested abstract sections
/// we start with an abstract map that is a 1-1 representation of the concrete instruction sections
/// we then apply a series of reductions to it, such that each transformation, if successfully applied, decrease the size of the graph
/// because of this, it's guaranteed to stop
/// this means i've basically solved the halting problem - take that, turing
pub fn iterated_cfg_reduction(sections: SectionMap) -> Option<AbstractGraph> {
    // generate cfg from code blocks
    let (_cfg, _indices) = build_graph(&sections)?;

    // generate 1-1 map of abstract sections
    let mut abstract_graph = build_abstract_graph(&sections)?;

    let mut processing = true;

    while processing {
        processing = false;

        // we attempt to apply the following reductions
        // - reduce sequential blocks to single blocks
        processing = processing || r_sequential_blocks(&mut abstract_graph);

        // - reduce self-loop to while
        processing = processing || r_single_block_while(&mut abstract_graph);

        // - reduce single-step branch to if
        processing = processing || r_if_then(&mut abstract_graph);

        // - reduce "diamond" to if-else statement
        processing = processing || r_if_else(&mut abstract_graph);
    }

    Some(abstract_graph)
}

// ----------------------------------------

/// insert n tab characters
macro_rules! indent {
    ($n:expr) => {
        "\t".repeat($n)
    };
}

/// function to convert to a higher-level representation
fn high_level_conversion(concrete_sections: SectionMap, abstract_sections: AbstractGraph) -> Vec<String> {
    // traverse and output to a vector of strings, i think 
    let mut indent = 0;
    let mut output: Vec<String> = Vec::new();

    // function signature
    // this will be hardcoded because i'm not extracting it well enough from the raw bytes
    output.push("void main() {".to_string());
    indent += 1;

    // call iteratively on any existing vertices
    for (_, section) in abstract_sections.get_vertices() {
        // get corresponding concrete section
        convert_section(section, &mut output, &abstract_sections, &concrete_sections, &mut indent);
    }

    // closing brace
    output.push("}".to_string());

    output
}

// ----------------------------------------

/// # Condition conversion helper function
/// conditions: 
/// - beq: c0 == c1
/// - bne: c0 != c1
/// - blt(u): c0 <  c1
/// - bgt(u): c0 >  c1
/// - ble(u): c0 <= c1
/// - bgt(u): c0 >= c1:w
fn condition(inst: &InstructionType) -> Option<String> {
    // we know this must be a b-type instruction
    // i think
    match inst.get_name() {
        "beq" => Some(format!("{} == {}", inst.get_rs1(), inst.get_rs2())),
        "bne" => Some(format!("{} != {}", inst.get_rs1(), inst.get_rs2())),
        "blt" | "bltu"
            => Some(format!("{} < {}", inst.get_rs1(), inst.get_rs2())),
        "bgt" | "bgtu"
            => Some(format!("{} > {}", inst.get_rs1(), inst.get_rs2())),
        "ble" | "bleu"
            => Some(format!("{} <= {}", inst.get_rs1(), inst.get_rs2())),
        "bge" | "bgeu"
            => Some(format!("{} >= {}", inst.get_rs1(), inst.get_rs2())),
        _ => None
    }
}

/// # Operator conversion helper function
/// operators:
/// - lb, lh, lw, lbu, lhu, lui, ld, kwu: dst = src
/// - add, addw, addi, addiw: dst = op0 + op1
/// - sub, subw, subi, subiw: dst = op0 - op1
/// - and, andi: dst = op0 & op1
/// - or, ori: dst = op0 | op1
/// - xor, xori: dst = op0 ^ op1
/// - mul, mulh, mulhsu, mulhu, mulw: dst = op0 * op1 
/// - div, divu, divw, divuw: dst = op0 / op1
/// - remw, remuw: dst = op0 % op1
fn operator(inst: &InstructionType) -> Option<String> {
    // these are all separated as some require register values, others immediates
    match inst.get_name() {
        "lb" | "lh" | "lw" | "lbu" | "lhu" | "ld" | "lwu" 
            => Some(format!("{} = {}", inst.get_rd(), inst.get_rs1())),
        "lui" 
            => Some(format!("{} = {}", inst.get_rd(), inst.get_imm())),
        "addi" | "addiw"
            => Some(format!("{} = {} + {}", inst.get_rd(), inst.get_rs1(), inst.get_imm())),
        "add" | "addw"
            => Some(format!("{} = {} + {}", inst.get_rd(), inst.get_rs1(), inst.get_rs2())),
        "subi" | "subiw"
            => Some(format!("{} = {} - {}", inst.get_rd(), inst.get_rs1(), inst.get_imm())),
        "sub" | "subw"
            => Some(format!("{} = {} - {}", inst.get_rd(), inst.get_rs1(), inst.get_rs2())),
        "ori"
            => Some(format!("{} = {} | {}", inst.get_rd(), inst.get_rs1(), inst.get_imm())),
        "or"
            => Some(format!("{} = {} | {}", inst.get_rd(), inst.get_rs1(), inst.get_rs2())),
        "xori"
            => Some(format!("{} = {} | {}", inst.get_rd(), inst.get_rs1(), inst.get_imm())),
        "xor"
            => Some(format!("{} = {} | {}", inst.get_rd(), inst.get_rs1(), inst.get_rs2())),
        "mul" | "mulh" | "mulhsu" | "mulhu" | "mulw"
            => Some(format!("{} = {} * {}", inst.get_rd(), inst.get_rs1(), inst.get_rs2())),
        "div" | "divu" | "divw" | "divuw"
            => Some(format!("{} = {} / {}", inst.get_rd(), inst.get_rs1(), inst.get_rs2())),
        "remw" | "remuw"
            => Some(format!("{} = {} % {}", inst.get_rd(), inst.get_rs1(), inst.get_rs2())),
        _ => None
    }
}

// ----------------------------------------

fn convert_section(section: AbstractSection, output: &mut Vec<String>, abstract_map: &AbstractGraph, concrete_sections: &SectionMap, indent: &mut usize) {
    let concrete_section = concrete_sections.get(&section.get_id());
    let instructions = concrete_section.unwrap().get_instructions();
    let count = instructions.values().count();

    // get the last instruction to check the branch condition
    let last_instruction = instructions.values().last().unwrap();

    // based on type of section, wrap guard and call on next section
    // pass in output to keep pushing
    // remember that we may have more sections contained, that just means that more sections follow, as this has been reduced down multiple times
    match section.get_type() {
        AbstractSectionType::If => {
            // stringify each instruction in the new language and push to the output vector
            for (index, instruction) in instructions.values().enumerate() {
                // the last instruction will be handled in the guard
                if index < count - 1 {
                    output.push(convert_instruction(instruction, *indent));
                }
            }

            output.push(format!("{}if ({}) {{", indent!(*indent), condition(last_instruction).unwrap_or("true".to_string())));
            *indent += 1;

            // call function for if branch
            convert_section(section.get_nested_sections().first().unwrap().clone(), output, abstract_map, concrete_sections, indent);

            *indent -= 1;
            output.push(format!("{}}}", indent!(*indent)));

            // we've already used the first in the if block
            for remaining in section.get_nested_sections().iter().skip(1) {
                convert_section(remaining.clone(), output, abstract_map, concrete_sections, indent);
            }
        },
        AbstractSectionType::IfElse => {
            // stringify each instruction in the new language and push to the output vector
            for (index, instruction) in instructions.values().enumerate() {
                // the last instruction will be handled in the guard
                if index < count - 1 {
                    output.push(convert_instruction(instruction, *indent));
                }
            }

            output.push(format!("{}if ({}) {{", indent!(*indent), condition(last_instruction).unwrap_or("true".to_string())));
            *indent += 1;

            // call function for if branch
            convert_section(section.get_nested_sections().first().unwrap().clone(), output, abstract_map, concrete_sections, indent);

            *indent -= 1;
            output.push(format!("{}}}", indent!(*indent)));

            output.push(format!("{}else {{", indent!(*indent)));
            *indent += 1;

            // call function for if branch
            convert_section(section.get_nested_sections().get(1).unwrap().clone(), output, abstract_map, concrete_sections, indent);

            *indent -= 1;
            output.push(format!("{}}}", indent!(*indent)));

            // this time we skip both of them
            for remaining in section.get_nested_sections().iter().skip(2) {
                convert_section(remaining.clone(), output, abstract_map, concrete_sections, indent);
            }
        },
        AbstractSectionType::SingleWhile => {
            // stringify each instruction in the new language and push to the output vector
            for (index, instruction) in instructions.values().enumerate() {
                // the last instruction will be handled in the guard
                if index < count - 1 {
                    output.push(convert_instruction(instruction, *indent));
                }
            }

            output.push(format!("{}while ({}) {{", indent!(*indent), condition(last_instruction).unwrap_or("true".to_string())));
            *indent += 1;

            for remaining in section.get_nested_sections() {
                convert_section(remaining.clone(), output, abstract_map, concrete_sections, indent);
            }

            *indent -= 1;
            output.push(format!("{}}}", indent!(*indent)));
        },
        AbstractSectionType::Unbranching => {
            // stringify each instruction in the new language and push to the output vector
            for (index, instruction) in instructions.values().enumerate() {
                // the last instruction will be handled in the guard
                if index < count - 1 {
                    output.push(convert_instruction(instruction, *indent));
                }
            }

            for remaining in section.get_nested_sections() {
                convert_section(remaining.clone(), output, abstract_map, concrete_sections, indent);
            }
        },
        _ => {
            unimplemented!()
        }
    }

    // if any outgoing edges from this function still exist
    // goto that section
    let outgoing = abstract_map.get_edges(section.get_id(), Direction::Outgoing);
    if !outgoing.is_empty() {
        for idx in outgoing {
            let (_, dest) = abstract_map.get_edge(idx).unwrap();
            output.push(format!("{}GOTO section {};", indent!(*indent), dest));
        }
    }
}

/// process each single instruction
fn convert_instruction(inst: &InstructionType, indent: usize) -> String {
    // do the actual logic here
    if let Some(op) = operator(inst) {
        return format!("{}{};", indent!(indent), op);
    }

    if inst.get_name() == "syscall" {
        return format!("{}{};", indent!(indent), "ecall()");
    }

    // yes this is half-assed, this has to somewhat function in the next 4 hours
    format!("{}{};", indent!(indent), inst)
}

// ----------------------------------------

/// function to be called by the main app
pub fn output_decompiled_code(cfg: SectionMap) -> Vec<String> {
    let reduced_graph = iterated_cfg_reduction(cfg.clone());

    high_level_conversion(cfg, reduced_graph.unwrap())
}

// ----------------------------------------

// ----------------------------------------
// unit tests
// ----------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    // helper functions to create graphs

    fn create_fibb_graph() -> AbstractGraph {
        let mut graph = AbstractGraph::new();

        graph.vertices.insert(0, AbstractSection::new(AbstractSectionType::Unbranching, 0));
        graph.vertices.insert(1, AbstractSection::new(AbstractSectionType::Unbranching, 1));
        graph.vertices.insert(2, AbstractSection::new(AbstractSectionType::Unbranching, 2));
        graph.vertices.insert(3, AbstractSection::new(AbstractSectionType::Unbranching, 3));
        graph.vertices.insert(4, AbstractSection::new(AbstractSectionType::Unbranching, 4));

        graph.edges.push((0, 4));
        graph.edges.push((0, 1));
        graph.edges.push((1, 2));
        graph.edges.push((1, 4));
        graph.edges.push((2, 3));
        graph.edges.push((2, 4));
        graph.edges.push((3, 2));

        graph
    }

    fn create_test_graph() -> AbstractGraph {
        let mut graph = AbstractGraph::new();

        // the structure here is specifically
        // - if-else
        // - if-then
        // - sb-while
        // - sequential
        // in that order

        graph.vertices.insert(0, AbstractSection::new(AbstractSectionType::Unbranching, 0));
        graph.vertices.insert(1, AbstractSection::new(AbstractSectionType::Unbranching, 1));
        graph.vertices.insert(2, AbstractSection::new(AbstractSectionType::Unbranching, 2));
        graph.vertices.insert(3, AbstractSection::new(AbstractSectionType::Unbranching, 3));
        graph.vertices.insert(4, AbstractSection::new(AbstractSectionType::Unbranching, 4));
        graph.vertices.insert(5, AbstractSection::new(AbstractSectionType::Unbranching, 5));
        graph.vertices.insert(6, AbstractSection::new(AbstractSectionType::Unbranching, 6));
        graph.vertices.insert(7, AbstractSection::new(AbstractSectionType::Unbranching, 7));
        graph.vertices.insert(8, AbstractSection::new(AbstractSectionType::Unbranching, 8));
        graph.vertices.insert(9, AbstractSection::new(AbstractSectionType::Unbranching, 9));
        graph.vertices.insert(10, AbstractSection::new(AbstractSectionType::Unbranching, 10));

        graph.edges.push((0, 1));
        graph.edges.push((1, 2));
        graph.edges.push((1, 3));
        graph.edges.push((2, 4));
        graph.edges.push((3, 4));
        graph.edges.push((4, 5));
        graph.edges.push((4, 6));
        graph.edges.push((5, 6));
        graph.edges.push((6, 7));
        graph.edges.push((7, 8));
        graph.edges.push((8, 7));
        graph.edges.push((7, 9));
        graph.edges.push((9, 10));

        graph
    }

    // functions to test specific graph functionalities
    // part 1: constructed graph

    #[test]
    fn test_reverse_inorder_traversal() {
        // using the structure of my fibbonacci function to test
        let graph = create_fibb_graph();

        let expected_order = vec![4, 3, 2, 1, 0];
        let result: Vec<usize> = graph.traverse().collect();

        assert_eq!(result, expected_order);
    }

    #[test]
    fn test_r_sequential_blocks() {
        let mut graph = create_test_graph();
        let initial_size = graph.get_no_vertices();

        let reduced = r_sequential_blocks(&mut graph);

        // check that the operation took place as intended
        assert!(reduced);
        assert_eq!(graph.vertices.len(), initial_size - 1);

        // check that node 9 has node 10 nested within
        let modified = graph.vertices.get(&9).unwrap();
        assert_eq!(modified.get_type(), AbstractSectionType::Unbranching);
        assert!(!modified.abstract_sections.is_empty());
    }

    #[test]
    fn test_r_single_block_while() {
        let mut graph = create_test_graph();
        let initial_size = graph.get_no_vertices();

        let reduced = r_single_block_while(&mut graph);

        // check that the operation took place as intended
        assert!(reduced);
        assert_eq!(graph.vertices.len(), initial_size - 1);

        // check that 7 contains 8
        let modified = graph.vertices.get(&7).unwrap();
        assert_eq!(modified.get_type(), AbstractSectionType::SingleWhile);
        assert!(!modified.abstract_sections.is_empty());
    }

    #[test]
    fn test_r_if_then() {
        let mut graph = create_test_graph();
        let initial_size = graph.get_no_vertices();

        let reduced = r_if_then(&mut graph);

        // check that the operation took place as intended
        assert!(reduced);
        assert_eq!(graph.vertices.len(), initial_size - 1);

        // check that 4 contains 5
        let modified = graph.vertices.get(&4).unwrap();
        assert_eq!(modified.get_type(), AbstractSectionType::If);
        assert!(!modified.abstract_sections.is_empty());
    }

    #[test]
    fn test_r_if_else() {
        let mut graph = create_test_graph();
        let initial_size = graph.get_no_vertices();

        let reduced = r_if_else(&mut graph);

        // check that the operation took place as intended
        assert!(reduced);
        assert_eq!(graph.vertices.len(), initial_size - 2);

        // check that 1 contains 2 and 3
        let modified = graph.vertices.get(&1).unwrap();
        assert_eq!(modified.get_type(), AbstractSectionType::IfElse);
        assert_eq!(modified.abstract_sections.len(), 2);
    }

    // test the iterated reduction process
    #[test]
    fn test_iterated_reduction() {
        let mut abstract_graph = create_test_graph();

        let mut processing = true;
        let mut count = 0;

        while processing {
            processing = false;
            count += 1;

            // we attempt to apply the following reductions
            // - reduce sequential blocks to single blocks
            processing = processing || r_sequential_blocks(&mut abstract_graph);
            //println!("graph:\n{:?}", abstract_graph);

            // - reduce self-loop to while
            processing = processing || r_single_block_while(&mut abstract_graph);
            //println!("graph:\n{:?}", abstract_graph);

            // - reduce single-step branch to if
            processing = processing || r_if_then(&mut abstract_graph);
            //println!("graph:\n{:?}", abstract_graph);

            // - reduce "diamond" to if-else statement
            processing = processing || r_if_else(&mut abstract_graph);
            //println!("graph:\n{:?}", abstract_graph);

        }

        // should reduce to a single vertex, actually
        assert_eq!(abstract_graph.get_no_vertices(), 1);
        assert_eq!(count, 10);
    }

    // part 2: fibbonacci function graph
    #[test]
    fn test_reverse_inorder_traversal_fibb() {
        // using the structure of my fibbonacci function to test
        let graph = create_fibb_graph();

        let expected_order = vec![4, 3, 2, 1, 0];
        let result: Vec<usize> = graph.traverse().collect();

        assert_eq!(result, expected_order);
    }

    // test the iterated reduction process
    #[test]
    fn test_iterated_reduction_fibb() {
        let mut abstract_graph = create_fibb_graph();

        let mut processing = true;
        let mut count = 0;

        while processing {
            processing = false;
            // println!("run {}, no. vertices in graph: {}, no. edges: {}", count, abstract_graph.get_no_vertices(), abstract_graph.get_no_edges());
            count += 1;

            // we attempt to apply the following reductions
            // - reduce sequential blocks to single blocks
            processing = processing || r_sequential_blocks(&mut abstract_graph);

            // - reduce self-loop to while
            processing = processing || r_single_block_while(&mut abstract_graph);

            // - reduce single-step branch to if
            processing = processing || r_if_then(&mut abstract_graph);

            // - reduce "diamond" to if-else statement
            processing = processing || r_if_else(&mut abstract_graph);
        }

        // should reduce to a single vertex, actually
        assert_eq!(abstract_graph.get_no_vertices(), 1);
        assert_eq!(count, 5);
    }
}
