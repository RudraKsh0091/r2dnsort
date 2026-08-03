//! Python bindings, built only with `--features python-ext` (via
//! `maturin`). Plain `cargo build`/`cargo test` never compile this file,
//! so the existing pure-Rust test suite is completely unaffected by it.
//!
//! Exposes a Pythonic surface close to the original `natsort` package's
//! most-used API: `natsorted`, `humansorted`, `realsorted`, `os_sorted`,
//! `natsort_keygen`, `index_natsorted`, `order_by_index`, plus the `ns`
//! flag constants. Inputs may be strings, bytes, ints, floats, `None`,
//! or nested lists/tuples of the above (mirroring what the pure-Rust
//! `natsort_key()` dispatcher already supports via `NatsortInType`).

use crate::numtype::{KeyPart, Num};
use crate::{
    humansorted as rs_humansorted, natsorted as rs_natsorted, os_sorted as rs_os_sorted,
    realsorted as rs_realsorted, Ns,
};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};
use std::any::Any;
use std::fmt;

/// A Rust-side stand-in for an arbitrary Python value, recursively
/// convertible to/from `PyAny` and into the `Box<dyn Any + Send + Sync>`
/// shape that the core `natsort_key()` dispatcher already understands.
/// This is what lets the same battle-tested key-computation pipeline
/// used by the pure-Rust API also power the Python bindings, instead of
/// re-implementing parsing/comparison a second time here.
#[derive(Clone)]
enum PyVal {
    Str(String),
    Bytes(Vec<u8>),
    Int(i64),
    Float(f64),
    List(Vec<PyVal>),
    None,
    /// Any Python object that isn't one of the natively-sortable shapes
    /// above (e.g. a `dict`, a custom class instance). Not usable as a
    /// direct sort key on its own, but perfectly valid as an *item* in
    /// `seq` when a `key=` callable is supplied to extract a sortable
    /// value from it -- exactly like real Python `sorted(seq, key=...)`.
    Opaque(PyObject),
}

impl fmt::Debug for PyVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PyVal::Str(s) => write!(f, "{:?}", s),
            PyVal::Bytes(b) => write!(f, "{:?}", b),
            PyVal::Int(i) => write!(f, "{}", i),
            PyVal::Float(x) => write!(f, "{}", x),
            PyVal::List(items) => write!(f, "{:?}", items),
            PyVal::None => write!(f, "None"),
            PyVal::Opaque(obj) => Python::with_gil(|py| {
                write!(
                    f,
                    "{}",
                    obj.as_ref(py)
                        .repr()
                        .map(|r| r.to_string())
                        .unwrap_or_else(|_| "<opaque>".to_string())
                )
            }),
        }
    }
}

impl PyVal {
    fn from_pyany(obj: &PyAny) -> PyResult<PyVal> {
        if obj.is_none() {
            return Ok(PyVal::None);
        }
        if let Ok(s) = obj.extract::<String>() {
            return Ok(PyVal::Str(s));
        }
        if let Ok(b) = obj.extract::<Vec<u8>>() {
            // Only routes here for real `bytes`/`bytearray` objects --
            // `String` is checked first above, so this never misfires
            // on plain text.
            if obj.is_instance_of::<PyBytes>() || obj.get_type().name()? == "bytearray" {
                return Ok(PyVal::Bytes(b));
            }
        }
        if let Ok(i) = obj.extract::<i64>() {
            return Ok(PyVal::Int(i));
        }
        if let Ok(f) = obj.extract::<f64>() {
            return Ok(PyVal::Float(f));
        }
        if let Ok(list) = obj.downcast::<PyList>() {
            let items = list
                .iter()
                .map(PyVal::from_pyany)
                .collect::<PyResult<Vec<_>>>()?;
            return Ok(PyVal::List(items));
        }
        if let Ok(tuple) = obj.downcast::<PyTuple>() {
            let items = tuple
                .iter()
                .map(PyVal::from_pyany)
                .collect::<PyResult<Vec<_>>>()?;
            return Ok(PyVal::List(items));
        }
        // Anything else (dict, custom object, ...) isn't natively
        // sortable, but is still a valid *item* in `seq` when a `key=`
        // callable is supplied -- carry it through unchanged rather
        // than erroring up front.
        Ok(PyVal::Opaque(obj.into_py(obj.py())))
    }

    fn to_pyobject(&self, py: Python<'_>) -> PyObject {
        match self {
            PyVal::Str(s) => s.into_py(py),
            PyVal::Bytes(b) => PyBytes::new(py, b).into_py(py),
            PyVal::Int(i) => i.into_py(py),
            PyVal::Float(x) => x.into_py(py),
            PyVal::List(items) => {
                let elems: Vec<PyObject> = items.iter().map(|v| v.to_pyobject(py)).collect();
                PyTuple::new(py, elems).into_py(py)
            }
            PyVal::None => py.None(),
            PyVal::Opaque(obj) => obj.clone_ref(py),
        }
    }

