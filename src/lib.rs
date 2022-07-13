#![no_std]

mod logger;
pub use logger::{log, Atom, Entry, Level, Outputter, Value};

#[cfg(feature = "std")]
mod stdfeatures;
#[cfg(feature = "std")]
pub use stdfeatures::{simple_log, ServiceLogger, StdoutOutputter};
