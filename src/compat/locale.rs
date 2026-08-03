//! Stand-in for Python's `locale` module integration.
//!
//! Python natsort queries `locale.localeconv()` / `PyICU` for the active
//! locale's decimal point, thousands separator, and string-collation
//! transform (`strxfrm`). Rust has no locale-aware `strxfrm` in std, and
//! binding real OS locale/ICU is out of scope for a pure-Rust crate, so we
//! provide well-defined, deterministic fallbacks:
//!
//! - decimal point / thousands separator default to the "C" locale ('.' and
//!   ',' respectively). This matches natsort's own behavior when no locale
//!   has been explicitly set via `locale.setlocale`.
//! - `strxfrm` falls back to plain NFC string comparison order (identity),
//!   which is correct for ASCII/English collation and is a documented
//!   approximation for other locales -- true locale collation would require
//!   an ICU binding, which is a larger follow-up (see icu_collator note in
//!   the review).
//! - `dumb_sort` reports whether the "locale is broken" workaround
//!   (case-swap before comparing) is needed. Since we never activate a
//!   real OS locale, it is always `false` here.

pub type StrOrBytes = String;

/// Separator string used between adjacent numeric key components so they
/// never accidentally concatenate into a different number. Sorts before
/// all normal text.
pub fn null_string() -> &'static str {
    "\0"
}

/// Same idea as `null_string`, but for `ns.NUMAFTER`: sorts after all
/// normal text instead of before it.
pub fn null_string_max() -> &'static str {
    "\u{10FFFF}"
}

/// Locale-aware variant of `null_string`. Without a bound locale library
/// this is identical to the non-locale separator.
pub fn null_string_locale() -> String {
    null_string().to_string()
}

/// Locale-aware variant of `null_string_max`.
pub fn null_string_locale_max() -> String {
    null_string_max().to_string()
}

/// Whether the active "locale" needs the swap-case dumb-sort workaround.
/// Always `false` since no real OS locale is bound.
pub fn dumb_sort() -> bool {
    false
}

/// The locale's decimal point character. Defaults to '.' (the "C" locale).
pub fn get_decimal_point() -> String {
    ".".to_string()
}

/// The locale's thousands separator character. Defaults to ','.
pub fn get_thousands_sep() -> String {
    ",".to_string()
}

/// A locale-aware string transform used for locale-based collation
/// (`ns.LOCALEALPHA`). Falls back to the identity function -- correct for
/// ASCII, a documented approximation otherwise.
pub fn get_strxfrm() -> Box<dyn Fn(&str) -> String + Send + Sync> {
    Box::new(|x: &str| x.to_string())
}
