//! Canonical numeric representation used inside sort keys.
//!
//! Python's natsort can freely mix arbitrary-precision `int` and `float`
//! inside the same sort-key tuple because Python's numeric tower compares
//! them exactly. Rust has no such tower, so every numeric sort-key
//! component (whether it came from an int-looking or float-looking string)
//! is normalized into this single `Num` enum, which knows how to compare
//! itself against any other variant without ever panicking (fixes the
//! NaN-unwrap panic and the >i64/>u128 overflow-to-string-fallback bugs).

use num_bigint::BigInt;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub enum Num {
    Int(i64),
    Big(BigInt),
    Float(f64),
}

impl Num {
    pub fn from_str_radix10(s: &str) -> Option<Num> {
        if let Ok(i) = s.parse::<i64>() {
            return Some(Num::Int(i));
        }
        // Too big (or too negative) for i64 -- use arbitrary precision
        // instead of silently truncating or falling back to string
        // comparison. This is the fix for "Integer Overflow on Large
        // Numeric Strings".
        s.parse::<BigInt>().ok().map(Num::Big)
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Num::Int(i) => *i as f64,
            Num::Big(b) => big_to_f64(b),
            Num::Float(f) => *f,
        }
    }
}

fn big_to_f64(b: &BigInt) -> f64 {
    // num-bigint doesn't provide a lossless BigInt->f64 conversion on all
    // versions, so we go through the decimal string. This is only used
    // when comparing a BigInt against a genuine f64 (a rare mixed case),
    // so the tiny precision loss for astronomically large integers is an
    // acceptable, documented tradeoff -- it never panics and never
    // silently reorders same-type comparisons (Int/Int and Big/Big below
    // stay exact).
    b.to_string()
        .parse::<f64>()
        .unwrap_or(if b.to_string().starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        })
}

/// Total ordering across mixed Num variants. Never panics, even on NaN:
/// NaN is treated as sorting consistently (via `total_cmp` semantics)
/// rather than via `partial_cmp().unwrap()`, which is what caused the
/// original "Float Comparison Panics" bug.
pub fn num_cmp(a: &Num, b: &Num) -> Ordering {
    match (a, b) {
        (Num::Int(x), Num::Int(y)) => x.cmp(y),
        (Num::Big(x), Num::Big(y)) => x.cmp(y),
        (Num::Int(x), Num::Big(y)) => BigInt::from(*x).cmp(y),
        (Num::Big(x), Num::Int(y)) => x.cmp(&BigInt::from(*y)),
        (Num::Float(x), Num::Float(y)) => x.total_cmp(y),
        (Num::Float(x), other) => x.total_cmp(&other.as_f64()),
        (other, Num::Float(y)) => other.as_f64().total_cmp(y),
    }
}

impl PartialEq for Num {
    fn eq(&self, other: &Self) -> bool {
        num_cmp(self, other) == Ordering::Equal
    }
}

/// A single component of a natural-sort key. This replaces the original
/// port's `Box<dyn Any + Send + Sync>` representation, which required
/// fragile runtime downcasting (and a fake `CloneBox` impl that silently
/// discarded data) to compare and clone key components. `KeyPart` is a
/// plain, cheaply-`Clone`-able enum that knows how to compare itself
/// against any other `KeyPart`, so no downcasting or panics are involved
/// anywhere in the sort path.
#[derive(Debug, Clone)]
pub enum KeyPart {
    Str(String),
    Num(Num),
    Bytes(Vec<u8>),
    /// A sub-key -- used for path components (`ns.PATH`) and for the
    /// pre-pended case-marker component of `ns.UNGROUPLETTERS`.
    Nested(Vec<KeyPart>),
}

impl PartialEq for KeyPart {
    fn eq(&self, other: &Self) -> bool {
        key_part_cmp(self, other) == Ordering::Equal
    }
}

/// Stable rank used only to order two `KeyPart`s of *different* variants.
/// In a well-formed sort key, same-position components across different
/// input items are always the same variant (produced by the same regex
/// split path/flags), so this branch is rarely hit in practice -- it
/// exists purely as a safe, deterministic, panic-free fallback rather than
/// an authoritative ordering.
fn variant_rank(k: &KeyPart) -> u8 {
    match k {
        KeyPart::Str(_) => 0,
        KeyPart::Num(_) => 1,
        KeyPart::Bytes(_) => 2,
        KeyPart::Nested(_) => 3,
    }
}

pub fn key_part_cmp(a: &KeyPart, b: &KeyPart) -> Ordering {
    match (a, b) {
        (KeyPart::Str(x), KeyPart::Str(y)) => x.cmp(y),
        (KeyPart::Num(x), KeyPart::Num(y)) => num_cmp(x, y),
        (KeyPart::Bytes(x), KeyPart::Bytes(y)) => x.cmp(y),
        (KeyPart::Nested(x), KeyPart::Nested(y)) => compare_key_vec(x, y),
        _ => variant_rank(a).cmp(&variant_rank(b)),
    }
}

/// Compares two full sort-key tuples the way Python compares tuples:
/// element-wise, and if one is a strict prefix of the other, the shorter
/// one sorts first.
pub fn compare_key_vec(a: &[KeyPart], b: &[KeyPart]) -> Ordering {
    for (x, y) in a.iter().zip(b.iter()) {
        let c = key_part_cmp(x, y);
        if c != Ordering::Equal {
            return c;
        }
    }
    a.len().cmp(&b.len())
}
