pub fn read_compiled(filepath: &str) -> Vec<u8> {
    // could get this to return an iterator?
    // TODO: error handling
    let bytes = std::fs::read(filepath).unwrap();

    // TODO: separate them in a useful way
    // maybe look for useful sections
    bytes
}

pub fn output_assembly(bytes: Vec<u8>) {
    let mut address : u64 = 0;
    
    // should maybe look at just outputting .text section?
    // maybe that's wrong, see how other programs do it
    // also this doesn't print the labels
    for row in bytes.chunks_exact(4) {
        print!("{:<#10x}", address);
        address += 4;

        for byte in row.iter() {
            print!("{:0>2x}", byte);
        }

        println!("    opcode goes here");
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
