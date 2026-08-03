//! Stand-in for natsort's optional `fastnumbers` C-extension acceleration
//! (`compat/fastnumbers.py`'s pure-Python fallback path).
//!
//! Converts a already-regex-isolated string chunk into either a numeric
//! `KeyPart::Num` or, if it isn't actually numeric, a (possibly
//! locale/grouping-transformed) `KeyPart::Str` via `on_fail`.
//!
//! Handles plain ASCII numbers via `str::parse`, and falls back to the
//! Unicode-numeral tables in `crate::parsing::unicode_numbers` so that
//! non-ASCII numerals (Arabic-Indic, Devanagari, superscripts, vulgar
//! fractions, ...) are parsed as numbers instead of silently degrading to
//! string comparison -- this is the fix for the "Unicode Numeral Parsing
//! Limitations" bug.

use crate::numtype::{KeyPart, Num};
use crate::parsing::unicode_numbers::{parse_unicode_digits, parse_unicode_numeric_char};

/// Attempts to interpret `s` as a float. Falls back to `on_fail(s)`
/// (or `s` itself, if no fallback is given) wrapped in `KeyPart::Str`.
pub fn try_float(s: &str, on_fail: Option<&dyn Fn(&str) -> String>) -> KeyPart {
    if let Ok(f) = s.parse::<f64>() {
        return KeyPart::Num(Num::Float(f));
    }
    if let Some(n) = parse_unicode_digits(s) {
        return KeyPart::Num(Num::Float(n.as_f64()));
    }
    if let Some(n) = parse_unicode_numeric_char(s) {
        return KeyPart::Num(n);
    }
    KeyPart::Str(apply_fallback(s, on_fail))
}

/// Attempts to interpret `s` as an integer (arbitrary precision -- see
/// `Num::from_str_radix10`). Falls back like `try_float`.
pub fn try_int(s: &str, on_fail: Option<&dyn Fn(&str) -> String>) -> KeyPart {
    if let Some(n) = Num::from_str_radix10(s) {
        return KeyPart::Num(n);
    }
    if let Some(n) = parse_unicode_digits(s) {
        return KeyPart::Num(n);
    }
    if let Some(n) = parse_unicode_numeric_char(s) {
        return KeyPart::Num(n);
    }
    KeyPart::Str(apply_fallback(s, on_fail))
}

fn apply_fallback(s: &str, on_fail: Option<&dyn Fn(&str) -> String>) -> String {
    match on_fail {
        Some(f) => f(s),
        None => s.to_string(),
    }
}
