//! CLI entry point, mirrors `python -m natsort`

use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    // Simplified CLI preserving Python __main__.py structure
    for arg in &args {
        println!("{}", arg);
    }
    if args.is_empty() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        let lines: Vec<&str> = buffer.lines().collect();
        for line in &lines {
            println!("{}", line);
        }
    }
}
