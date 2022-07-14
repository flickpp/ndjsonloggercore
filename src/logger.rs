use crate::conv::{itoa_base10, utoa_base10};

const BUF_SIZE: usize = 24;

pub enum Atom<'a> {
    Float(f64),
    Int(i64),
    Uint(u64),
    String(&'a str),
    Bool(bool),
}

impl<'a> Atom<'a> {
    fn write_value(&self, outputter: &mut impl Outputter, buf: &mut [u8; BUF_SIZE]) {
        match self {
            Atom::Float(f) => {
                let s = populate_buf_with_float(buf, *f);
                outputter.write_str(s, false);
            }
            Atom::Int(i) => outputter.write_str(itoa_base10(buf, *i), false),
            Atom::Uint(u) => outputter.write_str(utoa_base10(buf, *u), false),
            Atom::String(s) => outputter.write_json_string(s),
            Atom::Bool(b) => outputter.write_json_bool(*b),
        }
    }
}

pub enum Value<'s, 'a> {
    Atom(Atom<'a>),
    Array(&'s [Atom<'a>]),
}

pub struct Entry<'s, 'a> {
    pub key: &'static str,
    pub value: Value<'s, 'a>,
}

#[derive(Copy, Clone)]
pub enum Level {
    #[cfg(debug_assertions)]
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            #[cfg(debug_assertions)]
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
        }
    }
}

pub trait Outputter {
    fn write_str(&mut self, val: &str, fin: bool);

    fn write_str_with_escape(&mut self, val: &str) {
        let mut start = 0;
        for (n, c) in val.chars().enumerate() {
            if c == '"' {
                self.write_str(&val[start..n], false);
                self.write_str("\\\"", false);
                start = n + 1;
                continue;
            }

            if c == '\\' {
                self.write_str(&val[start..n], false);
                self.write_str("\\\\", false);
                start = n + 1;
                continue;
            }

            if c == '\n' {
                self.write_str(&val[start..n], false);
                self.write_str("\\n", false);
                start = n + 1;
                continue;
            }

            if c == '\r' {
                self.write_str(&val[start..n], false);
                self.write_str("\\r", false);
                start = n + 1;
                continue;
            }

            if c == '\t' {
                self.write_str(&val[start..n], false);
                self.write_str("\\t", false);
                start = n + 1;
                continue;
            }
        }

        self.write_str(&val[start..], false);
    }

    fn write_json_val(&mut self, val: &str) {
        self.write_str(val, false);
    }

    fn write_json_key(&mut self, key: &str) {
        self.write_json_string(key);
        self.write_str(":", false);
    }

    fn write_json_string(&mut self, s: &str) {
        self.write_str("\"", false);
        self.write_str_with_escape(s);
        self.write_str("\"", false);
    }

    fn write_json_bool(&mut self, b: bool) {
        if b {
            self.write_str("true", false);
        } else {
            self.write_str("false", false);
        }
    }

    fn write_json_end(&mut self) {
        self.write_str("}", true);
    }
}

pub fn log(
    service_name: Option<&'static str>,
    outputter: &mut impl Outputter,
    msg: &str,
    level: Level,
    entries: &[Entry<'_, '_>],
) {
    outputter.write_json_val("{");
    if let Some(service_name) = service_name {
        outputter.write_json_key("service");
        outputter.write_json_string(service_name);
        outputter.write_json_val(",");
    }

    // Timestamp
    #[cfg(feature = "isotimestamp")]
    write_isotimestamp(outputter);

    // Log Level
    outputter.write_json_key("level");
    outputter.write_json_string(level.as_str());
    outputter.write_json_val(",");

    // Message
    outputter.write_json_key("msg");
    outputter.write_json_string(msg);

    for e in entries {
        let mut buf = [0_u8; BUF_SIZE];
        // Comma
        outputter.write_json_val(",");

        // key
        outputter.write_json_key(e.key);

        // value
        match e.value {
            Value::Atom(ref a) => a.write_value(outputter, &mut buf),
            Value::Array(arr) => {
                outputter.write_json_val("[");
                for (n, a) in arr.iter().enumerate() {
                    if n != 0 {
                        outputter.write_json_val(",");
                    }

                    a.write_value(outputter, &mut buf);
                }
                outputter.write_json_val("]");
            }
        }
    }

    outputter.write_json_end();
}

#[cfg(feature = "isotimestamp")]
fn write_isotimestamp(outputter: &mut impl Outputter) {
    use chrono::prelude::*;

    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, false);

    outputter.write_json_key("ts");
    outputter.write_json_string(&ts);
    outputter.write_json_val(",");
}

fn populate_buf_with_float(buf: &mut [u8; BUF_SIZE], _f: f64) -> &str {
    // TODO
    unsafe { core::str::from_utf8_unchecked(buf) }
}

#[cfg(test)]
mod test {
    use super::*;

    extern crate std;
    use std::string::String;

    use random_fast_rng::{FastRng, Random};

    #[derive(Default)]
    struct Output {
        inner: String,
        fin_count: usize,
    }

    impl Outputter for Output {
        fn write_str(&mut self, val: &str, fin: bool) {
            self.inner.push_str(val);
            if fin {
                self.fin_count += 1;
            }
        }
    }

    #[cfg(feature = "isotimestamp")]
    #[test]
    fn write_isotimestamp_() {
        let mut out = Output::default();
        write_isotimestamp(&mut out);
        assert_eq!(&out.inner[..3], "ts:");
    }

    #[test]
    fn outputter_() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        struct LogLine {
            uint: u64,
            iint: i64,
            t: bool,
            f: bool,
            empty_string: String,
            single_quote: String,
            single_newline: String,
            tab_indent: String,
            multi_line: String,
        }

        let mut rng = FastRng::new();
        let uint = rng.gen::<u64>();
        let iint = rng.gen::<i64>();
        let empty_string = String::new();
        let single_quote = String::from("\"");
        let single_newline = String::from("\n");
        let multi_line = String::from("\thello\r\n\tMy name is Bob\r\n\t\"boo\"\r\n\tend");
        let tab_indent = String::from("\thello world");
        let mut out = Output::default();
        let mut buf = [0_u8; BUF_SIZE];

        out.write_json_val("{");

        // uint and iint
        out.write_json_key("uint");
        out.write_str(utoa_base10(&mut buf, uint), false);
        out.write_json_val(",");
        out.write_json_key("iint");
        out.write_str(itoa_base10(&mut buf, iint), false);
        out.write_json_val(",");

        // t and f
        out.write_json_key("t");
        out.write_json_bool(true);
        out.write_json_val(",");
        out.write_json_key("f");
        out.write_json_bool(false);
        out.write_json_val(",");

        // Strings
        out.write_json_key("empty_string");
        out.write_json_string(&empty_string);
        out.write_json_val(",");
        out.write_json_key("single_quote");
        out.write_json_string(&single_quote);
        out.write_json_val(",");
        out.write_json_key("single_newline");
        out.write_json_string(&single_newline);
        out.write_json_val(",");
        out.write_json_key("tab_indent");
        out.write_json_string(&tab_indent);
        out.write_json_val(",");
        out.write_json_key("multi_line");
        out.write_json_string(&multi_line);

        out.write_json_end();

        let log_line: LogLine =
            serde_json::from_str(&out.inner).expect("couldn't deserialize logline, invalid json");

        assert_eq!(
            log_line,
            LogLine {
                uint,
                iint,
                f: false,
                t: true,
                empty_string,
                single_quote,
                single_newline,
                tab_indent,
                multi_line,
            }
        );
    }
}
