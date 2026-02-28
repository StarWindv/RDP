//! Tests for pipeline and redirection functionality

use rs_dash_pro::modules::parser::Parser;
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
use rs_dash_pro::modules::ssa_executor::SsaExecutor;

#[test]
fn test_simple_pipeline() {
    // Test: echo hello | grep hello
    let input = "echo hello | grep hello";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    // Should succeed (grep finds hello)
    assert_eq!(result, 0);
}

#[test]
fn test_output_redirection() {
    // Test: echo test > /tmp/test.txt
    let input = "echo test > /tmp/test.txt";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    assert_eq!(result, 0);
}

#[test]
fn test_input_redirection() {
    // Test: cat < /tmp/test.txt
    let input = "cat < /tmp/test.txt";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    assert_eq!(result, 0);
}

#[test]
fn test_error_redirection() {
    // Test: ls /nonexistent 2> /tmp/error.log
    let input = "ls /nonexistent 2> /tmp/error.log";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    // ls /nonexistent should fail
    assert_ne!(result, 0);
}

#[test]
fn test_here_doc() {
    // Test: cat << EOF
    let input = "cat << EOF\ntest content\nEOF";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    assert_eq!(result, 0);
}

#[test]
fn test_multi_pipeline() {
    // Test: echo test | cat | grep test
    let input = "echo test | cat | grep test";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    assert_eq!(result, 0);
}

#[test]
fn test_append_redirection() {
    // Test: echo more >> /tmp/test.txt
    let input = "echo more >> /tmp/test.txt";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    assert_eq!(result, 0);
}

#[test]
fn test_fd_redirection() {
    // Test: echo test 2>&1 (redirect stderr to stdout)
    let input = "echo test 2>&1";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    assert_eq!(result, 0);
}

#[test]
fn test_pipeline_with_redirection() {
    // Test: echo test | grep test > /tmp/result.txt
    let input = "echo test | grep test > /tmp/result.txt";
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut generator = SsaIrGenerator::new();
    let func = generator.generate(ast);
    
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    assert_eq!(result, 0);
}