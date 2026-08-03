# r2dnsort

Simple yet flexible natural sorting, in Rust — with Python bindings.

This started as a line-by-line port of the Python [`natsort`](https://github.com/SethMMorton/natsort)
package by Seth M. Morton, then had its architecture reworked (a broken
`Box<dyn Any>`/`CloneBox` sort-key design replaced with a clean `KeyPart`
enum), several real bugs fixed (a `Regex::split()` data-loss bug that
silently made `natsorted()` a no-op, NaN panics, big-int overflow,
non-UTF-8 byte handling, Unicode numeral parsing), and a from-scratch
Python extension module (via `pyo3`/`maturin`) built on top.

## Rust usage

```rust
use r2dnsort::{natsorted, Ns};

fn main() {
    let items = vec!["a2", "a5", "a9", "a1", "a10"]
        .into_iter().map(String::from).collect();
    let sorted = natsorted(items, None, false, Ns::DEFAULT.0);
    println!("{:?}", sorted); // ["a1", "a2", "a5", "a9", "a10"]
}
```

Also available: `humansorted`, `realsorted`, `os_sorted`, `natsort_keygen`,
`index_natsorted`, `order_by_index`, and the full `ns` flag set
(`FLOAT`, `SIGNED`, `PATH`, `LOCALE`, `IGNORECASE`, `GROUPLETTERS`, ...).
Accepted item types include `String`, `PathBuf`, `Vec<u8>`, `i64`, `f64`,
and nested `Vec<Box<dyn Any>>` (compared recursively, like Python tuples).

## Python usage

Built via [`maturin`](https://www.maturin.rs/); see `TESTING.md` for the
full build workflow. Once installed:

```python
import r2dnsort

r2dnsort.natsorted(["a2", "a5", "a9", "a1", "a10"])
# -> ['a1', 'a2', 'a5', 'a9', 'a10']

r2dnsort.natsorted([3, 1.5, "a10", "a2"], alg=r2dnsort.ns.FLOAT)

sorted(["a2", "a10", "a1"], key=r2dnsort.natsort_keygen())

r2dnsort.os_sorted(["file10.txt", "file2.txt", "file1.txt"])
```

Exposed functions: `natsorted`, `humansorted`, `realsorted`, `os_sorted`,
`natsort_keygen`, `index_natsorted`, `order_by_index`, plus the `ns` flag
submodule. Inputs may be `str`, `bytes`/`bytearray`, `int`, `float`,
`None`, or nested `list`/`tuple` — matching what the Rust core supports.

### Package naming

The importable module is `r2dnsort` (not `natsort`), so it installs
alongside the real PyPI `natsort` package without conflict — this is
intentionally *not* a drop-in replacement.

## Status / known limitations

- Locale collation, Unicode casefolding, and Windows path-sort
  (`StrCmpLogicalW` parity) are documented approximations, not full
  parity with a real OS locale/ICU binding.
- `Num::Big` (arbitrary-precision integers) are exposed to Python as
  decimal strings inside `natsort_keygen()` tuples, since Python's `int`
  bridging for values beyond `i64` isn't wired through PyO3 here.

See `TESTING.md` for how to verify both the Rust crate and the Python
extension, and `CHANGELOG.MD` for the fix/feature history.
