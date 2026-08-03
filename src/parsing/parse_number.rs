//! Wraps a possibly-numeric input value (already produced by `num_func`'s
//! caller, e.g. an `i64`/`f64`/`Option<()>` "None"/NaN sentinel) into a
//! sort-key tuple, using sentinel components so that `None`/`NaN` values
//! consistently sort first or last (`ns.NANLAST`) instead of causing a
//! panic or comparing inconsistently with real numbers.
//! <- natsort/natsort_key.py (number branch)

use crate::compat::locale::StrOrBytes;
use crate::ns_enum::{NSType, Ns};
use crate::numtype::{KeyPart, Num};

pub type NumTransform = Vec<KeyPart>;
pub type NumTransformer = Box<dyn Fn(NumInput) -> NumTransform + Send + Sync>;

/// The kinds of "raw" values that can reach the number-parsing stage:
/// user-supplied numbers, or an explicit "missing" marker (equivalent to
/// Python's `None`, which natsort sorts consistently to one end).
#[derive(Debug, Clone)]
pub enum NumInput {
    I64(i64),
    F64(f64),
    None,
}

/// Create a function that formats a number (or a missing/NaN value) into
/// its sort-key component(s).
pub fn parse_number_or_none_factory(alg: NSType, sep: StrOrBytes, pre_sep: &str) -> NumTransformer {
    let alg = Ns(alg);
    // NaN/None are given a value that's either +inf or -inf so they sort
    // consistently last or first, matching ns.NANLAST semantics -- and
    // crucially this value is used only as a *tag*, never fed into an
    // `.unwrap()` on `partial_cmp`, so no NaN-comparison panic can occur
    // (fixes "Float Comparison Panics").
    let nan_replace = if alg.contains(Ns::NANLAST) {
        f64::INFINITY
    } else {
        f64::NEG_INFINITY
    };
    let use_path_prefix = alg.contains(Ns::PATH)
        || (alg.contains(Ns::UNGROUPLETTERS) && alg.contains(Ns::LOCALEALPHA));
    let pre_sep = pre_sep.to_string();
    let sep = sep.clone();

    Box::new(move |val: NumInput| {
        let body = num_component(val, nan_replace, &sep);
        if use_path_prefix {
            vec![
                KeyPart::Nested(vec![KeyPart::Str(pre_sep.clone())]),
                KeyPart::Nested(body),
            ]
        } else {
            body
        }
    })
}

fn num_component(val: NumInput, nan_replace: f64, sep: &str) -> NumTransform {
    match val {
        NumInput::F64(f) if f.is_nan() => {
            vec![
                KeyPart::Str(sep.to_string()),
                KeyPart::Num(Num::Float(nan_replace)),
            ]
        }
        NumInput::F64(f) => vec![KeyPart::Str(sep.to_string()), KeyPart::Num(Num::Float(f))],
        NumInput::I64(i) => vec![KeyPart::Str(sep.to_string()), KeyPart::Num(Num::Int(i))],
        NumInput::None => vec![
            KeyPart::Str(sep.to_string()),
            KeyPart::Num(Num::Float(nan_replace)),
        ],
    }
}
