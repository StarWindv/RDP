// Test script for rs-dash-pro
fn main() {
    // Test basic echo
    println!("Testing basic echo command...");
    let output = std::process::Command::new("./target/debug/rs-dash-pro")
        .arg("-c")
        .arg("echo hello world")
        .output()
        .expect("Failed to execute command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Test interactive mode
    println!("\nTesting interactive mode...");
    let mut child = std::process::Command::new("./target/debug/rs-dash-pro")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");
    
    // Send echo command
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        writeln!(stdin, "echo test from interactive").unwrap();
        writeln!(stdin, "exit").unwrap();
    }
    
    let output = child.wait_with_output().expect("Failed to wait for child");
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}