//! `natsorted()`: sorts a sequence using a natural-sort key.
//! <- natsort/natsort.py::natsorted

use crate::keygen::natsort_keygen;
use crate::ns_enum::{NSType, Ns};
use crate::numtype::compare_key_vec;
use std::any::Any;

pub fn natsorted<T: Clone + Send + Sync + std::fmt::Debug + 'static>(
    seq: Vec<T>,
    key: Option<crate::KeyFn<T>>,
    reverse: bool,
    alg: NSType,
) -> Vec<T> {
    let alg_flags = Ns(alg);
    let mut seq = seq;
    if alg_flags.contains(Ns::PRESORT) {
        seq.sort_by(|a, b| {
            let a_str = format!("{:?}", a);
            let b_str = format!("{:?}", b);
            if reverse {
                b_str.cmp(&a_str)
            } else {
                a_str.cmp(&b_str)
            }
        });
    }

    let natsort_key_fn = natsort_keygen(None, alg);
    let mut indexed: Vec<(T, Vec<crate::numtype::KeyPart>)> = seq
        .into_iter()
        .map(|item| {
            let key_val = match &key {
                Some(k) => k(&item),
                None => Box::new(item.clone()) as Box<dyn Any + Send + Sync>,
            };
            let sort_key = natsort_key_fn(key_val);
            (item, sort_key)
        })
        .collect();

    // `slice::sort_by` is a *stable* sort (Rust never uses an unstable
    // sort here), matching Python's Timsort guarantee that equal-key
    // items keep their original relative order ("Sort Stability" bug).
    indexed.sort_by(|a, b| {
        let ord = compare_key_vec(&a.1, &b.1);
        if reverse {
            ord.reverse()
        } else {
            ord
        }
    });

    indexed.into_iter().map(|(item, _)| item).collect()
}
