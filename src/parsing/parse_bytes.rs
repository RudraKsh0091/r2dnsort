//! Byte-string (`Vec<u8>`) parsing path.
//! <- natsort/natsort_key.py (bytes branch)
//!
//! Python's natsort can natural-sort raw `bytes`, including arbitrary
//! non-UTF-8 sequences, without ever raising a `UnicodeDecodeError`.
//! Converting `&[u8]` to `&str` via `std::str::from_utf8(..).unwrap()`
//! would panic on such input ("Non-UTF-8 Byte Handling" bug), so this
//! implementation compares raw bytes directly and never performs a
//! panicking UTF-8 validation.

use crate::ns_enum::{NSType, Ns};
use crate::numtype::KeyPart;

pub type BytesTransform = Vec<KeyPart>;
pub type BytesTransformer = Box<dyn Fn(Vec<u8>) -> BytesTransform + Send + Sync>;

pub fn parse_bytes_factory(alg: NSType) -> BytesTransformer {
    let alg = Ns(alg);
    let ignorecase = alg.contains(Ns::IGNORECASE);
    let path = alg.contains(Ns::PATH);

    Box::new(move |x: Vec<u8>| {
        let bytes = if ignorecase {
            x.to_ascii_lowercase()
        } else {
            x
        };
        let part = KeyPart::Bytes(bytes);
        if path {
            vec![KeyPart::Nested(vec![part])]
        } else {
            vec![part]
        }
    })
}
