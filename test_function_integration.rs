// Test for function system with arguments and return values

fn test_function_system() {
    use rs_dash_pro::modules::ssa_ir::{Function, ValueType, Instruction, BasicBlockId};
    use rs_dash_pro::modules::ssa_executor::SsaExecutor;
    use rs_dash_pro::modules::variables::init_variable_system;
    
    // Initialize variable system
    init_variable_system();
    
    println!("=== Testing Function System ===");
    
    // Test 1: Simple function with echo and return
    println!("\nTest 1: Simple function with echo and return");
    let mut func1 = Function::new("greet".to_string(), vec!["name".to_string()]);
    let entry_block = func1.entry_block;
    
    // Create string constant for echo
    let hello_str = func1.create_value(ValueType::String);
    func1.add_instruction(entry_block, Instruction::ConstString("Hello, ".to_string(), hello_str));
    
    // Get first argument ($1)
    let arg1 = func1.create_value(ValueType::String);
    func1.add_instruction(entry_block, Instruction::ConstString("$1".to_string(), arg1));
    
    // Concatenate "Hello, " + $1
    let concat_result = func1.create_value(ValueType::String);
    func1.add_instruction(entry_block, Instruction::Concat(hello_str, arg1, concat_result));
    
    // Call echo builtin
    let echo_args = vec![concat_result];
    let echo_result = func1.create_value(ValueType::ExitStatus);
    func1.add_instruction(entry_block, Instruction::CallBuiltin("echo".to_string(), echo_args, echo_result));
    
    // Return success (0)
    let return_status = func1.create_value(ValueType::ExitStatus);
    func1.add_instruction(entry_block, Instruction::ConstInt(0, return_status));
    func1.add_instruction(entry_block, Instruction::Return(return_status));
    
    // Execute the function
    let mut executor = SsaExecutor::new();
    
    // Create argument value
    let arg_value = executor.values.insert(
        rs_dash_pro::modules::ssa_ir::ValueId(999),
        rs_dash_pro::modules::ssa_executor::ExecValue::String("World".to_string())
    );
    
    // Store function
    executor.functions.insert("greet".to_string(), func1);
    
    // Create a main function that calls greet
    let mut main_func = Function::new("main".to_string(), vec![]);
    let main_entry = main_func.entry_block;
    
    // Create argument for function call
    let world_str = main_func.create_value(ValueType::String);
    main_func.add_instruction(main_entry, Instruction::ConstString("World".to_string(), world_str));
    
    // Call greet function with argument
    let call_result = main_func.create_value(ValueType::ExitStatus);
    let call_args = vec![world_str];
    main_func.add_instruction(main_entry, Instruction::CallFunction("greet".to_string(), call_args, call_result));
    
    // Return the result
    main_func.add_instruction(main_entry, Instruction::Return(call_result));
    
    // Execute main function
    let main_result = executor.execute_function(&main_func);
    println!("Main function execution result: {}", main_result);
    
    // Test 2: Function with local variables
    println!("\nTest 2: Function with local variables");
    let mut func2 = Function::new("test_locals".to_string(), vec![]);
    let func2_entry = func2.entry_block;
    
    // Allocate local variable
    let local_var = func2.create_value(ValueType::String);
    func2.add_instruction(func2_entry, Instruction::AllocVar("my_local".to_string(), local_var));
    
    // Store value in local variable
    let local_value = func2.create_value(ValueType::String);
    func2.add_instruction(func2_entry, Instruction::ConstString("Local value".to_string(), local_value));
    func2.add_instruction(func2_entry, Instruction::Store(local_var, local_value));
    
    // Load and echo the local variable
    let loaded_value = func2.create_value(ValueType::String);
    func2.add_instruction(func2_entry, Instruction::Load(local_var, loaded_value));
    
    let echo_args2 = vec![loaded_value];
    let echo_result2 = func2.create_value(ValueType::ExitStatus);
    func2.add_instruction(func2_entry, Instruction::CallBuiltin("echo".to_string(), echo_args2, echo_result2));
    
    // Return success
    let return_status2 = func2.create_value(ValueType::ExitStatus);
    func2.add_instruction(func2_entry, Instruction::ConstInt(0, return_status2));
    func2.add_instruction(func2_entry, Instruction::Return(return_status2));
    
    // Store and execute function
    executor.functions.insert("test_locals".to_string(), func2);
    
    let mut main_func2 = Function::new("main2".to_string(), vec![]);
    let main2_entry = main_func2.entry_block;
    
    let call_result2 = main_func2.create_value(ValueType::ExitStatus);
    main_func2.add_instruction(main2_entry, Instruction::CallFunction("test_locals".to_string(), vec![], call_result2));
    main_func2.add_instruction(main2_entry, Instruction::Return(call_result2));
    
    let main_result2 = executor.execute_function(&main_func2);
    println!("Test locals function execution result: {}", main_result2);
    
    // Test 3: Recursive function (factorial)
    println!("\nTest 3: Recursive function (factorial)");
    // Note: This is a simplified test - real recursion would need proper SSA IR generation
    // For now, just test that we can call functions recursively in principle
    
    println!("Function system tests completed!");
}

fn main() {
    test_function_system();
}