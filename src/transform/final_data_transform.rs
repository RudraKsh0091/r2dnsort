//! Final assembly step of a string sort key.
//! <- natsort/natsort_key.py (final tuple assembly, `ns.UNGROUPLETTERS`)

use crate::compat::locale::StrOrBytes;
use crate::ns_enum::{NSType, Ns, NS_DUMB};
use crate::numtype::KeyPart;

pub type FinalTransform = Vec<KeyPart>;
pub type FinalTransformer = Box<dyn Fn(Vec<KeyPart>, &str) -> FinalTransform + Send + Sync>;

/// Create a function that (optionally) prepends a case-marker component
/// to the split key, implementing `ns.UNGROUPLETTERS` /
/// `ns.CAPITALFIRST`: strings that differ only in the case of their first
/// letter are ordered by that first letter's case.
pub fn final_data_transform_factory(
    alg: NSType,
    sep: StrOrBytes,
    pre_sep: &str,
) -> FinalTransformer {
    let alg = Ns(alg);
    if !(alg.contains(Ns::UNGROUPLETTERS) && alg.contains(Ns::LOCALEALPHA)) {
        return Box::new(move |split_val: Vec<KeyPart>, _val: &str| split_val);
    }

    let swap = alg.contains(NS_DUMB) && alg.contains(Ns::LOWERCASEFIRST);
    let pre_sep = pre_sep.to_string();

    Box::new(move |split_val: Vec<KeyPart>, val: &str| {
        if split_val.is_empty() {
            return vec![KeyPart::Nested(Vec::new()), KeyPart::Nested(Vec::new())];
        }
        // If the key already starts with the separator marker (i.e. the
        // string began with a number, so there's no leading letter to
        // case-classify), use the neutral pre_sep marker instead of
        // trying to read a first character.
        let first_is_sep = matches!(&split_val[0], KeyPart::Str(s) if *s == sep);

        let marker = if first_is_sep {
            pre_sep.clone()
        } else {
            let first_char = val.chars().next().unwrap_or('\0');
            if swap {
                if first_char.is_lowercase() {
                    first_char.to_uppercase().collect::<String>()
                } else {
                    first_char.to_lowercase().collect::<String>()
                }
            } else {
                first_char.to_string()
            }
        };

        vec![
            KeyPart::Nested(vec![KeyPart::Str(marker)]),
            KeyPart::Nested(split_val),
        ]
    })
}
