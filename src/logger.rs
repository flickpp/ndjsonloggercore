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
                outputter.write_str(s);
            }
            Atom::Int(i) => outputter.write_str(itoa_base10(buf, *i)),
            Atom::Uint(u) => outputter.write_str(utoa_base10(buf, *u)),
            Atom::String(s) => outputter.write_json_string(s),
            Atom::Bool(b) => outputter.write_json_bool(*b),
        }
    }
}

pub enum Value<'s, 'a> {
    Atom(Atom<'a>),
    Array(&'s mut dyn Iterator<Item = Atom<'a>>),
    Optatom(Option<Atom<'a>>),
    Optarray(Option<&'s mut dyn Iterator<Item = Atom<'a>>>),
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
    fn write_str(&mut self, val: &str);
    fn endline(&mut self);

    fn write_str_with_escape(&mut self, val: &str) {
        let mut start = 0;
        for (n, c) in val.chars().enumerate() {
            if c == '"' {
                self.write_str(&val[start..n]);
                self.write_str("\\\"");
                start = n + 1;
                continue;
            }

            if c == '\\' {
                self.write_str(&val[start..n]);
                self.write_str("\\\\");
                start = n + 1;
                continue;
            }

            if c == '\n' {
                self.write_str(&val[start..n]);
                self.write_str("\\n");
                start = n + 1;
                continue;
            }

            if c == '\r' {
                self.write_str(&val[start..n]);
                self.write_str("\\r");
                start = n + 1;
                continue;
            }

            if c == '\t' {
                self.write_str(&val[start..n]);
                self.write_str("\\t");
                start = n + 1;
                continue;
            }
        }

        self.write_str(&val[start..]);
    }

    fn write_json_comma(&mut self) {
        self.write_str(",");
    }

    fn write_json_null(&mut self) {
        self.write_str("null");
    }

    fn write_json_key(&mut self, key: &str) {
        self.write_json_string(key);
        self.write_str(":");
    }

    fn write_json_string(&mut self, s: &str) {
        self.write_str("\"");
        self.write_str_with_escape(s);
        self.write_str("\"");
    }

    fn write_json_bool(&mut self, b: bool) {
        if b {
            self.write_str("true");
        } else {
            self.write_str("false");
        }
    }

    fn write_json_start(&mut self) {
        self.write_str("{");
    }

    fn write_json_end(&mut self) {
        self.write_str("}");
    }

    fn write_json_start_array(&mut self) {
        self.write_str("[");
    }

    fn write_json_end_array(&mut self) {
        self.write_str("]");
    }
}

pub fn log<'s>(
    service_name: Option<&'static str>,
    outputter: &mut impl Outputter,
    msg: &str,
    level: Level,
    entries: impl Iterator<Item = Entry<'s, 's>>,
) {
    outputter.write_json_start();
    if let Some(service_name) = service_name {
        outputter.write_json_key("service");
        outputter.write_json_string(service_name);
        outputter.write_json_comma();
    }

    // Log Level
    outputter.write_json_key("level");
    outputter.write_json_string(level.as_str());
    outputter.write_json_comma();

    // Timestamp
    #[cfg(feature = "isotimestamp")]
    write_isotimestamp(outputter);

    // Message
    outputter.write_json_key("msg");
    outputter.write_json_string(msg);

    for mut e in entries {
        let mut buf = [0_u8; BUF_SIZE];
        // Comma
        outputter.write_json_comma();

        // key
        outputter.write_json_key(e.key);

        // value
        match e.value {
            Value::Atom(ref a) => a.write_value(outputter, &mut buf),
            Value::Array(ref mut arr) => {
                let mut n = 0;
                outputter.write_json_start_array();
                while let Some(a) = arr.next() {
                    if n != 0 {
                        outputter.write_json_comma();
                    }

                    a.write_value(outputter, &mut buf);
                    n += 1;
                }
                outputter.write_json_end_array();
            }
            Value::Optatom(ref oa) => match oa {
                Some(a) => a.write_value(outputter, &mut buf),
                None => outputter.write_json_null(),
            },
            Value::Optarray(ref mut oarr) => match oarr {
                Some(ref mut arr) => {
                    let mut n = 0;
                    outputter.write_json_start_array();
                    while let Some(a) = arr.next() {
                        if n != 0 {
                            outputter.write_json_comma();
                        }

                        a.write_value(outputter, &mut buf);
                        n += 1;
                    }
                    outputter.write_json_end_array();
                }
                None => outputter.write_json_null(),
            },
        }
    }

    outputter.write_json_end();
    outputter.endline();
}

#[cfg(feature = "isotimestamp")]
fn write_isotimestamp(outputter: &mut impl Outputter) {
    use chrono::prelude::*;

    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);

    outputter.write_json_key("ts");
    outputter.write_json_string(&ts);
    outputter.write_json_comma();
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
    use std::vec::Vec;

    use random_fast_rng::{FastRng, Random};

    #[derive(Default)]
    struct Output {
        inner: String,
        fin_count: usize,
    }

    impl Outputter for Output {
        fn write_str(&mut self, val: &str) {
            self.inner.push_str(val);
        }

