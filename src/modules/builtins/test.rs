use crate::modules::env::ShellEnv;
use crate::modules::builtins::registry::BuiltinCommand;

pub struct TestCommand;

impl BuiltinCommand for TestCommand {
    fn name(&self) -> &'static str {
        "test"
    }
    
    fn execute(&self, args: &[String], _env: &mut ShellEnv) -> i32 {
        if args.is_empty() {
            return 1; // [ with no args is false
        }

        // For now, support simple numeric comparisons
        // [ $var -lt num ] - less than
        // [ $var -eq num ] - equal
        // [ $var -gt num ] - greater than
        
        if args.len() == 3 {
            let left = &args[0];
            let op = &args[1];
            let right = &args[2];

            // Parse operands as integers
            let left_num: i32 = left.parse().unwrap_or(0);
            let right_num: i32 = right.parse().unwrap_or(0);

            let result = match op.as_str() {
                "-eq" => left_num == right_num,
                "-ne" => left_num != right_num,
                "-lt" => left_num < right_num,
                "-le" => left_num <= right_num,
                "-gt" => left_num > right_num,
                "-ge" => left_num >= right_num,
                "-z" => left.is_empty(),
                "-n" => !left.is_empty(),
                "=" => left == right,
                "!=" => left != right,
                _ => return 1, // Unknown operator
            };

            if result { 0 } else { 1 }
        } else if args.len() == 1 {
            // Single argument: test if string is non-empty
            if args[0].is_empty() {
                1
            } else {
                0
            }
        } else {
            1 // Invalid number of arguments
        }
    }
}
