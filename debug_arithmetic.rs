//! Debug test for arithmetic expansion

use rs_dash_pro::EnhancedLexer;

fn main() {
    let input = "echo $((1 + 2))";
    println!("Testing input: {}", input);
    
    let lexer = EnhancedLexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?} '{}'", i, token.token_type, token.lexeme);
    }
}