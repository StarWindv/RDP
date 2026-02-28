//! Built-in shell commands
//! Organized by POSIX categories

// POSIX special builtins (must be built-in)
pub mod break_cmd; // break
pub mod colon; // : (null command)
pub mod continue_cmd; // continue
pub mod dot; // . (source)
pub mod eval; // eval
pub mod exec; // exec
pub mod exit; // exit
pub mod export; // export
pub mod local; // local
pub mod readonly; // readonly
pub mod return_cmd;
pub mod set; // set
pub mod shift; // shift
pub mod times; // times
pub mod trap; // trap
pub mod unset; // unset // return

// POSIX standard utility builtins (usually built-in)
pub mod alias; // alias
pub mod bg; // bg
pub mod cd; // cd
pub mod echo; // echo
pub mod false_cmd; // false
pub mod fg; // fg
pub mod help; // help
pub mod jobs; // jobs
pub mod printenv; // printenv
pub mod pwd; // pwd
pub mod test;
pub mod true_cmd; // true
pub mod wait; // wait // test / [

// TODO: Implement missing builtins
// pub mod command;       // command
// pub mod getopts;       // getopts
// pub mod hash;          // hash
// pub mod kill;          // kill
// pub mod read;          // read
// pub mod type_cmd;      // type
// pub mod umask;         // umask
// pub mod ulimit;        // ulimit
// pub mod printf;        // printf
// pub mod local;         // local

// Builtins registry and execution
mod registry;
pub use registry::{BuiltinRegistry, Builtins};

// Re-export builtin implementations (only for existing modules)
pub use alias::Alias;
pub use bg::Bg;
pub use break_cmd::Break;
pub use cd::Cd;
pub use colon::Colon;
pub use continue_cmd::Continue;
pub use dot::Dot;
pub use echo::Echo;
pub use eval::Eval;
pub use exec::Exec;
pub use exit::Exit;
pub use export::Export;
pub use false_cmd::False;
pub use fg::Fg;
pub use help::Help;
pub use jobs::Jobs;
pub use local::Local;
pub use printenv::PrintEnv;
pub use pwd::Pwd;
pub use readonly::Readonly;
pub use return_cmd::Return;
pub use set::Set;
pub use shift::Shift;
pub use test::TestCommand;
pub use times::Times;
pub use trap::Trap;
pub use true_cmd::True;
pub use unset::Unset;
pub use wait::Wait;
