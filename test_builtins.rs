//! Test program for rs-dash-pro

use rs_dash_pro::modules::builtins::Builtins;
use rs_dash_pro::modules::env::ShellEnv;

fn main() {
    println!("Testing rs-dash-pro builtins...");
    
    let builtins = Builtins::new();
    let mut env = ShellEnv::new();
    
    // Test basic builtins
    println!("\n1. Testing cd/pwd:");
    builtins.execute("cd", &["/tmp".to_string()], &mut env);
    builtins.execute("pwd", &[], &mut env);
    
    println!("\n2. Testing echo:");
    builtins.execute("echo", &["Hello".to_string(), "World".to_string()], &mut env);
    
    println!("\n3. Testing export:");
    builtins.execute("export", &["TEST_VAR=hello".to_string()], &mut env);
    builtins.execute("export", &[], &mut env);
    
    println!("\n4. Testing set:");
    builtins.execute("set", &["-x".to_string()], &mut env);
    builtins.execute("set", &[], &mut env);
    
    println!("\n5. Testing unset:");
    builtins.execute("unset", &["TEST_VAR".to_string()], &mut env);
    
    println!("\n6. Testing true/false:");
    let true_status = builtins.execute("true", &[], &mut env);
    let false_status = builtins.execute("false", &[], &mut env);
    println!("true exit status: {}", true_status);
    println!("false exit status: {}", false_status);
    
    println!("\nAll tests completed!");
}