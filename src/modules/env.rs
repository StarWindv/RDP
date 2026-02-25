/// Expand variables in a string
    pub fn expand_variables(&self, input: &str) -> String {
        println!("DEBUG EXPAND: expanding '{}'", input);
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '$' {
                if let Some(&next) = chars.peek() {
                    if next == '{' {
                        // ${var} syntax
                        chars.next(); // Skip {
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '}' {
                                chars.next(); // Skip }
                                break;
                            }
                            var_name.push(c);
                            chars.next();
                        }
                        
                        println!("DEBUG EXPAND: found ${{{}}}", var_name);
                        result.push_str(&self.expand_variable(&var_name));
                    } else if next.is_ascii_alphanumeric() || next == '_' || next == '?' || next == '$' || next == '0' || next == '*' || next == '@' || next == '#' {
                        // $var syntax or special variables
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_alphanumeric() || c == '_' || c == '?' || c == '$' || c == '0' || c == '*' || c == '@' || c == '#' {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        
                        println!("DEBUG EXPAND: found ${}", var_name);
                        result.push_str(&self.expand_variable(&var_name));
                    } else {
                        // Just a literal $
                        result.push(c);
                    }
                } else {
                    // Just a literal $ at end of string
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }
        
        println!("DEBUG EXPAND: result = '{}'", result);
        result
    }
    
    /// Expand a single variable
    fn expand_variable(&self, var_name: &str) -> String {
        match var_name {
            "?" => {
                println!("DEBUG EXPAND: special variable $?, value = {}", self.exit_status);
                self.exit_status.to_string()
            }
            "$" => {
                // Process ID
                let pid = std::process::id();
                println!("DEBUG EXPAND: special variable $$, value = {}", pid);
                pid.to_string()
            }
            "0" => {
                // Shell name
                println!("DEBUG EXPAND: special variable $0, value = rs-dash-pro");
                "rs-dash-pro".to_string()
            }
            "*" => {
                // All positional parameters
                let all_params = self.positional_params.join(" ");
                println!("DEBUG EXPAND: special variable $*, value = '{}'", all_params);
                all_params
            }
            "@" => {
                // All positional parameters (quoted)
                let all_params = self.positional_params.join(" ");
                println!("DEBUG EXPAND: special variable $@, value = '{}'", all_params);
                all_params
            }
            "#" => {
                // Number of positional parameters
                let count = self.positional_params.len();
                println!("DEBUG EXPAND: special variable $#, value = {}", count);
                count.to_string()
            }
            _ => {
                // Check if it's a positional parameter $1, $2, etc.
                if let Ok(index) = var_name.parse::<usize>() {
                    if let Some(param) = self.get_positional_param(index) {
                        println!("DEBUG EXPAND: positional parameter ${}, value = '{}'", index, param);
                        param.clone()
                    } else {
                        println!("DEBUG EXPAND: positional parameter ${} not set", index);
                        String::new()
                    }
                } else if let Some(value) = self.get_var(var_name) {
                    println!("DEBUG EXPAND: value = '{}'", value);
                    value.clone()
                } else {
                    println!("DEBUG EXPAND: variable not found");
                    String::new()
                }
            }
        }
    }