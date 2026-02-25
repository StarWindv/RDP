use rs_dash_pro::modules::lexer::Lexer;
use rs_dash_pro::modules::parser::Parser;
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
use rs_dash_pro::modules::ssa_executor::SsaExecutor;

fn test_ssa_workflow(input: &str) -> Result<i32, String> {
    println!("=== 测试: {} ===", input);
    
    // 1. Lexical analysis
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    println!("词法分析: {} tokens", tokens.len());
    
    // 2. Parsing
    let mut parser = Parser::new(input);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    println!("语法分析: {}", ast);
    
    // 3. SSA IR generation
    let mut generator = SsaIrGenerator::new();
    let ssa_func = generator.generate(ast);
    println!("SSA IR生成:");
    println!("{}", ssa_func);
    
    // 4. SSA execution
    let mut executor = SsaExecutor::new();
    let exit_status = executor.execute_function(&ssa_func);
    println!("执行结果: {}\n", exit_status);
    
    Ok(exit_status)
}

fn main() -> Result<(), String> {
    println!("=== 测试SSA架构完整工作流 ===\n");
    
    // 测试简单命令
    test_ssa_workflow("echo hello")?;
    
    // 测试变量赋值
    test_ssa_workflow("VAR=test")?;
    
    // 测试命令序列
    test_ssa_workflow("echo first; echo second")?;
    
    // 测试逻辑操作
    test_ssa_workflow("echo hello && echo world")?;
    
    // 测试管道（基本）
    test_ssa_workflow("echo test | cat")?;
    
    println!("=== 所有测试完成 ===");
    Ok(())
}