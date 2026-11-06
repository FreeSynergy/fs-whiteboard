//! Order keys: a text key strictly between two others. Pure.
//!
//! Pattern: Strategy (key generation) — a key is the fraction `0.d1d2d3...` in base 62 with the digits
//! `0-9A-Za-z`, which sort as bytes in digit order. A key never ends in `0`, so two different keys are
//! two different numbers. There is no fixed length: a key grows only when no shorter one fits.
//!
//! Appending counts upward: the first key is `V`, then `W1`..`Wz`, `X01`..`Xzz`, and so on, where the
//! first character tells how many counter digits follow. A key grows with the logarithm of the number
//! of appends (tests/order.rs measures 10 000 appends: 4 characters). Inserting again and again into
//! one shrinking gap is the worst case no scheme with unchanging keys can beat (about 5 inserts per
//! character); the editor refuses a key over 1 KiB, see `validate::MAX_ORDER_BYTES`.

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
    // anything that is not a key of this module counts as an open end, so no input can panic
    let a = a.filter(|k| is_valid(k)).unwrap_or("");
    match b.filter(|k| is_valid(k)) {
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

const CLASS0: usize = 31; // `V`: no counter digits

/// The next key in the counting sequence, if `a` belongs to it.
fn count_up(a: &str) -> Option<String> {
    let class = digit(a, 0);
    let width = class.checked_sub(CLASS0)?;
    if a.len() != width + 1 {
        return None;
    }
    let mut digits: Vec<usize> = (1..a.len()).map(|i| digit(a, i)).collect();
    loop {
        let mut i = digits.len();
        loop {
            if i == 0 {
                // all digits were `z`: the next class, counter 0..01
                (class + 1 < TOP).then_some(())?;
                let mut next = String::from(ch(class + 1));
                next.extend(std::iter::repeat_n('0', width));
                next.push('1');
                return Some(next);
            }
            i -= 1;
            if digits[i] + 1 < TOP {
                digits[i] += 1;
                break;
            }
            digits[i] = 0;
        }
        if digits.last().is_none_or(|d| *d != 0) {
            let tail: String = digits.iter().map(|d| ch(*d)).collect();
            return Some(format!("{}{tail}", ch(class)));
        }
    }
}

fn after(a: &str) -> String {
    if a.is_empty() {
        return "V".into();
    }
    if let Some(k) = count_up(a) {
        return k;
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
