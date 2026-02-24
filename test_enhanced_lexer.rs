//! Test enhanced lexer with new POSIX features

use rs_dash_pro::EnhancedLexer;

fn main() {
    println!("Testing Enhanced Lexer with POSIX features...");
    
    // Test 1: Pattern matching
    println!("\n=== Test 1: Pattern matching ===");
    test_lexer("echo *.txt");
    
    // Test 2: Arithmetic expansion
    println!("\n=== Test 2: Arithmetic expansion ===");
    test_lexer("echo $((1 + 2))");
    
    // Test 3: Process substitution (bash extension)
    println!("\n=== Test 3: Process substitution ===");
    test_lexer("cat <(ls) >(grep pattern)");
    
    // Test 4: Extended globbing
    println!("\n=== Test 4: Extended globbing ===");
    test_lexer("echo !(*.bak|*.tmp)");
    
    // Test 5: Complex command with all features
    println!("\n=== Test 5: Complex command ===");
    test_lexer("for file in *.txt; do echo $(( $(wc -l < \"$file\") + 1 )); done");
}

fn test_lexer(input: &str) {
    println!("Input: {}", input);
    
    let lexer = EnhancedLexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    
    println!("Tokens ({}):", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?} '{}'", i, token.token_type, token.lexeme);
    }
}