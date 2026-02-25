use rs_dash_pro::modules::lexer::Lexer;
use rs_dash_pro::modules::parser::Parser;
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
use rs_dash_pro::modules::ssa_executor::SsaExecutor;

fn main() {
    println!("测试增强的SSA IR实现");
    
    // 测试简单的命令
    test_command("echo hello");
    
    // 测试变量赋值
    test_command("VAR=test");
    
    // 测试命令序列
    test_command("echo first; echo second");
    
    // 测试逻辑操作
    test_command("echo hello && echo world");
    
    println!("基本测试完成！");
}

fn test_command(input: &str) {
    println!("\n=== 测试命令: '{}' ===", input);
    
    // 词法分析
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    println!("词法分析结果: {} 个token", tokens.len());
    
    // 语法分析
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(ast) => {
            println!("语法分析成功: {}", ast);
            
            // SSA IR 生成
            let mut generator = SsaIrGenerator::new();
            let ssa_func = generator.generate(ast);
            println!("SSA IR 生成成功");
            
            // 执行
            let mut executor = SsaExecutor::new();
            let exit_status = executor.execute_function(&ssa_func);
            println!("执行结果: 退出状态 = {}", exit_status);
        }
        Err(e) => {
            println!("语法分析失败: {}", e);
        }
    }
}