    /// Converts into the `Box<dyn Any + Send + Sync>` shape the core
    /// `natsort_key()` dispatcher downcasts against.
    fn into_any(self) -> Box<dyn Any + Send + Sync> {
        match self {
            PyVal::Str(s) => Box::new(s),
            PyVal::Bytes(b) => Box::new(b),
            PyVal::Int(i) => Box::new(i),
            PyVal::Float(f) => Box::new(f),
            PyVal::List(items) => {
                let boxed: Vec<Box<dyn Any + Send + Sync>> =
                    items.into_iter().map(PyVal::into_any).collect();
                Box::new(boxed)
            }
            // Deliberately not downcastable to any type natsort_key()
            // checks for -- both fall through to the "missing value"
            // path (NumInput::None). For `Opaque`, this only happens if
            // no `key=` callable was given to extract a sortable value
            // first (the normal path routes `Opaque` through the key
            // callable in `make_key_fn`, never through here directly).
            PyVal::None => Box::new(()),
            PyVal::Opaque(_) => Box::new(()),
        }
    }
}

fn pyseq_to_pyvals(seq: &PyAny) -> PyResult<Vec<PyVal>> {
    seq.iter()?.map(|item| PyVal::from_pyany(item?)).collect()
}

/// Always returns a real key function -- never `None`. This matters:
/// the pure-Rust `natsorted()`/etc. treat `key = None` as "the item
/// itself is already one of the directly-downcastable types (String,
/// i64, f64, Vec<u8>, ...)", and box the raw item unchanged. `PyVal` is
/// none of those -- it's a wrapper enum -- so passing `None` through
/// would silently produce an identical (unsorted) key for every item
/// and the sort would appear to do nothing. Always converting through
/// `PyVal::into_any()` (optionally after running the user's Python
/// callable first) avoids that trap.
fn make_key_fn(py_key: Option<PyObject>) -> crate::KeyFn<PyVal> {
    match py_key {
        Some(callable) => Box::new(move |item: &PyVal| {
            Python::with_gil(|py| {
                let obj = item.to_pyobject(py);
                let result = callable
                    .call1(py, (obj,))
                    .expect("natsort key function raised an exception");
                let val = PyVal::from_pyany(result.as_ref(py))
                    .expect("natsort key function returned an unsupported type");
                val.into_any()
            })
        }),
        None => Box::new(|item: &PyVal| item.clone().into_any()),
    }
}

fn keypart_to_pyobject(py: Python<'_>, part: &KeyPart) -> PyObject {
    match part {
        KeyPart::Str(s) => s.into_py(py),
        KeyPart::Bytes(b) => PyBytes::new(py, b).into_py(py),
        KeyPart::Nested(parts) => {
            let elems: Vec<PyObject> = parts.iter().map(|p| keypart_to_pyobject(py, p)).collect();
            PyTuple::new(py, elems).into_py(py)
        }
        KeyPart::Num(n) => match n {
            Num::Int(i) => i.into_py(py),
            Num::Big(b) => b.to_string().into_py(py), // arbitrary precision: expose as decimal str
            Num::Float(f) => f.into_py(py),
        },
    }
}

fn results_to_pylist(py: Python<'_>, items: Vec<PyVal>) -> PyObject {
    let elems: Vec<PyObject> = items.iter().map(|v| v.to_pyobject(py)).collect();
    PyList::new(py, elems).into_py(py)
}

#[pyfunction]
#[pyo3(signature = (seq, key = None, reverse = false, alg = 0))]
fn natsorted(
    py: Python<'_>,
    seq: &PyAny,
    key: Option<PyObject>,
    reverse: bool,
    alg: u32,
) -> PyResult<PyObject> {
    let items = pyseq_to_pyvals(seq)?;
    let key_fn = Some(make_key_fn(key));
    let sorted = rs_natsorted(items, key_fn, reverse, alg);
    Ok(results_to_pylist(py, sorted))
}

#[pyfunction]
#[pyo3(signature = (seq, key = None, reverse = false, alg = 0))]
fn humansorted(
    py: Python<'_>,
    seq: &PyAny,
    key: Option<PyObject>,
    reverse: bool,
    alg: u32,
) -> PyResult<PyObject> {
    let items = pyseq_to_pyvals(seq)?;
    let key_fn = Some(make_key_fn(key));
    let sorted = rs_humansorted(items, key_fn, reverse, alg);
    Ok(results_to_pylist(py, sorted))
}

#[pyfunction]
#[pyo3(signature = (seq, key = None, reverse = false, alg = 0))]
fn realsorted(
    py: Python<'_>,
    seq: &PyAny,
    key: Option<PyObject>,
    reverse: bool,
    alg: u32,
) -> PyResult<PyObject> {
    let items = pyseq_to_pyvals(seq)?;
    let key_fn = Some(make_key_fn(key));
    let sorted = rs_realsorted(items, key_fn, reverse, alg);
    Ok(results_to_pylist(py, sorted))
}

#[pyfunction]
#[pyo3(signature = (seq, key = None, reverse = false, presort = false))]
fn os_sorted(
    py: Python<'_>,
    seq: &PyAny,
    key: Option<PyObject>,
    reverse: bool,
    presort: bool,
) -> PyResult<PyObject> {
    let items = pyseq_to_pyvals(seq)?;
    let key_fn = Some(make_key_fn(key));
    let sorted = rs_os_sorted(items, key_fn, reverse, presort);
    Ok(results_to_pylist(py, sorted))
}

