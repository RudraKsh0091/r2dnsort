//! File-browser-style path sort (mirrors OS "natural" filename ordering).
//! <- natsort/natsort.py::os_sorted / os_sort_keygen

use crate::keygen::natsort_keygen;
use crate::ns_enum::Ns;
use crate::numtype::{compare_key_vec, KeyPart};
use crate::utils::NatsortInType;
use std::any::Any;

pub type OsSortKeyType = std::sync::Arc<dyn Fn(NatsortInType) -> Vec<KeyPart> + Send + Sync>;

pub fn os_sort_keygen() -> OsSortKeyType {
    // A true Windows build would call `StrCmpLogicalW` via the Win32 API
    // for byte-for-byte parity with Explorer's sort order; that binding
    // is a platform-specific follow-up. On all platforms (including
    // Windows, for now) this uses the same locale+path+ignorecase
    // fallback natsort itself uses on POSIX.
    natsort_keygen(None, (Ns::LOCALE | Ns::PATH | Ns::IGNORECASE).0)
}

pub fn os_sorted<T: Clone + Send + Sync + std::fmt::Debug + 'static>(
    seq: Vec<T>,
    key: Option<crate::KeyFn<T>>,
    reverse: bool,
    presort: bool,
) -> Vec<T> {
    let mut seq = seq;
    if presort {
        seq.sort_by(|a, b| {
            let a_str = format!("{:?}", a);
            let b_str = format!("{:?}", b);
            a_str.cmp(&b_str)
        });
        if reverse {
            seq.reverse();
        }
    }

    let os_key = os_sort_keygen();
    let mut indexed: Vec<(T, Vec<KeyPart>)> = seq
        .into_iter()
        .map(|item| {
            let key_val = match &key {
                Some(k) => k(&item),
                None => Box::new(item.clone()) as Box<dyn Any + Send + Sync>,
            };
            let sort_key = os_key(key_val);
            (item, sort_key)
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

    indexed.into_iter().map(|(item, _)| item).collect()
}
