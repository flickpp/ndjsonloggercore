const JSON_INF: &str = "\"Infinity\"";
const JSON_NEG_INF: &str = "\"-Infinity\"";
const JSON_NAN: &str = "\"Nan\"";

pub fn itoa_base10(buf: &mut [u8], val: i64) -> &str {
    if val >= 0 {
        utoa_base10(buf, val as u64)
    } else {
        // First char is negative
        let val: u64 = if val == i64::MIN {
            9223372036854775808
        } else {
            (-val) as u64
        };

        let start_pos = utoa_alg(buf, val) - 1;
        buf[start_pos] = b'-';
        unsafe { core::str::from_utf8_unchecked(&buf[start_pos..]) }
    }
}

pub fn utoa_base10(buf: &mut [u8], val: u64) -> &str {
    if val == 0 {
        buf[0] = b'0';
        return unsafe { core::str::from_utf8_unchecked(&buf[..1]) };
    }

    let start_pos = utoa_alg(buf, val);
    unsafe { core::str::from_utf8_unchecked(&buf[start_pos..]) }
}

pub fn f64_to_str(buf: &mut [u8], val: f64) -> &str {
    let num_bytes = if val.is_nan() {
        for (a, b) in buf.iter_mut().zip(JSON_NAN.bytes()) {
            *a = b;
        }
        JSON_NAN.len()
    } else if val.is_infinite() && val.is_sign_positive() {
        for (a, b) in buf.iter_mut().zip(JSON_INF.bytes()) {
            *a = b;
        }
        JSON_INF.len()
    } else if val.is_infinite() && val.is_sign_negative() {
        for (a, b) in buf.iter_mut().zip(JSON_NEG_INF.bytes()) {
            *a = b;
        }
        JSON_NEG_INF.len()
    } else {
        unsafe { ryu::raw::format64(val, buf.as_mut_ptr()) }
    };

    unsafe { core::str::from_utf8_unchecked(&buf[..num_bytes]) }
}

fn utoa_alg(buf: &mut [u8], mut val: u64) -> usize {
    let mut pos = buf.len() - 1;

    while val > 0 {
        let r = (val % 10) as u8;

        buf[pos] = r + 48;
        pos -= 1;

        val /= 10;
    }

    pos + 1
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn utoa_alg_() {
        let mut buf = [0_u8; 24];

        assert_eq!(utoa_base10(&mut buf, 0), "0");
        assert_eq!(utoa_base10(&mut buf, 1), "1");
        assert_eq!(utoa_base10(&mut buf, 2), "2");
        assert_eq!(utoa_base10(&mut buf, 9), "9");
        assert_eq!(utoa_base10(&mut buf, 10), "10");
        assert_eq!(utoa_base10(&mut buf, 11), "11");
        assert_eq!(utoa_base10(&mut buf, 12), "12");
        assert_eq!(utoa_base10(&mut buf, 99), "99");
        assert_eq!(utoa_base10(&mut buf, 100), "100");
        assert_eq!(utoa_base10(&mut buf, 101), "101");
        assert_eq!(utoa_base10(&mut buf, u64::MAX), "18446744073709551615");
    }

    #[test]
    fn itoa_alg_() {
        let mut buf = [0_u8; 24];

        assert_eq!(itoa_base10(&mut buf, 0), "0");
        assert_eq!(itoa_base10(&mut buf, 1), "1");
        assert_eq!(itoa_base10(&mut buf, -1), "-1");
        assert_eq!(itoa_base10(&mut buf, -2), "-2");
        assert_eq!(itoa_base10(&mut buf, -9), "-9");
        assert_eq!(itoa_base10(&mut buf, -10), "-10");
        assert_eq!(itoa_base10(&mut buf, -11), "-11");
        assert_eq!(itoa_base10(&mut buf, -12), "-12");
        assert_eq!(itoa_base10(&mut buf, -99), "-99");
        assert_eq!(itoa_base10(&mut buf, -100), "-100");
        assert_eq!(itoa_base10(&mut buf, -101), "-101");
        assert_eq!(itoa_base10(&mut buf, i64::MAX), "9223372036854775807");
        assert_eq!(itoa_base10(&mut buf, i64::MIN + 1), "-9223372036854775807");
        assert_eq!(itoa_base10(&mut buf, i64::MIN), "-9223372036854775808");
    }

    #[test]
    fn f64_to_str_() {
        let mut buf = [0_u8; 24];

        assert_eq!(f64_to_str(&mut buf, 0.), "0.0");
        assert_eq!(f64_to_str(&mut buf, 1.), "1.0");
        assert_eq!(f64_to_str(&mut buf, 2.), "2.0");

        assert_eq!(f64_to_str(&mut buf, f64::NAN), JSON_NAN);
        assert_eq!(f64_to_str(&mut buf, f64::INFINITY), JSON_INF);
        assert_eq!(f64_to_str(&mut buf, f64::NEG_INFINITY), JSON_NEG_INF);
    }
}
