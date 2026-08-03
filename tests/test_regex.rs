//! <- test_regex.py

use r2dnsort::ns_enum::Ns;
use r2dnsort::regex::{regex_chooser, NumericalRegularExpressions};

#[test]
fn test_regex_chooser_returns_correct_regular_expression_object() {
    let re = regex_chooser(Ns::I.0);
    assert_eq!(
        re.as_str(),
        NumericalRegularExpressions::int_nosign().as_str()
    );
}
