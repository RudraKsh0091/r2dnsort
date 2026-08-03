//! Simple std-only micro-benchmark (maps to profile_natsorted.py).
//!
//! Deliberately dependency-free (no `criterion`): pulling in criterion's
//! current dependency chain requires a newer rustc (1.80+) than some
//! build environments have available, and that's an unrelated concern
//! from whether the library itself is correct. Run with:
//!   cargo run --release --bin natsort_bench
//! or wire it into `criterion` yourself if your toolchain supports it --
//! `bench_natsorted` below is trivially reusable either way.

use r2dnsort::{natsorted, Ns};
use std::time::Instant;

fn bench_natsorted(iterations: u32) -> std::time::Duration {
    let data = vec!["a2", "a5", "a9", "a1", "a4", "a10", "a6"];
    let data: Vec<String> = data.into_iter().map(|s| s.to_string()).collect();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = natsorted(data.clone(), None, false, Ns::DEFAULT.0);
    }
    start.elapsed()
}

fn main() {
    let iterations = 100_000;
    let elapsed = bench_natsorted(iterations);
    println!(
        "natsorted_default: {} iterations in {:?} ({:.3} us/iter)",
        iterations,
        elapsed,
        elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64
    );
}
