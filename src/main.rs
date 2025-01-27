use asha::{self, output_assembly, read_compiled};

fn main() {
    let bytes = read_compiled("./executables/hello");
    output_assembly(bytes).expect("error reading object file");
}
