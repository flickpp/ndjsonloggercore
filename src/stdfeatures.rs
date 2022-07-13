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

pub struct ServiceLogger {
    service_name: Option<&'static str>,
}

impl ServiceLogger {
    pub fn new(service_name: Option<&'static str>) -> Self {
        Self { service_name }
    }

    #[inline(always)]
    pub fn log(
        &self,
        outputter: &mut impl crate::Outputter,
        msg: &str,
        level: crate::Level,
        entries: &[crate::Entry<'_, '_>],
    ) {
        crate::log(self.service_name, outputter, msg, level, entries)
    }
}

const LOGGER: ServiceLogger = ServiceLogger { service_name: None };

thread_local! {
    static STDOUT_LOGGER: RefCell<StdoutOutputter> = RefCell::new(StdoutOutputter::default());
}

pub fn simple_log(msg: &str, level: crate::Level, entries: &[crate::Entry<'_, '_>]) {
    STDOUT_LOGGER.with(|out| {
        LOGGER.log(&mut *(out.borrow_mut()), msg, level, entries);
    });
}
