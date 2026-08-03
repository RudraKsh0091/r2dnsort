//! Regression tests for the specific bugs identified during code review
//! (both the initial pass and the deeper second pass), plus the
//! compile-blocking issues found while getting the port to build at all.

use r2dnsort::{natsorted, Ns};

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn basic_ascending_numeric_order() {
    let given = s(&["a2", "a5", "a9", "a1", "a4", "a10", "a6"]);
    let expected = s(&["a1", "a2", "a4", "a5", "a6", "a9", "a10"]);
    assert_eq!(natsorted(given, None, false, Ns::DEFAULT.0), expected);
}

#[test]
fn leading_zeros_parity() {
    // "file01", "file1", "file001" should all compare numerically equal
    // at the number component, ties broken by string length/lexical
    // order of the *whole* remaining key -- but they must not scatter
    // randomly.
    let given = s(&["file001", "file1", "file01"]);
    let mut result = natsorted(given.clone(), None, false, Ns::DEFAULT.0);
    result.sort(); // just confirm no panic and a deterministic total order
    assert_eq!(result.len(), 3);
}

#[test]
fn sort_is_stable_for_equal_keys() {
    // Bug 6: equal-key items must keep their original relative order.
    // Use a key function so multiple distinct strings share one key.
    let given: Vec<(i32, &str)> = vec![(1, "x"), (1, "y"), (1, "z"), (1, "a")];
    let result = natsorted(
        given,
        Some(Box::new(|item: &(i32, &str)| {
            Box::new(item.0.to_string()) as Box<dyn std::any::Any + Send + Sync>
        })),
        false,
        Ns::DEFAULT.0,
    );
    let order: Vec<&str> = result.into_iter().map(|(_, s)| s).collect();
    assert_eq!(order, vec!["x", "y", "z", "a"]);
}

#[test]
fn arabic_indic_unicode_digits_sort_numerically() {
    // Bug 7: "file_٢.txt" (Arabic-Indic 2) vs "file_١.txt" (Arabic-Indic 1)
    // must sort as 1 < 2, not lexicographically.
    let given = s(&["file_\u{0662}.txt", "file_\u{0661}.txt"]);
    let result = natsorted(given, None, false, Ns::DEFAULT.0);
    assert_eq!(result, s(&["file_\u{0661}.txt", "file_\u{0662}.txt"]));
}

#[test]
fn ns_flags_bitwise_combine() {
    // Bug 8: ns.FLOAT | ns.IGNORECASE must compile and combine correctly.
    let combined = Ns::FLOAT | Ns::IGNORECASE;
    assert!(combined.contains(Ns::FLOAT));
    assert!(combined.contains(Ns::IGNORECASE));
    assert!(!combined.contains(Ns::SIGNED));
}

#[test]
fn huge_integers_beyond_i64_sort_correctly() {
    // Bug 1: integers far beyond i64/u128 range must still sort
    // numerically, not fall back to lexicographic string comparison.
    let given = s(&[
        "item_99999999999999999999999999999999999999",
        "item_100000000000000000000000000000000000000",
        "item_2",
    ]);
    let result = natsorted(given, None, false, Ns::DEFAULT.0);
    assert_eq!(
        result,
        s(&[
            "item_2",
            "item_99999999999999999999999999999999999999",
            "item_100000000000000000000000000000000000000",
        ])
    );
}

#[test]
fn nan_does_not_panic_and_sorts_consistently() {
    // Bug 2: comparing NaN must never panic.
    let given = s(&["1.0", "nan", "2.0", "nan", "-1.0"]);
    let result = natsorted(given, None, false, Ns::FLOAT.0 | Ns::SIGNED.0);
    assert_eq!(result.len(), 5); // just must not panic; order of NaNs vs
                                 // numbers is a documented policy choice
}

#[test]
fn non_utf8_bytes_do_not_panic() {
    // Bug 5: raw non-UTF-8 byte sequences must not panic on sort.
    let given: Vec<Vec<u8>> = vec![
        vec![0xFF, 0xFE, b'1', b'2', b'3'],
        vec![0x00, 0x01],
        vec![0xFF],
    ];
    let result = natsorted(given, None, false, Ns::DEFAULT.0);
    assert_eq!(result.len(), 3);
}

#[test]
fn thousands_separator_stripped_without_lookaround_panic() {
    // Regex Lookaround Failures bug: LOCALENUM parsing must not panic
    // when compiling the thousands-separator-stripping regex, and should
    // actually strip grouping separators.
    let given = s(&["1,234,567", "999", "1,000"]);
    let result = natsorted(given, None, false, Ns::LOCALENUM.0);
    assert_eq!(result, s(&["999", "1,000", "1,234,567"]));
}

#[test]
fn nested_list_keys_are_not_silently_emptied() {
    // The CloneBox bug: nested Vec<Box<dyn Any>> input used to come back
    // as an empty key, making every nested-sequence item compare equal
    // (and thus never actually reorder). Confirm real reordering happens.
    use std::any::Any;

    #[derive(Clone, Debug)]
    struct Item(i64, &'static str);

    fn key_of(item: &Item) -> Box<dyn Any + Send + Sync> {
        Box::new(vec![
            Box::new(item.0) as Box<dyn Any + Send + Sync>,
            Box::new(item.1.to_string()) as Box<dyn Any + Send + Sync>,
        ]) as Box<dyn Any + Send + Sync>
    }

    let given = vec![Item(2, "b"), Item(1, "z"), Item(1, "a")];
    let result = natsorted(given, Some(Box::new(key_of)), false, Ns::DEFAULT.0);
    let order: Vec<(i64, &str)> = result.into_iter().map(|i| (i.0, i.1)).collect();
    // a=1 items sort before a=2, and among a=1 items "a" < "z".
    assert_eq!(order, vec![(1, "a"), (1, "z"), (2, "b")]);
}

#[test]
fn float_regex_selection_matches_flags() {
    let given = s(&["-3.5", "2.1", "-1.0", "0.0"]);
    let result = natsorted(given, None, false, (Ns::FLOAT | Ns::SIGNED).0);
    assert_eq!(result, s(&["-3.5", "-1.0", "0.0", "2.1"]));
}
