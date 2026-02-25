//! 测试SSA IR生成器对POSIX Shell语法的覆盖情况

use rs_dash_pro::modules::lexer::Lexer;
use rs_dash_pro::modules::parser::Parser;
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;

fn test_ssa_generation(input: &str) -> Result<(), String> {
    println!("测试: {}", input);
    
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    
    let mut generator = SsaIrGenerator::new();
    let ssa_func = generator.generate(ast);
    
    println!("生成的SSA IR:");
    println!("{}", ssa_func);
    println!("---\n");
    
    Ok(())
}

fn main() -> Result<(), String> {
    println!("=== 测试SSA IR生成器对POSIX Shell语法的覆盖 ===\n");
    
    // 1. 简单命令
    println!("=== 简单命令 ===");
    test_ssa_generation("echo hello world")?;
    test_ssa_generation("ls -l")?;
    
    // 2. 变量赋值
    println!("=== 变量赋值 ===");
    test_ssa_generation("VAR=value")?;
    test_ssa_generation("A=1 B=2 echo test")?;
    
    // 3. 管道
    println!("=== 管道 ===");
    test_ssa_generation("ls | grep txt")?;
    test_ssa_generation("cat file.txt | grep pattern | wc -l")?;
    
    // 4. 逻辑操作
    println!("=== 逻辑操作 ===");
    test_ssa_generation("echo hello && echo world")?;
    test_ssa_generation("false || echo fallback")?;
    
    // 5. 命令列表
    println!("=== 命令列表 ===");
    test_ssa_generation("echo first; echo second")?;
    
    // 6. 复合命令
    println!("=== 复合命令 ===");
    test_ssa_generation("{ echo inside; }")?;
    
    // 7. 子shell
    println!("=== 子shell ===");
    test_ssa_generation("(cd /tmp && ls)")?;
    
    // 8. If语句
    println!("=== If语句 ===");
    test_ssa_generation("if true; then echo yes; fi")?;
    test_ssa_generation("if false; then echo yes; else echo no; fi")?;
    
    // 9. While循环
    println!("=== While循环 ===");
    test_ssa_generation("while true; do echo loop; done")?;
    
    // 10. Until循环
    println!("=== Until循环 ===");
    test_ssa_generation("until false; do echo loop; done")?;
    
    // 11. For循环
    println!("=== For循环 ===");
    test_ssa_generation("for i in 1 2 3; do echo $i; done")?;
    
    // 12. Case语句
    println!("=== Case语句 ===");
    test_ssa_generation("case $var in a) echo a;; b) echo b;; esac")?;
    
    // 13. 函数定义
    println!("=== 函数定义 ===");
    test_ssa_generation("myfunc() { echo hello; }")?;
    
    // 14. 重定向
    println!("=== 重定向 ===");
    test_ssa_generation("echo test > file.txt")?;
    test_ssa_generation("cat < file.txt")?;
    
    // 15. 后台执行
    println!("=== 后台执行 ===");
    test_ssa_generation("sleep 10 &")?;
    
    // 16. 命令替换
    println!("=== 命令替换 ===");
    test_ssa_generation("echo $(date)")?;
    
    // 17. 参数扩展
    println!("=== 参数扩展 ===");
    test_ssa_generation("echo ${VAR:-default}")?;
    
    println!("=== 测试完成 ===");
    Ok(())
}