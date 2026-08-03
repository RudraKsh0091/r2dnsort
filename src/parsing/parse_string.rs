//! Splits a string into alternating text/number chunks and assembles the
//! final sort-key tuple for it.
//! <- natsort/natsort_key.py (string branch)

use crate::ns_enum::{NSType, Ns};
use crate::numtype::KeyPart;
use crate::regex::regex_chooser;
use crate::transform::final_data_transform::FinalTransformer;
use crate::transform::input_string_transform::StrToStr;
use crate::transform::string_component_transform::StrTransformer;
use std::path::PathBuf;

pub type StrParser = Box<dyn Fn(&str) -> Vec<KeyPart> + Send + Sync>;
pub type PathSplitter = Box<dyn Fn(&str) -> Vec<Vec<KeyPart>> + Send + Sync>;

pub fn parse_string_factory(
    alg: NSType,
    sep: String,
    input_transform: StrToStr,
    component_transform: StrTransformer,
    final_transform: FinalTransformer,
) -> StrParser {
    let alg = Ns(alg);
    let normalize_input = normalize_input_factory(alg.0);
    let compose_input: StrToStr = if alg.contains(Ns::LOCALEALPHA) {
        compose_input_factory(alg.0)
    } else {
        Box::new(|x| x.to_string())
    };
    let regex = regex_chooser(alg.0);

    Box::new(move |x: &str| {
        // NFD/NFKD-normalize first (matches Python's own upfront
        // normalization step), remembering the pre-transform original
        // (used by ns.UNGROUPLETTERS to look at the *actual* first
        // character of the string, unaffected by case transforms).
        let normalized = normalize_input(x);
        let original = normalized.clone();
        let cased = input_transform(&normalized);
        let composed = compose_input(&cased);

        let pieces: Vec<String> = split_keeping_matches(&regex, &composed)
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

        let components = component_transform(pieces);
        let with_seps = sep_inserter(components, &sep);
        final_transform(with_seps, &original)
    })
}

/// Rust's `Regex::split` discards the matched text and returns only the
/// text *between* matches -- unlike Python's `re.split()`, which, when the
/// pattern contains a capturing group (as every natsort number regex
/// does), interleaves the captured matches back into the result. Since
/// natsort's whole approach depends on keeping the numbers it splits on
/// (not discarding them), we replicate Python's behavior explicitly:
/// alternating [text, match, text, match, ..., text].
fn split_keeping_matches(re: &regex::Regex, s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut last = 0;
    for m in re.find_iter(s) {
        result.push(s[last..m.start()].to_string());
        result.push(m.as_str().to_string());
        last = m.end();
    }
    result.push(s[last..].to_string());
    result
}

fn normalize_input_factory(alg: NSType) -> StrToStr {
    use unicode_normalization::UnicodeNormalization;
    if Ns(alg).contains(Ns::COMPATIBILITYNORMALIZE) {
        Box::new(|x: &str| x.nfkd().collect::<String>())
    } else {
        Box::new(|x: &str| x.nfd().collect::<String>())
    }
}

fn compose_input_factory(alg: NSType) -> StrToStr {
    use unicode_normalization::UnicodeNormalization;
    if Ns(alg).contains(Ns::COMPATIBILITYNORMALIZE) {
        Box::new(|x: &str| x.nfkc().collect::<String>())
    } else {
        Box::new(|x: &str| x.nfc().collect::<String>())
    }
}

/// Ensures adjacent numeric components (which would otherwise merge into
/// one comparison operand) are kept separate by interleaving `sep`
/// markers, mirroring Python natsort's own `_sep_inserter`.
fn sep_inserter(items: Vec<KeyPart>, sep: &str) -> Vec<KeyPart> {
    let mut result = Vec::with_capacity(items.len() + 1);
    let mut prev_is_num = false;

    for item in items {
        let is_num = matches!(item, KeyPart::Num(_));
        if is_num && (prev_is_num || result.is_empty()) {
            result.push(KeyPart::Str(sep.to_string()));
        }
        prev_is_num = is_num;
        result.push(item);
    }
    result
}

pub fn parse_path_factory(str_split: StrParser) -> PathSplitter {
    Box::new(move |x: &str| {
        let parts = path_splitter(x);
        parts.iter().map(|p| str_split(p)).collect()
    })
}

pub fn path_splitter(s: &str) -> Vec<String> {
    let p = PathBuf::from(s);
    let mut parts: Vec<String> = p
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    if parts.is_empty() {
        parts.push(s.to_string());
    }
    parts
}
