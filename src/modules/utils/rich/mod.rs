pub mod colors;
pub mod rich;

macro_rules! red    { () => { "\x1B[31m" }; }
macro_rules! orange { () => { "\x1B[38;2;255;165;0m" }; }
macro_rules! yellow { () => { "\x1B[38;5;220m" }; }
macro_rules! green  { () => { "\x1B[32m" }; }
macro_rules! cyan   { () => { "\x1B[36m" }; }
macro_rules! blue   { () => { "\x1B[34m" }; }
macro_rules! purple { () => { "\x1B[35m" }; }
macro_rules! reset  { () => { "\x1B[0m" }; }
macro_rules! bold   { () => { "\x1B[1m" }; }
macro_rules! hyper_start { () => { "\x1B]8;;" }; }
macro_rules! hyper_text  { () => { "\x07" }; }
macro_rules! hyper_end   { () => { "\x1B]8;;\x07" }; }
macro_rules! underline   { () => { "\x1b[4m" }; }
macro_rules! last_start { () => { "\x1b[1A\x1b[G" }; }

pub const RED: &str = red!();
pub const ORANGE: &str = orange!();
pub const YELLOW: &str = yellow!();
pub const GREEN: &str = green!();
pub const CYAN: &str = cyan!();
pub const BLUE: &str = blue!();
pub const PURPLE: &str = purple!();
pub const RESET: &str = reset!();
pub const BOLD: &str = bold!();
pub const HYPER_START: &str = hyper_start!();
pub const HYPER_TEXT: &str = hyper_text!();
pub const HYPER_END: &str = hyper_end!();
pub const UNDERLINE: &str = underline!();
pub const LAST_START: &str = last_start!();

pub(crate) use red;
pub(crate) use orange;
pub(crate) use yellow;
pub(crate) use green;
pub(crate) use cyan;
pub(crate) use blue;
pub(crate) use purple;
pub(crate) use reset;
pub(crate) use bold;
pub(crate) use hyper_start;
pub(crate) use hyper_text;
pub(crate) use hyper_end;
pub(crate) use underline;
pub(crate) use last_start;
