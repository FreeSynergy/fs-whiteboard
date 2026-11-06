//! Order keys: a text key strictly between two others. Pure.
//!
//! Pattern: Strategy (key generation) — a key is the fraction `0.d1d2d3...` in base 62 with the digits
//! `0-9A-Za-z`, which sort as bytes in digit order. A key never ends in `0`, so two different keys are
//! two different numbers. There is no fixed length: a key grows only when no shorter one fits.
//!
//! Measured (tests/order.rs): 10 000 appends at the end give keys of up to 165 characters (one more
//! character per 61 appends); 10 000 prepends 323; 10 000 inserts into one ever-shrinking gap, the
//! worst case that no scheme with immutable keys can beat, 2 001 (about 5 inserts per character);
//! random positions stay far below 24.

const DIGITS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const TOP: usize = 62;

fn digit(key: &str, i: usize) -> usize {
    key.as_bytes()
        .get(i)
        .and_then(|c| DIGITS.iter().position(|d| d == c))
        .unwrap_or(0)
}

fn ch(d: usize) -> char {
    char::from(DIGITS[d])
}

/// True if `key` is a key this module makes: digits only, not empty, not ending in `0`.
pub fn is_valid(key: &str) -> bool {
    !key.is_empty() && !key.ends_with('0') && key.bytes().all(|c| DIGITS.contains(&c))
}

/// A key strictly between `a` and `b`, with `None` as the open end on either side. If `a` is not
/// below `b`, the key goes after `a`.
pub fn between(a: Option<&str>, b: Option<&str>) -> String {
    let a = a.unwrap_or("");
    match b.filter(|b| !b.is_empty()) {
        None => after(a),
        Some(b) if !a.is_empty() && a >= b => after(a),
        Some(b) if a.is_empty() => before(b),
        Some(b) => midpoint(a, b),
    }
}

/// `n` keys in increasing order, all between `a` and `b`, spread so that they stay short.
pub fn spread(a: Option<&str>, b: Option<&str>, n: usize) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let m = between(a, b);
    let mut out = spread(a, Some(&m), n / 2);
    out.push(m.clone());
    out.extend(spread(Some(&m), b, n - 1 - n / 2));
    out
}

fn after(a: &str) -> String {
    if a.is_empty() {
        return "V".into();
    }
    for i in 0..a.len() {
        let d = digit(a, i);
        if d < TOP - 1 {
            return format!("{}{}", &a[..i], ch(d + 1));
        }
    }
    format!("{a}1")
}

fn before(b: &str) -> String {
    let b = b.trim_end_matches('0');
    for i in 0..b.len() {
        let d = digit(b, i);
        if d >= 2 {
            return format!("{}{}", &b[..i], ch(d - 1));
        }
    }
    match b.len() {
        0 => "V".into(),
        n => format!("{}0V", &b[..n - 1]),
    }
}

fn midpoint(a: &str, b: &str) -> String {
    let mut out = String::new();
    let mut tight = true;
    for i in 0.. {
        let da = digit(a, i);
        let db = if tight { digit(b, i) } else { TOP };
        if db > da + 1 {
            out.push(ch(usize::midpoint(da, db)));
            return out;
        }
        out.push(ch(da));
        tight &= db == da;
        if tight && i >= b.len() {
            return after(a);
        }
    }
    out
}
