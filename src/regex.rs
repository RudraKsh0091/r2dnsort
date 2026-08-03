//! compiled regex patterns for number/string splitting
//! <- test_regex.rs

use regex::Regex;

use crate::ns_enum::{NSType, Ns};
use crate::parsing::unicode_numbers::{digits_no_decimals, numeric_no_decimals};

/// Container of regular expressions that match numbers.
/// The numbers also account for unicode non-decimal characters.
pub struct NumericalRegularExpressions;

impl NumericalRegularExpressions {
    fn numeric() -> String {
        numeric_no_decimals().to_string()
    }
    fn digits() -> String {
        digits_no_decimals().to_string()
    }
    fn exp() -> &'static str {
        r"(?:[eE][-+]?\d+)?"
    }
    fn float_num() -> &'static str {
        r"(?:\d+\.?\d*|\.\d+)"
    }

    fn construct_regex(fmt: &str) -> Regex {
        let numeric = Self::numeric();
        let digits = Self::digits();
        let exp = Self::exp();
        let float_num = Self::float_num();
        let pattern = fmt
            .replace("{numeric}", &numeric)
            .replace("{digits}", &digits)
            .replace("{exp}", exp)
            .replace("{float_num}", float_num);
        Regex::new(&pattern).unwrap()
    }

    pub fn int_sign() -> Regex {
        Self::construct_regex(r"([-+]?\d+|[{digits}])")
    }

    pub fn int_nosign() -> Regex {
        Self::construct_regex(r"(\d+|[{digits}])")
    }

    pub fn float_sign_exp() -> Regex {
        Self::construct_regex(r"([-+]?{float_num}{exp}|[{numeric}])")
    }

    pub fn float_nosign_exp() -> Regex {
        Self::construct_regex(r"({float_num}{exp}|[{numeric}])")
    }

    pub fn float_sign_noexp() -> Regex {
        Self::construct_regex(r"([-+]?{float_num}|[{numeric}])")
    }

    pub fn float_nosign_noexp() -> Regex {
        Self::construct_regex(r"({float_num}|[{numeric}])")
    }
}

/// Select an appropriate regex for the type of number of interest.
pub fn regex_chooser(alg: NSType) -> Regex {
    let mut alg = Ns(alg);
    if alg.contains(Ns::FLOAT) {
        alg = Ns(alg.0 & (Ns::FLOAT.0 | Ns::SIGNED.0 | Ns::NOEXP.0));
    } else {
        alg = Ns(alg.0 & (Ns::INT.0 | Ns::SIGNED.0));
    }

    match alg.0 {
        x if x == Ns::INT.0 => NumericalRegularExpressions::int_nosign(),
        x if x == Ns::FLOAT.0 => NumericalRegularExpressions::float_nosign_exp(),
        x if x == (Ns::INT.0 | Ns::SIGNED.0) => NumericalRegularExpressions::int_sign(),
        x if x == (Ns::FLOAT.0 | Ns::SIGNED.0) => NumericalRegularExpressions::float_sign_exp(),
        x if x == (Ns::FLOAT.0 | Ns::NOEXP.0) => NumericalRegularExpressions::float_nosign_noexp(),
        x if x == (Ns::FLOAT.0 | Ns::SIGNED.0 | Ns::NOEXP.0) => {
            NumericalRegularExpressions::float_sign_noexp()
        }
        _ => NumericalRegularExpressions::int_nosign(),
    }
}
