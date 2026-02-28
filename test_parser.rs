use rs_dash_pro::modules::parser::Parser;

fn test_parser(input: &str) {
    println!("Testing parser with input: '{}'", input);
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(ast) => {
            println!("Parse successful!");
            println!("AST: {:?}", ast);
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
    println!("---");
}

fn main() {
    // Test basic redirections
    test_parser("echo hello > file.txt");
    test_parser("echo hello >> file.txt");
    test_parser("cat < file.txt");
    test_parser("ls /nonexistent 2> error.log");
    test_parser("echo test 2>&1");
    test_parser("echo test 1>&2");
    
    // Test pipelines
    test_parser("echo hello | grep hello");
    test_parser("echo test | cat | grep test");
    
    // Test combination
    test_parser("echo hello | grep hello > output.txt");
}