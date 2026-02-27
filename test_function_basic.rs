//! Basic function system test

use rs_dash_pro::modules::ssa_ir::{Function, ValueType, Instruction};
use rs_dash_pro::modules::ssa_ir_generator::SsaIrGenerator;
use rs_dash_pro::modules::ssa_executor::SsaExecutor;
use rs_dash_pro::modules::env::ShellEnv;

fn main() {
    println!("Testing basic function system...");
    
    // Create a simple function that echoes "Hello from function!"
    let mut func = Function::new("test_func".to_string(), vec!["arg1".to_string(), "arg2".to_string()]);
    
    // Entry block
    let entry_block = func.entry_block;
    
    // Create a string constant
    let hello_str = func.create_value(ValueType::String);
    func.add_instruction(entry_block, Instruction::ConstString("Hello from function!".to_string(), hello_str));
    
    // Call echo builtin
    let echo_args = vec![hello_str];
    let echo_result = func.create_value(ValueType::ExitStatus);
    func.add_instruction(entry_block, Instruction::CallBuiltin("echo".to_string(), echo_args, echo_result));
    
    // Return success
    let return_status = func.create_value(ValueType::ExitStatus);
    func.add_instruction(entry_block, Instruction::ConstInt(0, return_status));
    func.add_instruction(entry_block, Instruction::Return(return_status));
    
    // Execute the function
    let mut executor = SsaExecutor::new();
    let result = executor.execute_function(&func);
    
    println!("Function execution result: {}", result);
    
    // Test function call instruction
    println!("\nTesting function call instruction...");
    
    // Create a main function that calls test_func
    let mut main_func = Function::new("main".to_string(), vec![]);
    let main_entry = main_func.entry_block;
    
    // Call test_func
    let call_result = main_func.create_value(ValueType::ExitStatus);
    let call_args: Vec<_> = vec![]; // No arguments for now
    
    // Store the function in executor's function table
    executor.functions.insert("test_func".to_string(), func);
    
    // Create CallFunction instruction
    main_func.add_instruction(main_entry, Instruction::CallFunction("test_func".to_string(), call_args, call_result));
    
    // Return the result
    main_func.add_instruction(main_entry, Instruction::Return(call_result));
    
    // Execute main function
    let main_result = executor.execute_function(&main_func);
    println!("Main function execution result: {}", main_result);
}