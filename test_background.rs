// Test for background job control

fn main() {
    // Test 1: Simple background execution
    println!("Test 1: Simple background execution");
    
    // In a real test, we would execute a command like "sleep 2 &"
    // But for now, we'll just test the job control API
    
    use rs_dash_pro::modules::job_control_enhanced::{
        get_enhanced_job_control,
        init_enhanced_job_control,
        execute_with_job_control,
    };
    use rs_dash_pro::modules::env::ShellEnv;
    
    // Initialize job control
    match init_enhanced_job_control() {
        Ok(_) => println!("Job control initialized"),
        Err(e) => println!("Failed to initialize job control: {}", e),
    }
    
    // Create a simple environment
    let env = ShellEnv::new();
    
    // Test executing a command in background
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    match execute_with_job_control("echo", &args, &env, false) {
        Ok(job_id) => println!("Started background job {}: echo arg1 arg2", job_id),
        Err(e) => println!("Failed to start background job: {}", e),
    }
    
    // List jobs
    let job_control = get_enhanced_job_control();
    let jc = job_control.lock().unwrap();
    let jobs = jc.get_all_jobs();
    
    println!("Current jobs:");
    for job in jobs {
        println!("  {}", jc.format_job(job));
    }
}