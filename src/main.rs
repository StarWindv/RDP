use rdp::modules::cli::parse::Cli;

fn main() {
    let argv = Cli::run();
    match argv {
        _ => {}, // TODO: args allocator
    }
}
