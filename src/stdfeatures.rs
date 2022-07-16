extern crate std;
use std::cell::RefCell;
use std::string::String;
use std::{println, thread_local};

#[derive(Default)]
pub struct StdoutOutputter {
    buf: String,
}

impl crate::Outputter for StdoutOutputter {
    fn write_str(&mut self, val: &str) {
        self.buf.push_str(val);
    }

    fn endline(&mut self) {
        println!("{}", self.buf);
        self.buf.clear();
    }
}

impl StdoutOutputter {
    pub fn new() -> Self {
        Self::default()
    }
}

thread_local! {
    static STDOUT_LOGGER: RefCell<StdoutOutputter> = RefCell::new(StdoutOutputter::new());
}

pub fn stdout_log<'s>(
    msg: &str,
    level: crate::Level,
    entries: impl Iterator<Item = crate::Entry<'s, 's>>,
) {
    STDOUT_LOGGER.with(|out| {
        crate::log(None, &mut *(out.borrow_mut()), msg, level, entries);
    });
}
