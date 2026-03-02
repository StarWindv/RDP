use rdp::modules::{
    utils::tools::Tools
};


fn main() {
    println!("{}", Tools::build_usage());
    println!("\n{}", Tools::build_version());
}
