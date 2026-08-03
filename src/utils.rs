//! Shared helpers tying the parsing/transform pipeline together, and the
//! `natsort_key()` value dispatcher.
//! <- natsort/natsort_key.py, natsort/utils.py

pub use crate::parsing::parse_bytes::{parse_bytes_factory, BytesTransformer};
pub use crate::parsing::parse_number::{parse_number_or_none_factory, NumInput, NumTransformer};
pub use crate::parsing::parse_string::{parse_path_factory, parse_string_factory, path_splitter};
pub use crate::regex::{regex_chooser, NumericalRegularExpressions};
pub use crate::transform::final_data_transform::final_data_transform_factory;
pub use crate::transform::input_string_transform::input_string_transform_factory;
pub use crate::transform::string_component_transform::string_component_transform_factory;

use crate::numtype::KeyPart;
use std::any::Any;
use std::sync::Arc;

pub type NatsortInType = Box<dyn Any + Send + Sync>;
pub type NatsortOutType = Vec<KeyPart>;

/// A user-supplied key-extraction callback, e.g. "sort by this field of
/// the item". Wrapped in `Arc` (not `Box`) so it -- and the pipeline
/// functions below -- can be cheaply cloned into the reusable closure
/// `natsort_keygen` returns *and* recursed into for nested list/tuple
/// input, without the fake, data-discarding `clone_box` shim the original
/// port used (which silently produced empty keys for any nested-sequence
/// input).
pub type KeyType = Arc<dyn Fn(NatsortInType) -> NatsortInType + Send + Sync>;
pub type StringKeyFn = Arc<dyn Fn(&str) -> NatsortOutType + Send + Sync>;
pub type BytesKeyFn = Arc<dyn Fn(Vec<u8>) -> NatsortOutType + Send + Sync>;
pub type NumKeyFn = Arc<dyn Fn(NumInput) -> NatsortOutType + Send + Sync>;

/// Computes the natural-sort key for a single value.
///
/// Accepted input types: `String`, `std::path::PathBuf`, `Vec<u8>`,
/// `i64`, `f64`, or `Vec<Box<dyn Any + Send + Sync>>` (a nested
/// list/tuple of any of the above, compared recursively -- this recursion
/// is what the original port's broken `clone_box` stub silently defeated).
/// Anything else is treated as a missing/`None` value and sorts
/// consistently to one end (matching `ns.NANLAST`).
pub fn natsort_key(
    val: NatsortInType,
    key: Option<&KeyType>,
    string_func: &StringKeyFn,
    bytes_func: &BytesKeyFn,
    num_func: &NumKeyFn,
) -> NatsortOutType {
    let val: NatsortInType = match key {
        Some(k) => k(val),
        None => val,
    };

    let val = match val.downcast::<String>() {
        Ok(s) => return string_func(&s),
        Err(v) => v,
    };
    let val = match val.downcast::<std::path::PathBuf>() {
        Ok(p) => return string_func(&p.to_string_lossy()),
        Err(v) => v,
    };
    let val = match val.downcast::<Vec<u8>>() {
        Ok(b) => return bytes_func(*b),
        Err(v) => v,
    };
    let val = match val.downcast::<i64>() {
        Ok(i) => return num_func(NumInput::I64(*i)),
        Err(v) => v,
    };
    let val = match val.downcast::<f64>() {
        Ok(f) => return num_func(NumInput::F64(*f)),
        Err(v) => v,
    };
    let val = match val.downcast::<Vec<Box<dyn Any + Send + Sync>>>() {
        Ok(list) => {
            return list
                .into_iter()
                .map(|item| {
                    KeyPart::Nested(natsort_key(item, None, string_func, bytes_func, num_func))
                })
                .collect();
        }
        Err(v) => v,
    };
    let _ = val;
    num_func(NumInput::None)
}

/// Decodes a value if it's bytes, using `encoding` (currently only
/// UTF-8-compatible decoding is supported -- non-UTF-8-decodable bytes
/// are passed through unchanged rather than panicking).
pub fn do_decoding(s: Box<dyn Any + Send + Sync>, _encoding: &str) -> Box<dyn Any + Send + Sync> {
    if let Some(b) = s.downcast_ref::<Vec<u8>>() {
        if let Ok(decoded) = String::from_utf8(b.clone()) {
            return Box::new(decoded);
        }
        return s;
    }
    s
}
