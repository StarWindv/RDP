//! Shell options system
//! Implements POSIX shell options with inheritance and persistence

use std::collections::HashMap;

/// Shell options as defined by POSIX
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellOption {
    // Single-letter options (set -o)
    ErrExit, // -e: Exit immediately if a command exits with non-zero status
    NoUnset, // -u: Treat unset variables as an error
    XTrace,  // -x: Print commands and their arguments as they are executed
    NoExec,  // -n: Read commands but do not execute them
    Verbose, // -v: Print shell input lines as they are read

    // Long options (set -o option)
    AllExport,   // allexport: Automatically export all defined variables
    IgnoreEof,   // ignoreeof: Don't exit on EOF (require 'exit' or Ctrl+D)
    Interactive, // interactive: Shell is interactive
    Monitor,     // monitor: Enable job control
    NoClobber,   // noclobber: Prevent overwriting existing files with >
    NoGlob,      // noglob: Disable pathname expansion
    NoLog,       // nolog: Don't save function definitions in history
    Notify,      // notify: Report job termination status immediately
    Physical,    // physical: Use physical directory structure for cd/pwd
    Posix,       // posix: Change behavior to match POSIX standard
    Vi,          // vi: Use vi-style command line editing
}

impl ShellOption {
    /// Get the single-letter option character
    pub fn short_name(&self) -> Option<char> {
        match self {
            ShellOption::ErrExit => Some('e'),
            ShellOption::NoUnset => Some('u'),
            ShellOption::XTrace => Some('x'),
            ShellOption::NoExec => Some('n'),
            ShellOption::Verbose => Some('v'),
            _ => None,
        }
    }

    /// Get the long option name
    pub fn long_name(&self) -> &'static str {
        match self {
            ShellOption::ErrExit => "errexit",
            ShellOption::NoUnset => "nounset",
            ShellOption::XTrace => "xtrace",
            ShellOption::NoExec => "noexec",
            ShellOption::Verbose => "verbose",
            ShellOption::AllExport => "allexport",
            ShellOption::IgnoreEof => "ignoreeof",
            ShellOption::Interactive => "interactive",
            ShellOption::Monitor => "monitor",
            ShellOption::NoClobber => "noclobber",
            ShellOption::NoGlob => "noglob",
            ShellOption::NoLog => "nolog",
            ShellOption::Notify => "notify",
            ShellOption::Physical => "physical",
            ShellOption::Posix => "posix",
            ShellOption::Vi => "vi",
        }
    }

    /// Get the default value for the option
    pub fn default_value(&self) -> bool {
        match self {
            ShellOption::Interactive => true, // Default to interactive in interactive shells
            ShellOption::Monitor => true,     // Default to job control enabled
            _ => false,
        }
    }

    /// Get description of the option
    pub fn description(&self) -> &'static str {
        match self {
            ShellOption::ErrExit => "Exit immediately if a command exits with non-zero status",
            ShellOption::NoUnset => "Treat unset variables as an error when substituting",
            ShellOption::XTrace => "Print commands and their arguments as they are executed",
            ShellOption::NoExec => "Read commands but do not execute them",
            ShellOption::Verbose => "Print shell input lines as they are read",
            ShellOption::AllExport => "Automatically export all subsequently defined variables",
            ShellOption::IgnoreEof => "Ignore EOF (require 'exit' or Ctrl+D to exit)",
            ShellOption::Interactive => "Shell is interactive",
            ShellOption::Monitor => "Enable job control",
            ShellOption::NoClobber => "Prevent overwriting existing files with redirection",
            ShellOption::NoGlob => "Disable pathname expansion (globbing)",
            ShellOption::NoLog => "Don't save function definitions in history",
            ShellOption::Notify => "Report job termination status immediately",
            ShellOption::Physical => "Use physical directory structure for cd/pwd",
            ShellOption::Posix => "Change behavior to match POSIX standard",
            ShellOption::Vi => "Use vi-style command line editing",
        }
    }

    /// Parse a short option character
    pub fn from_short(ch: char) -> Option<Self> {
        match ch {
            'e' => Some(ShellOption::ErrExit),
            'u' => Some(ShellOption::NoUnset),
            'x' => Some(ShellOption::XTrace),
            'n' => Some(ShellOption::NoExec),
            'v' => Some(ShellOption::Verbose),
            _ => None,
        }
    }

    /// Parse a long option name
    pub fn from_long(name: &str) -> Option<Self> {
        match name {
            "errexit" => Some(ShellOption::ErrExit),
            "nounset" => Some(ShellOption::NoUnset),
            "xtrace" => Some(ShellOption::XTrace),
            "noexec" => Some(ShellOption::NoExec),
            "verbose" => Some(ShellOption::Verbose),
            "allexport" => Some(ShellOption::AllExport),
            "ignoreeof" => Some(ShellOption::IgnoreEof),
            "interactive" => Some(ShellOption::Interactive),
            "monitor" => Some(ShellOption::Monitor),
            "noclobber" => Some(ShellOption::NoClobber),
            "noglob" => Some(ShellOption::NoGlob),
            "nolog" => Some(ShellOption::NoLog),
            "notify" => Some(ShellOption::Notify),
            "physical" => Some(ShellOption::Physical),
            "posix" => Some(ShellOption::Posix),
            "vi" => Some(ShellOption::Vi),
            _ => None,
        }
    }
}

/// Shell options manager
#[derive(Debug, Clone)]
pub struct ShellOptions {
    options: HashMap<ShellOption, bool>,
    positional_params: Vec<String>,
}

