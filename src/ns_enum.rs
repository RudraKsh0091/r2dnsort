//! Sort-option bitflags, equivalent to Python natsort's `ns` IntFlag enum.
//! <- ns_enum.py / test_ns_enum.rs
//!
//! `Ns` wraps a `u32` bitmask. It supports `|` combination (via `BitOr`) so
//! callers can write `Ns::FLOAT | Ns::SIGNED` exactly like Python's
//! `ns.FLOAT | ns.SIGNED`, and `.contains(flag)` for membership checks.

use std::ops::{BitOr, BitOrAssign};

pub type NSType = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Ns(pub NSType);

impl Ns {
    // --- Primitive bits (long-form names) ---
    pub const FLOAT: Ns = Ns(0x0001);
    pub const SIGNED: Ns = Ns(0x0002);
    pub const NOEXP: Ns = Ns(0x0004);
    pub const PATH: Ns = Ns(0x0008);
    pub const LOCALEALPHA: Ns = Ns(0x0010);
    pub const LOCALENUM: Ns = Ns(0x0020);
    pub const IGNORECASE: Ns = Ns(0x0040);
    pub const LOWERCASEFIRST: Ns = Ns(0x0080);
    pub const GROUPLETTERS: Ns = Ns(0x0100);
    pub const UNGROUPLETTERS: Ns = Ns(0x0200);
    pub const NANLAST: Ns = Ns(0x0400);
    pub const COMPATIBILITYNORMALIZE: Ns = Ns(0x0800);
    pub const NUMAFTER: Ns = Ns(0x1000);
    pub const PRESORT: Ns = Ns(0x2000);

    // --- Derived / convenience combos ---
    pub const DEFAULT: Ns = Ns(0x0000);
    pub const INT: Ns = Ns(0x0000);
    pub const UNSIGNED: Ns = Ns(0x0000);
    pub const REAL: Ns = Ns(Ns::FLOAT.0 | Ns::SIGNED.0);
    pub const LOCALE: Ns = Ns(Ns::LOCALEALPHA.0 | Ns::LOCALENUM.0);

    // --- Short aliases (mirror Python's ns.I, ns.F, ... aliases) ---
    pub const I: Ns = Ns::INT;
    pub const U: Ns = Ns::UNSIGNED;
    pub const F: Ns = Ns::FLOAT;
    pub const S: Ns = Ns::SIGNED;
    pub const R: Ns = Ns::REAL;
    pub const N: Ns = Ns::NOEXP;
    pub const P: Ns = Ns::PATH;
    pub const LA: Ns = Ns::LOCALEALPHA;
    pub const LN: Ns = Ns::LOCALENUM;
    pub const L: Ns = Ns::LOCALE;
    pub const IC: Ns = Ns::IGNORECASE;
    pub const LF: Ns = Ns::LOWERCASEFIRST;
    pub const G: Ns = Ns::GROUPLETTERS;
    pub const UG: Ns = Ns::UNGROUPLETTERS;
    pub const C: Ns = Ns::UNGROUPLETTERS; // CAPITALFIRST is an alias for UNGROUPLETTERS
    pub const CAPITALFIRST: Ns = Ns::UNGROUPLETTERS;
    pub const NL: Ns = Ns::NANLAST;
    pub const CN: Ns = Ns::COMPATIBILITYNORMALIZE;
    pub const NA: Ns = Ns::NUMAFTER;
    pub const PS: Ns = Ns::PRESORT;

    #[inline]
    pub fn raw(self) -> NSType {
        self.0
    }

    #[inline]
    pub fn contains(self, other: Ns) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for Ns {
    type Output = Ns;
    #[inline]
    fn bitor(self, rhs: Ns) -> Ns {
        Ns(self.0 | rhs.0)
    }
}

impl BitOrAssign for Ns {
    #[inline]
    fn bitor_assign(&mut self, rhs: Ns) {
        self.0 |= rhs.0;
    }
}

/// Extra internal flag (not part of the public `ns` surface in Python) used
/// to mark "dumb sort" mode -- i.e. the locale library is misbehaving (e.g.
/// glibc's infamous "dumb" locale bug) and natsort needs to swap-case input
/// before comparison to work around it. See `compat::locale::dumb_sort`.
pub const NS_DUMB: Ns = Ns(0x4000);
