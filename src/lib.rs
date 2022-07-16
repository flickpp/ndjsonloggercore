#![no_std]

mod logger;
pub use logger::{log, Atom, Entry, Level, Outputter, Value};

// Primative to string functions
mod conv;

#[cfg(feature = "std")]
mod stdfeatures;
#[cfg(feature = "std")]
pub use stdfeatures::{stdout_log, StdoutOutputter};
