//! runnable Rust versions of README Quick Examples

use r2dnsort::{natsorted, Ns};

fn main() {
    let a = vec![
        "2 ft 7 in",
        "1 ft 5 in",
        "10 ft 2 in",
        "2 ft 11 in",
        "7 ft 6 in",
    ];
    let a: Vec<String> = a.into_iter().map(|s| s.to_string()).collect();
    let sorted = natsorted(a, None, false, Ns::DEFAULT.0);
    println!("{:?}", sorted);
}
