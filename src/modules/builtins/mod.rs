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
pub mod bg;            // bg
pub mod cd;            // cd
pub mod command;       // command
pub mod echo;          // echo
pub mod false_cmd;     // false
pub mod fg;            // fg
pub mod getopts;       // getopts
pub mod hash;          // hash
pub mod jobs;          // jobs
pub mod kill;          // kill
pub mod pwd;           // pwd
pub mod read;          // read
pub mod true_cmd;      // true
pub mod type_cmd;      // type
pub mod umask;         // umask
pub mod ulimit;        // ulimit
pub mod wait;          // wait
pub mod printf;        // printf
pub mod local;         // local

// Other builtins
pub mod help;          // help

// Builtins registry and execution
mod registry;
pub use registry::{Builtins, BuiltinRegistry};

// Re-export builtin implementations
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
pub use bg::Bg;
pub use cd::Cd;
pub use command::Command;
pub use echo::Echo;
pub use false_cmd::False;
pub use fg::Fg;
pub use getopts::Getopts;
pub use hash::Hash;
pub use jobs::Jobs;
pub use kill::Kill;
pub use pwd::Pwd;
pub use read::Read;
pub use true_cmd::True;
pub use type_cmd::Type;
pub use umask::Umask;
pub use ulimit::Ulimit;
pub use wait::Wait;
pub use printf::Printf;
pub use local::Local;
pub use help::Help;