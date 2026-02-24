//! Built-in shell commands
//! Organized by POSIX categories

// POSIX special builtins (must be built-in)
pub mod dot;           // . (source)
pub mod colon;         // : (null command)
pub mod break_cmd;     // break
pub mod continue_cmd;  // continue
pub mod eval;          // eval
pub mod exec;          // exec
pub mod exit;          // exit
pub mod export;        // export
pub mod readonly;      // readonly
pub mod set;           // set
pub mod shift;         // shift
pub mod times;         // times
pub mod trap;          // trap
pub mod unset;         // unset

// POSIX standard utility builtins (usually built-in)
pub mod alias;         // alias
pub mod cd;            // cd
pub mod echo;          // echo
pub mod false_cmd;     // false
pub mod pwd;           // pwd
pub mod true_cmd;      // true
pub mod help;          // help

// TODO: Implement missing builtins
// pub mod bg;            // bg
// pub mod command;       // command
// pub mod fg;            // fg
// pub mod getopts;       // getopts
// pub mod hash;          // hash
// pub mod jobs;          // jobs
// pub mod kill;          // kill
// pub mod read;          // read
// pub mod type_cmd;      // type
// pub mod umask;         // umask
// pub mod ulimit;        // ulimit
// pub mod wait;          // wait
// pub mod printf;        // printf
// pub mod local;         // local

// Builtins registry and execution
mod registry;
pub use registry::{Builtins, BuiltinRegistry};

// Re-export builtin implementations (only for existing modules)
pub use dot::Dot;
pub use colon::Colon;
pub use break_cmd::Break;
pub use continue_cmd::Continue;
pub use eval::Eval;
pub use exec::Exec;
pub use exit::Exit;
pub use export::Export;
pub use readonly::Readonly;
pub use set::Set;
pub use shift::Shift;
pub use times::Times;
pub use trap::Trap;
pub use unset::Unset;
pub use alias::Alias;
pub use cd::Cd;
pub use echo::Echo;
pub use false_cmd::False;
pub use pwd::Pwd;
pub use true_cmd::True;
pub use help::Help;