use rs_dash_pro::*;
use std::process::Command;

fn test_builtin(name: &str, args: &[&str], env: &mut ShellEnv) -> i32 {
    let builtins = Builtins;
    let args_str: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    builtins.execute(name, &args_str, env)
}

fn main() {
    let mut env = ShellEnv::new();
    
    println!("=== Testing POSIX Special Builtins ===");
    println!();
    
    // Test : (colon)
    println!("Testing ':' (colon):");
    let status = test_builtin(":", &[], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    // Test . (dot) - source a file
    println!("Testing '.' (dot):");
    let status = test_builtin(".", &["test_source.sh"], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    // Test eval
    println!("Testing 'eval':");
    let status = test_builtin("eval", &["echo", "Hello", "from", "eval"], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    // Test export
    println!("Testing 'export':");
    let status = test_builtin("export", &["TEST_VAR=exported_value"], &mut env);
    println!("Exit status: {}", status);
    println!("TEST_VAR = {:?}", env.get_var("TEST_VAR"));
    println!();
    
    // Test readonly
    println!("Testing 'readonly':");
    let status = test_builtin("readonly", &["READONLY_VAR=readonly_value"], &mut env);
    println!("Exit status: {}", status);
    println!("READONLY_VAR = {:?}", env.get_var("READONLY_VAR"));
    println!();
    
    // Test set
    println!("Testing 'set':");
    let status = test_builtin("set", &[], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    // Test shift
    println!("Testing 'shift':");
    let status = test_builtin("shift", &[], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    // Test times
    println!("Testing 'times':");
    let status = test_builtin("times", &[], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    // Test trap
    println!("Testing 'trap':");
    let status = test_builtin("trap", &["'echo SIGINT'", "INT"], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    // Test unset
    println!("Testing 'unset':");
    env.set_var("UNSET_VAR".to_string(), "test_value".to_string());
    println!("Before unset: UNSET_VAR = {:?}", env.get_var("UNSET_VAR"));
    let status = test_builtin("unset", &["UNSET_VAR"], &mut env);
    println!("Exit status: {}", status);
    println!("After unset: UNSET_VAR = {:?}", env.get_var("UNSET_VAR"));
    println!();
    
    // Test break and continue (need loop context)
    println!("Testing 'break':");
    let status = test_builtin("break", &[], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    println!("Testing 'continue':");
    let status = test_builtin("continue", &[], &mut env);
    println!("Exit status: {}", status);
    println!();
    
    println!("=== Builtin tests completed ===");
}