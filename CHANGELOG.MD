# Changelog

## 8.4.0 - r2dnsort (renamed from natsort-rs)
- Renamed crate/package from `natsort` / `natsort-rs` to `r2dnsort`
  (Cargo package, binary, PyPI distribution name, and importable Python
  module) to avoid colliding with the real PyPI `natsort` package.
- Expanded the Python bindings (`src/python.rs`) from a single
  `natsorted(seq: List[str])` function to the fuller original-`natsort`
  surface: `natsorted`, `humansorted`, `realsorted`, `os_sorted`,
  `natsort_keygen`, `index_natsorted`, `order_by_index`, plus `ns` flags.
  Accepted input types expanded from strings-only to `str`, `bytes`/
  `bytearray`, `int`, `float`, `None`, and nested `list`/`tuple`.
- `natsort_keygen()` now returns a genuine Python callable
  (`sorted(seq, key=r2dnsort.natsort_keygen())` works directly), backed
  by the same Rust key-computation pipeline as `natsorted()`.
- Added a `KeyFn<T>` type alias to clean up repeated complex closure
  types flagged by `cargo clippy` across `lib.rs`/`main_api.rs`/
  `natsorted.rs`/`os_sorted.rs`.
- Fixed a `clippy::let_and_return` lint in the case-folding logic of
  `final_data_transform.rs`.
- Ran `cargo fmt` across the whole tree.
- Added `LICENSE` (MIT) and `.gitignore`.

## 8.4.0 - Rust Port
- Initial Rust conversion from Python natsort.
- Preserved all original Python logic, bugs and all.
- (Later) Found and fixed the actual showstopper: `Regex::split()`
  discards matched text in Rust (unlike Python's `re.split()`), so every
  parsed number was vanishing and `natsorted()` silently no-op'd.
- Replaced the original `Box<dyn Any>` + broken `CloneBox` sort-key
  representation (which silently emptied nested list/tuple keys) with
  the `KeyPart`/`Num` enum in `numtype.rs`.
- Fixed NaN panic safety, big-int overflow (`num-bigint`), non-UTF-8
  byte handling, unsupported regex lookaround, and Unicode numeral
  parsing (Arabic-Indic, etc.).
- Added `tests/test_bugfixes.rs`: one regression test per review-doc bug.
- Built real PyO3 Python bindings behind a `python-ext` Cargo feature.
