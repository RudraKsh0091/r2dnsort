//! Unicode numeral support.
//! <- natsort/unicode_numbers.py
//!
//! Python's `int()`/`float()` builtins natively understand non-ASCII
//! Unicode decimal digits (e.g. Arabic-Indic `١٢٣`) and even some
//! non-digit numeric characters (superscripts, vulgar fractions). Rust's
//! `str::parse::<i64>()`/`str::parse::<f64>()` are ASCII-only, so without
//! an explicit translation step, natural-sort strings containing these
//! characters silently fall back to lexicographic string comparison
//! instead of numeric comparison ("Unicode Numeral Parsing Limitations").
//!
//! This module provides:
//! - `digits_no_decimals()` / `numeric_no_decimals()`: regex
//!   character-class fragments (used by `crate::regex`) that match
//!   non-ASCII numeral characters so they get split out of the string
//!   alongside ordinary ASCII digit runs.
//! - `unicode_digit_value` / `unicode_numeric_value`: per-character value
//!   lookups used to translate a matched numeral into an actual number
//!   before parsing.

use crate::numtype::Num;

/// (start, end) inclusive codepoint ranges of 10 consecutive Unicode
/// decimal-digit characters (Unicode General Category `Nd`), covering the
/// major scripts. Each range's digits are in the same 0-9 order as ASCII.
/// Not an exhaustive list of every `Nd` block in Unicode, but covers all
/// commonly-encountered scripts (Arabic, Devanagari, Bengali, Gurmukhi,
/// Gujarati, Oriya, Tamil, Telugu, Kannada, Malayalam, Thai, Lao, Tibetan,
/// Myanmar, Khmer, Mongolian, fullwidth).
const DIGIT_RANGES: &[(u32, u32)] = &[
    (0x0660, 0x0669), // Arabic-Indic
    (0x06F0, 0x06F9), // Extended Arabic-Indic
    (0x0966, 0x096F), // Devanagari
    (0x09E6, 0x09EF), // Bengali
    (0x0A66, 0x0A6F), // Gurmukhi
    (0x0AE6, 0x0AEF), // Gujarati
    (0x0B66, 0x0B6F), // Oriya
    (0x0BE6, 0x0BEF), // Tamil
    (0x0C66, 0x0C6F), // Telugu
    (0x0CE6, 0x0CEF), // Kannada
    (0x0D66, 0x0D6F), // Malayalam
    (0x0E50, 0x0E59), // Thai
    (0x0ED0, 0x0ED9), // Lao
    (0x0F20, 0x0F29), // Tibetan
    (0x1040, 0x1049), // Myanmar
    (0x17E0, 0x17E9), // Khmer
    (0x1810, 0x1819), // Mongolian
    (0xFF10, 0xFF19), // Fullwidth
];

/// Individual non-digit numeric characters and the value each represents:
/// superscripts, subscripts, and vulgar fractions.
const NUMERIC_EXTRA: &[(char, f64)] = &[
    ('\u{00B2}', 2.0), // superscript two
    ('\u{00B3}', 3.0), // superscript three
    ('\u{00B9}', 1.0), // superscript one
    ('\u{2070}', 0.0), // superscript zero
    ('\u{2074}', 4.0),
    ('\u{2075}', 5.0),
    ('\u{2076}', 6.0),
    ('\u{2077}', 7.0),
    ('\u{2078}', 8.0),
    ('\u{2079}', 9.0),
    ('\u{2080}', 0.0), // subscript zero
    ('\u{2081}', 1.0),
    ('\u{2082}', 2.0),
    ('\u{2083}', 3.0),
    ('\u{2084}', 4.0),
    ('\u{2085}', 5.0),
    ('\u{2086}', 6.0),
    ('\u{2087}', 7.0),
    ('\u{2088}', 8.0),
    ('\u{2089}', 9.0),
    ('\u{00BC}', 0.25),      // 1/4
    ('\u{00BD}', 0.5),       // 1/2
    ('\u{00BE}', 0.75),      // 3/4
    ('\u{2150}', 1.0 / 7.0), // 1/7
    ('\u{2151}', 1.0 / 9.0), // 1/9
    ('\u{2152}', 0.1),       // 1/10
    ('\u{2153}', 1.0 / 3.0), // 1/3
    ('\u{2154}', 2.0 / 3.0), // 2/3
    ('\u{2155}', 0.2),       // 1/5
    ('\u{2156}', 0.4),       // 2/5
    ('\u{2157}', 0.6),       // 3/5
    ('\u{2158}', 0.8),       // 4/5
    ('\u{2159}', 1.0 / 6.0), // 1/6
    ('\u{215A}', 5.0 / 6.0), // 5/6
    ('\u{215B}', 0.125),     // 1/8
    ('\u{215C}', 0.375),     // 3/8
    ('\u{215D}', 0.625),     // 5/8
    ('\u{215E}', 0.875),     // 7/8
];

/// If `c` is a non-ASCII Unicode decimal digit, returns its value 0-9.
pub fn unicode_digit_value(c: char) -> Option<u32> {
    let cp = c as u32;
    for &(start, end) in DIGIT_RANGES {
        if cp >= start && cp <= end {
            return Some(cp - start);
        }
    }
    None
}

/// If `c` is any recognized Unicode numeral (a decimal digit, superscript,
/// subscript, or vulgar fraction), returns its numeric value.
pub fn unicode_numeric_value(c: char) -> Option<f64> {
    if let Some(d) = unicode_digit_value(c) {
        return Some(d as f64);
    }
    for &(ch, v) in NUMERIC_EXTRA {
        if ch == c {
            return Some(v);
        }
    }
    None
}

/// Regex character-class fragment (content only, no surrounding `[...]`)
/// matching non-ASCII Unicode decimal digits.
pub fn digits_no_decimals() -> String {
    DIGIT_RANGES
        .iter()
        .map(|(s, e)| format!("\\u{{{:04X}}}-\\u{{{:04X}}}", s, e))
        .collect()
}

/// Regex character-class fragment matching any recognized Unicode numeral
/// (digits plus superscripts/subscripts/fractions).
pub fn numeric_no_decimals() -> String {
    let mut s = digits_no_decimals();
    for (ch, _) in NUMERIC_EXTRA {
        s.push_str(&format!("\\u{{{:04X}}}", *ch as u32));
    }
    s
}

/// Converts a string made up entirely of ASCII and/or Unicode decimal
/// digits into a `Num`, honoring Unicode digit values. Returns `None` if
/// any character isn't a recognized digit.
pub fn parse_unicode_digits(s: &str) -> Option<Num> {
    let mut ascii_digits = String::with_capacity(s.len());
    for c in s.chars() {
        if let Some(d) = c.to_digit(10) {
            ascii_digits.push(std::char::from_digit(d, 10).unwrap());
        } else if let Some(d) = unicode_digit_value(c) {
            ascii_digits.push(std::char::from_digit(d, 10).unwrap());
        } else {
            return None;
        }
    }
    if ascii_digits.is_empty() {
        None
    } else {
        Num::from_str_radix10(&ascii_digits)
    }
}

/// Converts a single recognized non-digit Unicode numeral character (a
/// superscript, subscript, or vulgar fraction) into a `Num`.
pub fn parse_unicode_numeric_char(s: &str) -> Option<Num> {
    let mut chars = s.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None; // more than one char -- not a single numeral symbol
    }
    unicode_numeric_value(c).map(Num::Float)
}
