pub mod colors;
pub mod rich;


// Front Colors
macro_rules! red    { () => { "\x1B[31m" }; }
macro_rules! orange { () => { "\x1B[38;2;255;165;0m" }; }
macro_rules! yellow { () => { "\x1B[38;5;220m" }; }
macro_rules! green  { () => { "\x1B[32m" }; }
macro_rules! cyan   { () => { "\x1B[36m" }; }
macro_rules! blue   { () => { "\x1B[34m" }; }
macro_rules! purple { () => { "\x1B[35m" }; }

// Styles
macro_rules! bold        { () => { "\x1B[1m"  }; }
macro_rules! clean_bold  { () => { "\x1B[22m" }; }

macro_rules! underline       { () => { "\x1b[4m"  }; }
macro_rules! clean_underline { () => { "\x1b[24m" }; }

macro_rules! last_start  { () => { "\x1b[1A\x1b[G" }; } // back to last line's beginning

macro_rules! hyper_start { () => { "\x1B]8;;" }; }
macro_rules! hyper_text  { () => { "\x07"     }; }
macro_rules! hyper_end   { () => { "\x1B]8;;\x07"  }; }

macro_rules! italic_start   { () => { "\x1B3m"  }; }
macro_rules! italic_end     { () => { "\x1B23m"  }; }

// Reset
macro_rules! reset_all   { () => { "\x1B[0m"  }; }
macro_rules! reset_fg    { () => { "\x1B[39m"  }; }
macro_rules! reset_bg    { () => { "\x1B[49m"  }; }


pub const RED: &str = red!();
pub const ORANGE: &str = orange!();
pub const YELLOW: &str = yellow!();
pub const GREEN: &str = green!();
pub const CYAN: &str = cyan!();
pub const BLUE: &str = blue!();
pub const PURPLE: &str = purple!();

pub const BOLD: &str = bold!();
pub const CLEAN_BOLD: &str = clean_bold!();

pub const UNDERLINE: &str = underline!();
pub const CLEAN_UNDERLINE: &str = clean_underline!();

pub const LAST_START: &str = last_start!();

pub const HYPER_START: &str = hyper_start!();
pub const HYPER_TEXT: &str = hyper_text!();
pub const HYPER_END: &str = hyper_end!();

pub const ITALIC_START: &str = italic_start!();
pub const ITALIC_END: &str = italic_end!();

pub const RESET_ALL: &str = reset_all!();
pub const RESET_FG : &str = reset_fg!();
pub const RESET_BG : &str = reset_bg!();


pub(crate) use red;
pub(crate) use orange;
pub(crate) use yellow;
pub(crate) use green;
pub(crate) use cyan;
pub(crate) use blue;
pub(crate) use purple;

pub(crate) use bold;
pub(crate) use clean_bold;

pub(crate) use hyper_start;
pub(crate) use hyper_text;
pub(crate) use hyper_end;

pub(crate) use underline;
pub(crate) use clean_underline;

pub(crate) use italic_start;
pub(crate) use italic_end;

pub(crate) use last_start;

pub(crate) use reset_all;
pub(crate) use reset_bg;
pub(crate) use reset_fg;
