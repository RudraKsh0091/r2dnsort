//! <- test_natsorted.py

use r2dnsort::{natsorted, Ns};

#[test]
fn test_natsorted_numbers_in_ascending_order() {
    let given = vec!["a2", "a5", "a9", "a1", "a4", "a10", "a6"];
    let expected = vec!["a1", "a2", "a4", "a5", "a6", "a9", "a10"];
    let result: Vec<String> = natsorted(
        given.into_iter().map(|s| s.to_string()).collect(),
        None,
        false,
        Ns::DEFAULT.0,
    )
    .into_iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(result, expected);
}
