//! Pre-processing transform applied to input strings before they're
//! regex-split: case folding / dumb-sort swap-case, and locale-aware
//! number-grouping (thousands separator) stripping.
//! <- transform/input_string_transform.py

use crate::compat::locale::{get_decimal_point, get_thousands_sep};
use crate::ns_enum::{NSType, Ns, NS_DUMB};
use regex::Regex;

pub type StrToStr = Box<dyn Fn(&str) -> String + Send + Sync>;

fn no_op(x: &str) -> String {
    x.to_string()
}

/// Create a function to transform a string prior to splitting.
pub fn input_string_transform_factory(alg: NSType) -> StrToStr {
    let alg = Ns(alg);
    let lowfirst = alg.contains(Ns::LOWERCASEFIRST);
    let dumb = alg.contains(NS_DUMB);

    let mut function_chain: Vec<StrToStr> = Vec::new();

    // Case-swap workaround needed when either lowfirst xor dumb is active
    // (matches Python's `swapcase` branch).
    if (dumb && !lowfirst) || (lowfirst && !dumb) {
        function_chain.push(Box::new(swap_case));
    }

    if alg.contains(Ns::IGNORECASE) {
        function_chain.push(Box::new(|x: &str| x.to_lowercase()));
    }

    if alg.contains(Ns::LOCALENUM) {
        let thousands_sep = get_thousands_sep();
        let decimal_point = get_decimal_point();
        function_chain.push(Box::new(move |x: &str| {
            let mut s = strip_thousands_seps(x, &thousands_sep);
            if decimal_point != "." {
                s = s.replace(&decimal_point, ".");
            }
            s
        }));
    }

    chain_functions(function_chain)
}

fn swap_case(x: &str) -> String {
    x.chars()
        .flat_map(|c| {
            if c.is_lowercase() {
                c.to_uppercase().collect::<Vec<_>>()
            } else {
                c.to_lowercase().collect::<Vec<_>>()
            }
        })
        .collect()
}

/// Removes thousands-grouping separators (e.g. "1,234,567" -> "1234567").
///
/// The original port attempted this with a lookbehind/lookahead regex
/// (`(?<=\d)SEP(?=\d{3}...)`), which the `regex` crate rejects outright at
/// compile time (it supports neither look-ahead nor look-behind, by
/// design, to guarantee linear-time matching) -- this was a hard compile
/// failure, not just an inaccuracy ("Regex Lookaround Failures" bug).
/// This version uses only a capturing group (`(\d)SEP(\d{3})` -> `$1$2`),
/// applied repeatedly until stable so multi-group numbers like
/// "1,234,567" are fully stripped.
fn strip_thousands_seps(s: &str, thousands_sep: &str) -> String {
    if thousands_sep.is_empty() {
        return s.to_string();
    }
    let pattern = format!(r"(\d){}(\d{{3}})", regex::escape(thousands_sep));
    let re = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(_) => return s.to_string(),
    };
    let mut cur = s.to_string();
    loop {
        let next = re.replace_all(&cur, "$1$2").to_string();
        if next == cur {
            return cur;
        }
        cur = next;
    }
}

fn chain_functions(functions: Vec<StrToStr>) -> StrToStr {
    if functions.is_empty() {
        return Box::new(no_op);
    }
    if functions.len() == 1 {
        return functions.into_iter().next().unwrap();
    }
    Box::new(move |x: &str| {
        let mut res = x.to_string();
        for f in &functions {
            res = f(&res);
        }
        res
    })
}
