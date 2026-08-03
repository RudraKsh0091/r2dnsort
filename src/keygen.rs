//! `natsort_keygen()`: builds a reusable natural-sort key function for a
//! given `ns` flag combination.
//! <- natsort/natsort.py::natsort_keygen

use crate::compat::locale::{
    dumb_sort, null_string, null_string_locale, null_string_locale_max, null_string_max,
};
use crate::ns_enum::{NSType, Ns, NS_DUMB};
use crate::parsing::parse_number::NumInput;
use crate::utils::{
    final_data_transform_factory, input_string_transform_factory, natsort_key, parse_bytes_factory,
    parse_number_or_none_factory, parse_path_factory, parse_string_factory,
    string_component_transform_factory, BytesKeyFn, KeyType, NatsortInType, NatsortOutType,
    NumKeyFn, StringKeyFn,
};
use std::sync::Arc;

pub type NatsortKeyType = Arc<dyn Fn(NatsortInType) -> NatsortOutType + Send + Sync>;

pub fn natsort_keygen(key: Option<KeyType>, alg: NSType) -> NatsortKeyType {
    let alg = Ns(alg);

    // If the "locale" is broken/dumb, fall back to the swap-case
    // workaround (mirrors Python's own dumb-sort detection).
    let mut alg = alg;
    if alg.contains(Ns::LOCALEALPHA) && dumb_sort() {
        alg |= NS_DUMB;
    }

    let (sep, pre_sep) = if alg.contains(Ns::NUMAFTER) {
        if alg.contains(Ns::LOCALEALPHA) {
            (null_string_locale_max(), null_string_locale_max())
        } else {
            (null_string_max().to_string(), null_string_max().to_string())
        }
    } else if alg.contains(Ns::LOCALEALPHA) {
        (null_string_locale(), null_string_locale())
    } else {
        (null_string().to_string(), null_string().to_string())
    };

    let input_transform = input_string_transform_factory(alg.0);
    let component_transform = string_component_transform_factory(alg.0);
    let final_transform = final_data_transform_factory(alg.0, sep.clone(), &pre_sep);

    let base_string_func = parse_string_factory(
        alg.0,
        sep.clone(),
        input_transform,
        component_transform,
        final_transform,
    );

    let string_func: StringKeyFn = if alg.contains(Ns::PATH) {
        let path_func = parse_path_factory(base_string_func);
        Arc::new(move |x: &str| {
            path_func(x)
                .into_iter()
                .map(crate::numtype::KeyPart::Nested)
                .collect()
        })
    } else {
        Arc::new(move |x: &str| base_string_func(x))
    };

    let base_bytes_func = parse_bytes_factory(alg.0);
    let bytes_func: BytesKeyFn = Arc::new(move |b: Vec<u8>| base_bytes_func(b));

    let base_num_func = parse_number_or_none_factory(alg.0, sep, &pre_sep);
    let num_func: NumKeyFn = Arc::new(move |n: NumInput| base_num_func(n));

    Arc::new(move |val: NatsortInType| {
        natsort_key(val, key.as_ref(), &string_func, &bytes_func, &num_func)
    })
}