        fn endline(&mut self) {
            self.fin_count += 1;
        }
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
            null: Option<String>,
            null_array: Option<Vec<String>>,
        }

        let mut rng = FastRng::new();
        let uint = rng.gen::<u64>();
        let iint = rng.gen::<i64>();
        let empty_string = String::new();
        let single_quote = String::from("\"");
        let single_newline = String::from("\n");
        let multi_line = String::from("\thello\r\n\tMy name is Bob\r\n\t\"boo\"\r\n\tend");
        let tab_indent = String::from("\thello world");
        let null = None;
        let null_array = None;
        let mut out = Output::default();
        let mut buf = [0_u8; BUF_SIZE];

        out.write_json_start();

        // uint and iint
        out.write_json_key("uint");
        out.write_str(utoa_base10(&mut buf, uint));
        out.write_json_comma();
        out.write_json_key("iint");
        out.write_str(itoa_base10(&mut buf, iint));
        out.write_json_comma();

        // t and f
        out.write_json_key("t");
        out.write_json_bool(true);
        out.write_json_comma();
        out.write_json_key("f");
        out.write_json_bool(false);
        out.write_json_comma();

        // Strings
        out.write_json_key("empty_string");
        out.write_json_string(&empty_string);
        out.write_json_comma();
        out.write_json_key("single_quote");
        out.write_json_string(&single_quote);
        out.write_json_comma();
        out.write_json_key("single_newline");
        out.write_json_string(&single_newline);
        out.write_json_comma();
        out.write_json_key("tab_indent");
        out.write_json_string(&tab_indent);
        out.write_json_comma();
        out.write_json_key("multi_line");
        out.write_json_string(&multi_line);

        // Null value
        out.write_json_comma();
        out.write_json_key("null");
        out.write_json_null();
        out.write_json_comma();
        out.write_json_key("null_array");
        out.write_json_null();

        out.write_json_end();
        out.endline();

        let log_line: LogLine =
            serde_json::from_str(&out.inner).expect("couldn't deserialize logline, invalid json");

        assert_eq!(out.fin_count, 1);

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
                null,
                null_array,
            }
        );
    }

    #[test]
    fn log_no_entries() {
        #[derive(serde::Deserialize)]
        struct LogLine {
            level: String,
            msg: String,

            #[cfg(feature = "isotimestamp")]
            ts: String,
        }

        let msg = "hello world";
        let level = Level::Info;

        let mut out = Output::default();
        log(None, &mut out, msg, level, [].into_iter());

        let log_line: LogLine =
            serde_json::from_str(&out.inner).expect("couldn't deserialize logline, invalid json");

        assert_eq!(log_line.msg, msg);
        assert_eq!(log_line.level, level.as_str());
        #[cfg(feature = "isotimestamp")]
        assert!(log_line.ts.len() > 0)
    }

    #[test]
    fn log_entries() {
        #[derive(serde::Deserialize)]
        struct LogLine {
            level: String,
            msg: String,

            #[cfg(feature = "isotimestamp")]
            ts: String,

            resource_id: u64,
            healthy: bool,
            some_nums: [i64; 4],
            two_strings: Vec<String>,
            null_array: Option<Vec<String>>,
            opt_array: Option<Vec<String>>,
        }

        let mut rng = FastRng::new();

        let msg = "hello world";
        let level = Level::Info;
        let resource_id = rng.gen::<u64>();
        let healthy = true;
        let some_nums = rng.gen::<[i64; 4]>();
        let two_strings = [String::from(""), String::from("boo")];
        let opt_array = [
            String::from("hello"),
            String::from("goose"),
            String::from("world"),
        ];

        let mut out = Output::default();
        log(
            None,
            &mut out,
            msg,
            level,
            [
                Entry {
                    key: "resource_id",
                    value: Value::Atom(Atom::Uint(resource_id)),
                },
                Entry {
                    key: "healthy",
                    value: Value::Atom(Atom::Bool(healthy)),
                },
                Entry {
                    key: "some_nums",
                    value: Value::Array(&mut some_nums.iter().map(|i| Atom::Int(*i))),
                },
                Entry {
                    key: "two_strings",
                    value: Value::Array(&mut two_strings.iter().map(|s| Atom::String(&s[..]))),
                },
                Entry {
                    key: "null_array",
                    value: Value::Optarray(None),
                },
                Entry {
                    key: "opt_array",
                    value: Value::Optarray(Some(
                        &mut opt_array.iter().map(|s| Atom::String(&s[..])),
                    )),
                },
            ]
            .into_iter(),
        );

        let log_line: LogLine =
            serde_json::from_str(&out.inner).expect("couldn't deserialize logline, invalid json");

        assert_eq!(log_line.msg, msg);
        assert_eq!(log_line.level, level.as_str());
        assert_eq!(log_line.resource_id, resource_id);
        assert_eq!(log_line.healthy, healthy);
        assert_eq!(log_line.some_nums, some_nums);
        assert_eq!(log_line.two_strings, two_strings);
        assert_eq!(log_line.null_array, None);
        assert_eq!(
            log_line.opt_array.as_ref().map(|v| &v[..]),
            Some(&opt_array[..])
        );
        #[cfg(feature = "isotimestamp")]
        assert!(log_line.ts.len() > 0)
    }
}