#[pyfunction]
#[pyo3(signature = (seq, key = None, reverse = false, alg = 0))]
fn index_natsorted(
    py: Python<'_>,
    seq: &PyAny,
    key: Option<PyObject>,
    reverse: bool,
    alg: u32,
) -> PyResult<PyObject> {
    let items = pyseq_to_pyvals(seq)?;
    let key_fn = Some(make_key_fn(key));
    let idx = crate::index_natsorted(items, key_fn, reverse, alg);
    Ok(idx.into_py(py))
}

#[pyfunction]
fn order_by_index(py: Python<'_>, seq: &PyAny, index: Vec<usize>) -> PyResult<PyObject> {
    let items = pyseq_to_pyvals(seq)?;
    let ordered = crate::order_by_index(items, index);
    Ok(results_to_pylist(py, ordered))
}

/// Returns a reusable key function -- `sorted(seq, key=natsort_keygen(alg=...))`
/// works exactly like the pure-Python `natsort.natsort_keygen()`. Each
/// call computes the natural-sort key via the same Rust pipeline used by
/// `natsorted()`, then converts it into a plain nested Python tuple of
/// str/int/float/bytes so Python's own Timsort can compare keys directly.
#[pyfunction]
#[pyo3(signature = (key = None, alg = 0))]
fn natsort_keygen(py: Python<'_>, key: Option<PyObject>, alg: u32) -> PyResult<PyObject> {
    // Wrap the actual key computation in a small PyO3 class implementing
    // `__call__`, so the returned object can be passed straight into
    // Python's builtin `sorted(..., key=...)`.
    #[pyclass]
    struct NatsortKeyFn {
        alg: u32,
        has_key: bool,
        key: Option<PyObject>,
    }

    #[pymethods]
    impl NatsortKeyFn {
        fn __call__(&self, py: Python<'_>, val: &PyAny) -> PyResult<PyObject> {
            let keygen = crate::natsort_keygen(None, self.alg);
            let py_val = PyVal::from_pyany(val)?;
            let input_any: Box<dyn Any + Send + Sync> = if self.has_key {
                let callable = self.key.as_ref().unwrap();
                let result = callable.call1(py, (val,))?;
                PyVal::from_pyany(result.as_ref(py))?.into_any()
            } else {
                py_val.into_any()
            };
            let parts = keygen(input_any);
            let elems: Vec<PyObject> = parts.iter().map(|p| keypart_to_pyobject(py, p)).collect();
            Ok(PyTuple::new(py, elems).into_py(py))
        }
    }

    let has_key = key.is_some();
    let obj = NatsortKeyFn { alg, has_key, key };
    Ok(Py::new(py, obj)?.into_py(py))
}

#[pymodule]
fn r2dnsort(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(natsorted, m)?)?;
    m.add_function(wrap_pyfunction!(humansorted, m)?)?;
    m.add_function(wrap_pyfunction!(realsorted, m)?)?;
    m.add_function(wrap_pyfunction!(os_sorted, m)?)?;
    m.add_function(wrap_pyfunction!(index_natsorted, m)?)?;
    m.add_function(wrap_pyfunction!(order_by_index, m)?)?;
    m.add_function(wrap_pyfunction!(natsort_keygen, m)?)?;

    // `ns` flag constants, mirroring Python natsort's `ns` IntFlag values.
    let ns = PyModule::new(_py, "ns")?;
    ns.add("INT", Ns::INT.raw())?;
    ns.add("FLOAT", Ns::FLOAT.raw())?;
    ns.add("SIGNED", Ns::SIGNED.raw())?;
    ns.add("REAL", Ns::REAL.raw())?;
    ns.add("NOEXP", Ns::NOEXP.raw())?;
    ns.add("PATH", Ns::PATH.raw())?;
    ns.add("LOCALEALPHA", Ns::LOCALEALPHA.raw())?;
    ns.add("LOCALENUM", Ns::LOCALENUM.raw())?;
    ns.add("LOCALE", Ns::LOCALE.raw())?;
    ns.add("IGNORECASE", Ns::IGNORECASE.raw())?;
    ns.add("LOWERCASEFIRST", Ns::LOWERCASEFIRST.raw())?;
    ns.add("GROUPLETTERS", Ns::GROUPLETTERS.raw())?;
    ns.add("UNGROUPLETTERS", Ns::UNGROUPLETTERS.raw())?;
    ns.add("CAPITALFIRST", Ns::CAPITALFIRST.raw())?;
    ns.add("NANLAST", Ns::NANLAST.raw())?;
    ns.add("COMPATIBILITYNORMALIZE", Ns::COMPATIBILITYNORMALIZE.raw())?;
    ns.add("NUMAFTER", Ns::NUMAFTER.raw())?;
    ns.add("PRESORT", Ns::PRESORT.raw())?;
    ns.add("DEFAULT", Ns::DEFAULT.raw())?;
    m.add_submodule(ns)?;

    Ok(())
}
