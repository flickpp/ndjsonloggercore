extern crate std;
use std::cell::RefCell;
use std::string::String;
use std::{print, thread_local};

#[derive(Default)]
pub struct StdoutOutputter {
    buf: String,
}

impl crate::Outputter for StdoutOutputter {
    fn write_str(&mut self, val: &str, fin: bool) {
        self.buf.push_str(val);

        if fin {
            print!("{}", self.buf);
            self.buf.clear();
        }
    }
}

impl StdoutOutputter {
    pub fn new() -> Self {
        Self::default()
    }
}

thread_local! {
    static STDOUT_LOGGER: RefCell<StdoutOutputter> = RefCell::new(StdoutOutputter::default());
}

pub fn simple_log(msg: &str, level: crate::Level, entries: &[crate::Entry<'_, '_>]) {
    STDOUT_LOGGER.with(|out| {
        crate::log(None, &mut *(out.borrow_mut()), msg, level, entries);
    });
}
