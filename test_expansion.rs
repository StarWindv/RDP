use rs_dash_pro::modules::env::ShellEnv;

fn main() {
    let env = ShellEnv::new();
    
    // Test 1: Regular arguments should NOT be expanded
    let arg1 = "-L";
    let expanded1 = env.expand_variables(arg1);
    println!("Test 1: '{}' -> '{}' (should be '-L')", arg1, expanded1);
    assert_eq!(expanded1, "-L", "Regular arguments should not be expanded");
    
    // Test 2: Regular number arguments should NOT be expanded
    let arg2 = "0";
    let expanded2 = env.expand_variables(arg2);
    println!("Test 2: '{}' -> '{}' (should be '0')", arg2, expanded2);
    assert_eq!(expanded2, "0", "Number arguments should not be expanded");
    
    // Test 3: Variable arguments SHOULD be expanded
    env.set_var("TEST_VAR".to_string(), "test_value".to_string());
    let arg3 = "$TEST_VAR";
    let expanded3 = env.expand_variables(arg3);
    println!("Test 3: '{}' -> '{}' (should be 'test_value')", arg3, expanded3);
    assert_eq!(expanded3, "test_value", "Variable arguments should be expanded");
    
    // Test 4: Mixed string with variable should be expanded
    let arg4 = "prefix_$TEST_VAR_suffix";
    let expanded4 = env.expand_variables(arg4);
    println!("Test 4: '{}' -> '{}' (should be 'prefix_test_value_suffix')", arg4, expanded4);
    assert_eq!(expanded4, "prefix_test_value_suffix", "Mixed string should expand variable part");
    
    // Test 5: Special variable $? should be expanded
    let arg5 = "$?";
    let expanded5 = env.expand_variables(arg5);
    println!("Test 5: '{}' -> '{}' (should be '0')", arg5, expanded5);
    assert_eq!(expanded5, "0", "Special variable $? should be expanded");
    
    println!("All tests passed!");
}