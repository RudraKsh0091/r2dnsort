//! natsorted()/humansorted()/realsorted()/os_sorted()
//! top-level convenience wrappers
//! <- test_main.py, test_natsorted_convenience.rs

use crate::natsorted::natsorted;
use crate::ns_enum::{NSType, Ns};
use crate::os_sorted::os_sorted;

pub fn humansorted<T: Clone + Send + Sync + std::fmt::Debug + 'static>(
    seq: Vec<T>,
    key: Option<crate::KeyFn<T>>,
    reverse: bool,
    alg: NSType,
) -> Vec<T> {
    natsorted(seq, key, reverse, (Ns::LOCALE | Ns(alg)).0)
}

pub fn realsorted<T: Clone + Send + Sync + std::fmt::Debug + 'static>(
    seq: Vec<T>,
    key: Option<crate::KeyFn<T>>,
    reverse: bool,
    alg: NSType,
) -> Vec<T> {
    natsorted(seq, key, reverse, (Ns::REAL | Ns(alg)).0)
}

pub fn os_sorted_wrapper<T: Clone + Send + Sync + std::fmt::Debug + 'static>(
    seq: Vec<T>,
    key: Option<crate::KeyFn<T>>,
    reverse: bool,
    presort: bool,
) -> Vec<T> {
    os_sorted(seq, key, reverse, presort)
}