impl ShellOptions {
    /// Create new shell options with defaults
    pub fn new() -> Self {
        let mut options = HashMap::new();

        // Set default values
        for opt in &[
            ShellOption::ErrExit,
            ShellOption::NoUnset,
            ShellOption::XTrace,
            ShellOption::NoExec,
            ShellOption::Verbose,
            ShellOption::AllExport,
            ShellOption::IgnoreEof,
            ShellOption::Interactive,
            ShellOption::Monitor,
            ShellOption::NoClobber,
            ShellOption::NoGlob,
            ShellOption::NoLog,
            ShellOption::Notify,
            ShellOption::Physical,
            ShellOption::Posix,
            ShellOption::Vi,
        ] {
            options.insert(*opt, opt.default_value());
        }

        Self {
            options,
            positional_params: Vec::new(),
        }
    }

    /// Check if an option is enabled
    pub fn is_enabled(&self, option: ShellOption) -> bool {
        *self.options.get(&option).unwrap_or(&false)
    }

    /// Set an option
    pub fn set_option(&mut self, option: ShellOption, enabled: bool) {
        self.options.insert(option, enabled);
    }

    /// Toggle an option
    pub fn toggle_option(&mut self, option: ShellOption) -> bool {
        let current = self.is_enabled(option);
        let new = !current;
        self.set_option(option, new);
        new
    }

    /// Set positional parameters
    pub fn set_positional_params(&mut self, params: Vec<String>) {
        self.positional_params = params;
    }

    /// Get positional parameters
    pub fn get_positional_params(&self) -> &[String] {
        &self.positional_params
    }

    /// Shift positional parameters
    pub fn shift_positional_params(&mut self, n: usize) -> Vec<String> {
        if n == 0 || self.positional_params.is_empty() {
            return Vec::new();
        }

        let shift_count = std::cmp::min(n, self.positional_params.len());
        let shifted: Vec<String> = self.positional_params.drain(0..shift_count).collect();

        // If all params were shifted, add an empty string
        if self.positional_params.is_empty() {
            self.positional_params.push(String::new());
        }

        shifted
    }

    /// Get all options as a string for display
    pub fn format_options(&self) -> String {
        let mut result = String::new();

        // Format short options
        for (opt, enabled) in &self.options {
            if let Some(ch) = opt.short_name() {
                if *enabled {
                    result.push(ch);
                }
            }
        }

        result
    }

    /// Parse and apply options from command line arguments
    pub fn apply_options(&mut self, args: &[String]) -> Result<usize, String> {
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];

            if arg == "--" {
                // End of options
                i += 1;
                break;
            }

            if arg.starts_with('-') && !arg.starts_with("--") && arg != "-" && arg != "-o" {
                // Handle short options: -e, -u, -x, etc.
                for ch in arg[1..].chars() {
                    match ShellOption::from_short(ch) {
                        Some(opt) => {
                            self.set_option(opt, true);
                        }
                        None => {
                            return Err(format!("set: invalid option -{}", ch));
                        }
                    }
                }
            } else if arg.starts_with('+') && arg != "+o" {
                // Clear options: +e, +u, +x, etc.
                for ch in arg[1..].chars() {
                    match ShellOption::from_short(ch) {
                        Some(opt) => {
                            self.set_option(opt, false);
                        }
                        None => {
                            return Err(format!("set: invalid option +{}", ch));
                        }
                    }
                }
            } else if arg == "-o" {
                // Long option: -o option_name
                if i + 1 >= args.len() {
                    return Err("set: -o requires an argument".to_string());
                }

                let opt_name = &args[i + 1];
                match ShellOption::from_long(opt_name) {
                    Some(opt) => {
                        self.set_option(opt, true);
                    }
                    None => {
                        return Err(format!("set: invalid option name: {}", opt_name));
                    }
                }

                i += 1; // Skip the option name
            } else if arg == "+o" {
                // Clear long option: +o option_name
                if i + 1 >= args.len() {
                    return Err("set: +o requires an argument".to_string());
                }

                let opt_name = &args[i + 1];
                match ShellOption::from_long(opt_name) {
                    Some(opt) => {
                        self.set_option(opt, false);
                    }
                    None => {
                        return Err(format!("set: invalid option name: {}", opt_name));
                    }
                }

                i += 1; // Skip the option name
            } else {
                // Not an option, start of positional parameters
                break;
            }

            i += 1;
        }

        Ok(i)
    }

    /// Create a child options instance (for subshells)
    pub fn create_child(&self) -> Self {
        self.clone()
    }
}

/// Global shell options instance
lazy_static::lazy_static! {
    static ref SHELL_OPTIONS: std::sync::Mutex<ShellOptions> =
        std::sync::Mutex::new(ShellOptions::new());
}

/// Get global shell options
pub fn get_options() -> std::sync::MutexGuard<'static, ShellOptions> {
    SHELL_OPTIONS.lock().unwrap()
}

/// Initialize shell options
pub fn init_options() {
    // Already initialized by lazy_static
}

/// Check if errexit option is enabled
pub fn errexit_enabled() -> bool {
    get_options().is_enabled(ShellOption::ErrExit)
}

/// Check if nounset option is enabled
pub fn nounset_enabled() -> bool {
    get_options().is_enabled(ShellOption::NoUnset)
}

/// Check if xtrace option is enabled
pub fn xtrace_enabled() -> bool {
    get_options().is_enabled(ShellOption::XTrace)
}

/// Check if noexec option is enabled
pub fn noexec_enabled() -> bool {
    get_options().is_enabled(ShellOption::NoExec)
}

/// Check if verbose option is enabled
pub fn verbose_enabled() -> bool {
    get_options().is_enabled(ShellOption::Verbose)
}
