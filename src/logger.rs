pub enum Atom<'a> {
    Float(f64),
    Int(i64),
    Uint(u64),
    String(&'a str),
    Bool(bool),
}

impl<'a> Atom<'a> {
    fn write_value(&self, outputter: &mut impl Outputter, buf: &mut [u8]) {
        match self {
            Atom::Float(f) => {
                let s = populate_buf_with_float(buf, *f);
                outputter.write_str(s, false);
            }
            Atom::Int(i) => {
                let s = populate_buf_with_int(buf, *i);
                outputter.write_str(s, false);
            }
            Atom::Uint(i) => {
                let s = populate_buf_with_uint(buf, *i);
                outputter.write_str(s, false);
            }
            Atom::String(s) => outputter.write_json_string(s),
            Atom::Bool(b) => outputter.write_json_bool(*b),
        }
    }
}

pub enum Value<'s, 'a> {
    Atom(Atom<'a>),
    Array(&'s [Atom<'a>]),
    OptionAtom(Atom<'a>),
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

    fn write_json_val(&mut self, val: &str) {
        self.write_str(val, false);
    }

    fn write_json_key(&mut self, key: &str) {
        self.write_json_string(key);
        self.write_str(":", false);
    }

    fn write_json_string(&mut self, s: &str) {
        self.write_str("\"", false);
        self.write_str(s, false);
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
        self.write_str("}\n", true);
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

    // TimestampC
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
        let mut buf = [0_u8; 16];
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
            Value::OptionAtom(_) => {}
        }
    }

    outputter.write_json_end();
}

#[cfg(feature = "isotimestamp")]
fn write_isotimestamp(outputter: &mut impl Outputter) {
    use chrono::prelude::*;

    let ts = Utc::now().to_rfc3339();

    outputter.write_json_key("ts");
    outputter.write_json_string(&ts);
    outputter.write_json_val(",");
}

fn populate_buf_with_float(buf: &mut [u8], _f: f64) -> &str {
    // TODO
    unsafe { core::str::from_utf8_unchecked(buf) }
}

fn populate_buf_with_int(buf: &mut [u8], val: i64) -> &str {
    // TODO
    let mut num_digits = 0;
    let mut upper_bound = 1;

    if val == 0 {
        num_digits = 1;
        buf[0] = b'0';
    } else if val == 1 {
        num_digits = 1;
        buf[0] = b'1';
    } else {
        while upper_bound < val {
            num_digits += 1;
            upper_bound *= 10;
        }
    }

    unsafe { core::str::from_utf8_unchecked(&buf[..num_digits]) }
}

fn populate_buf_with_uint(buf: &mut [u8], val: u64) -> &str {
    // TODO
    let mut num_digits = 0;
    let mut upper_bound = 1;

    if val == 0 {
        num_digits = 1;
        buf[0] = b'0';
    } else if val == 1 {
        num_digits = 1;
        buf[0] = b'1';
    } else {
        while upper_bound < val {
            num_digits += 1;
            upper_bound *= 10;
        }
    }

    unsafe { core::str::from_utf8_unchecked(&buf[..num_digits]) }
}
