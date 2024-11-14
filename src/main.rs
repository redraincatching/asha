use asha::{self, output_assembly, read_compiled};

fn main() {
    println!("{:?}", std::env::current_dir());

    let bytes = read_compiled("./executables/hello");
    output_assembly(bytes);
}
