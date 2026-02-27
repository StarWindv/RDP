// Integration test for job control

fn main() {
    println!("Job Control Integration Test");
    
    // Initialize job control
    use rs_dash_pro::modules::job_control_enhanced::init_enhanced_job_control;
    
    match init_enhanced_job_control() {
        Ok(_) => println!("Job control initialized successfully"),
        Err(e) => println!("Failed to initialize job control: {}", e),
    }
    
    // Test built-in jobs command
    println!("\nTesting 'jobs' builtin:");
    use rs_dash_pro::modules::builtins::Builtins;
    use rs_dash_pro::modules::env::ShellEnv;
    
    let builtins = Builtins::new();
    let mut env = ShellEnv::new();
    
    // Execute jobs command
    let status = builtins.execute("jobs", &[], &mut env);
    println!("jobs command exit status: {}", status);
    
    // Test signal handling
    println!("\nTesting signal handling:");
    use rs_dash_pro::modules::ssa_ir::{
        Function, ValueType, Instruction, IrBuilder
    };
    
    // Create a simple function with a trap handler
    let mut builder = IrBuilder::new();
    builder.begin_function("test_trap".to_string(), Vec::new());
    
    // Create a signal value (SIGINT = 2)
    let signal_val = builder.create_value(ValueType::Integer).unwrap();
    builder.add_instruction(Instruction::ConstInt(2, signal_val));
    
    // Create a handler block
    let handler_block = builder.create_block_with_label("sigint_handler".to_string()).unwrap();
    builder.set_current_block(handler_block);
    
    // In handler: print message and return
    let message_val = builder.create_value(ValueType::String).unwrap();
    builder.add_instruction(Instruction::ConstString("SIGINT received!".to_string(), message_val));
    
    let echo_args = vec![message_val];
    let echo_result = builder.create_value(ValueType::ExitStatus).unwrap();
    builder.add_instruction(Instruction::CallBuiltin("echo".to_string(), echo_args, echo_result));
    
    builder.add_instruction(Instruction::Return(echo_result));
    
    // Back to main block
    let main_block = builder.current_block().unwrap();
    builder.set_current_block(main_block);
    
    // Set trap
    builder.add_instruction(Instruction::Trap(signal_val, handler_block));
    
    // Main code: just exit
    let exit_val = builder.create_value(ValueType::ExitStatus).unwrap();
    builder.add_instruction(Instruction::ConstInt(0, exit_val));
    builder.add_instruction(Instruction::Return(exit_val));
    
    let func = builder.end_function().unwrap();
    println!("Created function with trap handler: {}", func.name);
    
    // Execute the function
    use rs_dash_pro::modules::ssa_executor::SsaExecutor;
    let mut executor = SsaExecutor::new();
    let status = executor.execute_function(&func);
    println!("Function execution status: {}", status);
    
    // Try to deliver SIGINT
    println!("\nDelivering SIGINT signal:");
    executor.deliver_signal(2, &func);
    
    println!("\nIntegration test completed.");
}