//! Transforms applied to each split string component: either converts it
//! to a number, or (if it isn't numeric) applies grouping/locale string
//! transforms to it.
//! <- transform/string_component_transform.py

use crate::compat::fastnumbers::{try_float, try_int};
use crate::compat::locale::get_strxfrm;
use crate::ns_enum::{NSType, Ns, NS_DUMB};
use crate::numtype::KeyPart;

pub type StrTransformer = Box<dyn Fn(Vec<String>) -> Vec<KeyPart> + Send + Sync>;

/// Interleave the lowercase and original-case form of each character.
/// Used by `ns.GROUPLETTERS` so that, e.g., "Case" and "case" don't
/// compare as exactly equal (matching Python natsort's `groupletters`).
fn groupletters(x: &str) -> String {
    let mut result = String::with_capacity(x.len() * 2);
    for c in x.chars() {
        for lc in c.to_lowercase() {
            result.push(lc);
        }
        result.push(c);
    }
    result
}

/// Create a function to either transform a string or convert it to a
/// number, for every split component of an item.
pub fn string_component_transform_factory(alg: NSType) -> StrTransformer {
    let alg = Ns(alg);
    let use_locale = alg.contains(Ns::LOCALEALPHA);
    let dumb = alg.contains(NS_DUMB);
    let group_letters = alg.contains(Ns::GROUPLETTERS) || (use_locale && dumb);

    let mut func_chain: Vec<Box<dyn Fn(&str) -> String + Send + Sync>> = Vec::new();
    if group_letters {
        func_chain.push(Box::new(groupletters));
    }
    if use_locale {
        let strxfrm = get_strxfrm();
        func_chain.push(Box::new(move |x: &str| strxfrm(x)));
    }
    let on_fail = chain_str_functions(func_chain);

    let is_float = alg.contains(Ns::FLOAT);

    Box::new(move |items: Vec<String>| {
        items
            .into_iter()
            .map(|s| {
                if is_float {
                    try_float(&s, Some(on_fail.as_ref()))
                } else {
                    try_int(&s, Some(on_fail.as_ref()))
                }
            })
            .collect()
    })
}

fn chain_str_functions(
    functions: Vec<Box<dyn Fn(&str) -> String + Send + Sync>>,
) -> Box<dyn Fn(&str) -> String + Send + Sync> {
    if functions.is_empty() {
        return Box::new(|x: &str| x.to_string());
    }
    Box::new(move |x: &str| {
        let mut res = x.to_string();
        for f in &functions {
            res = f(&res);
        }
        res
    })
}
