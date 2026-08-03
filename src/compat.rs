//! Compatibility shims for the bits of Python's standard library / optional
//! C-extensions (`locale`, `fastnumbers`) that natsort leans on.
//! <- compat/locale.py, compat/fastnumbers.py

pub mod fastnumbers;
pub mod locale;
