//! public API: natsorted, natsort_keygen, realsorted, humansorted, os_sorted, natsort_key, ns

pub mod compat;
pub mod keygen;
pub mod main_api;
pub mod natsort_key;
pub mod natsorted;
pub mod ns_enum;
pub mod numtype;
pub mod os_sorted;
pub mod parsing;
pub mod regex;
pub mod transform;
pub mod utils;

#[cfg(feature = "python-ext")]
pub mod python;

pub use keygen::natsort_keygen;
pub use main_api::{humansorted, os_sorted_wrapper as os_sorted, realsorted};
pub use natsort_key::natsort_key;
pub use natsorted::natsorted;
pub use ns_enum::{NSType, Ns, NS_DUMB};
pub use numtype::{compare_key_vec, KeyPart, Num};
pub use regex::regex_chooser;
pub use utils::do_decoding;

use std::any::Any;

/// A user-supplied "sort by this" key-extraction callback: given an item
/// of type `T`, returns the value that should actually be natural-sorted
/// in its place. Aliased here (rather than spelled out at every call
/// site) purely to keep signatures readable.
pub type KeyFn<T> = Box<dyn Fn(&T) -> Box<dyn Any + Send + Sync> + Send + Sync>;

pub fn as_ascii(s: Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync> {
    do_decoding(s, "ascii")
}

pub fn as_utf8(s: Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync> {
    do_decoding(s, "utf-8")
}

pub fn decoder(
    encoding: String,
) -> Box<dyn Fn(Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync> + Send + Sync> {
    Box::new(move |s| do_decoding(s, &encoding))
}

pub fn numeric_regex_chooser(alg: NSType) -> String {
    let re = regex_chooser(alg);
    let pat = re.as_str().to_string();
    if pat.starts_with('(') && pat.ends_with(')') {
        pat[1..pat.len() - 1].to_string()
    } else {
        pat
    }
}

pub fn order_by_index<T: Clone>(seq: Vec<T>, index: Vec<usize>) -> Vec<T> {
    index
        .into_iter()
        .filter_map(|i| seq.get(i).cloned())
        .collect()
}

pub fn index_natsorted<T: Clone + Send + Sync + 'static>(
    seq: Vec<T>,
    key: Option<crate::KeyFn<T>>,
    reverse: bool,
    alg: NSType,
) -> Vec<usize> {
    let natsort_key_fn = natsort_keygen(None, alg);
    let mut indexed: Vec<(usize, Vec<KeyPart>)> = seq
        .into_iter()
        .enumerate()
        .map(|(i, item)| {
            let key_val = match &key {
                Some(k) => k(&item),
                None => Box::new(item) as Box<dyn Any + Send + Sync>,
            };
            let sort_key = natsort_key_fn(key_val);
            (i, sort_key)
        })
        .collect();

    indexed.sort_by(|a, b| {
        let ord = compare_key_vec(&a.1, &b.1);
        if reverse {
            ord.reverse()
        } else {
            ord
        }
    });

    indexed.into_iter().map(|(i, _)| i).collect()
}

pub fn index_humansorted<T: Clone + Send + Sync + 'static>(
    seq: Vec<T>,
    key: Option<crate::KeyFn<T>>,
    reverse: bool,
    alg: NSType,
) -> Vec<usize> {
    index_natsorted(seq, key, reverse, (Ns::LOCALE | Ns(alg)).0)
}

pub fn index_realsorted<T: Clone + Send + Sync + 'static>(
    seq: Vec<T>,
    key: Option<crate::KeyFn<T>>,
    reverse: bool,
    alg: NSType,
) -> Vec<usize> {
    index_natsorted(seq, key, reverse, (Ns::REAL | Ns(alg)).0)
}
