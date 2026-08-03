# Testing this port

There are two independent things to verify: (1) the Rust crate itself, and
(2) the Python extension module built from it. Do both — passing (1)
does not guarantee (2) works, since the Python binding is separate glue
code.

## 1. Rust crate (`cargo test`)

```bash
cargo build --all-targets   # lib, bin, tests, the std-only bench binary
cargo test                  # runs the full suite, including tests/test_bugfixes.rs
cargo clippy --all-targets  # lints (a handful of pre-existing cosmetic
                             # type_complexity / format_collect nits remain
                             # in stub tests and are not correctness issues)
cargo fmt --check           # formatting
```

You should see 22 test binaries, all `ok`, zero failures, zero warnings
from `cargo build`/`cargo test`. `tests/test_bugfixes.rs` specifically
regression-tests every bug from the original review docs (stability,
Unicode digits, ns flag combining, bigint overflow, NaN safety, non-UTF-8
bytes, thousands-separator regex, and the nested-list key bug).

This does **not** touch Python at all — it's pure `cargo`.

## 2. Python extension (`import r2dnsort`)

The Rust crate has no Python bindings by default — `cargo build` never
compiles `src/python.rs` (it's behind a `python-ext` feature flag so it
can't accidentally break the plain Rust build/tests). Building the actual
importable module needs `maturin`.

### Windows (CMD)

```cmd
python -m venv .venv
.venv\Scripts\activate
pip install maturin

python -m maturin develop --features python-ext --release
```

(`maturin develop` needs an active virtualenv/conda env — that's what
`.venv\Scripts\activate` provides. If you'd rather skip the venv:
`python -m maturin build --features python-ext --release` then
`pip install target\wheels\<generated>.whl`.)

### Verify the import (same activated shell)

```cmd
python -c "import r2dnsort; print(r2dnsort.natsorted(['a2','a5','a9','a1','a4','a10','a6']))"
```

Expect: `['a1', 'a2', 'a4', 'a5', 'a6', 'a9', 'a10']`

A fuller smoke test covering the expanded API lives at
`examples/python_smoke_test.py` — run it after `maturin develop`:

```cmd
python examples\python_smoke_test.py
```

It exercises `natsorted`, `humansorted`, `realsorted`, `os_sorted`,
`natsort_keygen`, `index_natsorted`/`order_by_index`, bytes and nested-tuple
input, `reverse=True`, and a custom Python `key=` callable (including on
non-natively-sortable items like `dict`s, which is only valid *with* a
`key=` that extracts a sortable value from them — matching real Python
`sorted()` semantics).

> **Note on a bug found and fixed during this pass:** the pure-Rust
> `natsorted()`/`os_sorted()`/etc. treat `key = None` as "the item is
> already a directly-downcastable type (`String`, `i64`, `f64`,
> `Vec<u8>`, ...)" and box it unchanged. The Python bridge's internal
> `PyVal` wrapper enum is *not* one of those types, so an earlier version
> of `src/python.rs` that passed `key = None` straight through silently
> produced an identical ("missing value") sort key for every item —
> `natsorted()` looked like it ran but actually returned the input
> unchanged. Running `examples/python_smoke_test.py` and comparing
> output to input is exactly what caught this; the fix (`make_key_fn`
> always returns a real converting key function, never `None`) is in
> `src/python.rs`.

### What's exposed to Python

`natsorted`, `humansorted`, `realsorted`, `os_sorted`, `natsort_keygen`,
`index_natsorted`, `order_by_index`, and the `ns` flag submodule. Inputs
may be `str`, `bytes`/`bytearray`, `int`, `float`, `None`, or nested
`list`/`tuple` (recursively converted through the same `natsort_key()`
pipeline the pure-Rust API uses — see `src/python.rs`).

Not yet wired up: dedicated `PathBuf`-typed input from Python (paths
currently just go through as plain strings, which works for `ns.PATH`
sorting but doesn't preserve a distinct Python `Path` type on the way
back out).

### Package naming

The importable module is `r2dnsort`, and the PyPI distribution name is
also `r2dnsort` (see `pyproject.toml`) — deliberately distinct from the
real `natsort` package on PyPI, so both can be installed side by side
without conflict. This is *not* a drop-in replacement for `natsort`.

## 3. Before publishing

- [x] `cargo fmt` — done, whole tree reformatted.
- [x] `cargo clippy --all-targets` — done; remaining warnings are
      cosmetic (`type_complexity` on a couple of closure signatures,
      `format_collect` in `unicode_numbers.rs`, and `assert!(true)` stub
      placeholders in not-yet-written tests) — none are correctness bugs.
- [x] `LICENSE` — added (MIT, matching `Cargo.toml`).
- [x] `.gitignore` — added (`target/`, `.venv/`, `*.pyd`, `*.whl`,
      `__pycache__/`, editor/OS cruft).
- [ ] `Cargo.lock` — currently committed. Since this crate also ships a
      binary (`[[bin]] r2dnsort`), committing it is reasonable/common;
      if you want pure-library conventions instead, add it to
      `.gitignore` and re-generate on each build.
