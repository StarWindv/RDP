//! Simple test for pipeline functionality

#[test]
fn test_process_manager_basic() {
    use crate::modules::process_manager::ProcessManager;
    
    let pm = ProcessManager::new();
    
    // Test executing a simple command
    let result = pm.execute("echo", &["hello".to_string()]);
    assert!(result.is_ok());
    
    // Test creating a pipe
    let pipe_result = pm.create_pipe();
    assert!(pipe_result.is_ok());
}

#[test]
fn test_pipeline_executor_basic() {
    use crate::modules::pipeline_enhanced::PipelineExecutor;
    
    let executor = PipelineExecutor::new();
    
    // Test single command
    let commands = vec![
        ("echo".to_string(), vec!["test".to_string()]),
    ];
    
    let result = executor.execute_pipeline(&commands);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_parse_redirection() {
    use crate::modules::pipeline_enhanced::parse_redirection;
    
    // Test basic redirections
    assert_eq!(
        parse_redirection("> output.txt"),
        Some((1, "output.txt".to_string(), false, false))
    );
    
    assert_eq!(
        parse_redirection(">> output.txt"),
        Some((1, "output.txt".to_string(), true, false))
    );
    
    assert_eq!(
        parse_redirection("2> error.txt"),
        Some((1, "error.txt".to_string(), false, true))
    );
    
    // Test invalid redirections
    assert_eq!(parse_redirection(""), None);
    assert_eq!(parse_redirection(">"), None);
}