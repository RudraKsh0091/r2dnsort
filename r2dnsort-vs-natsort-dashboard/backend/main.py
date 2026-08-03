"""
r2dnsort vs natsort — Live Comparison Dashboard
Backend: FastAPI

Runs Python's `natsort` and the Rust-powered `r2dnsort` (PyO3 extension,
published on PyPI as `r2dnsort`) side by side on the same input and
returns both results plus timing, so a judge can see structural and
performance parity in real time.
"""
from __future__ import annotations

import time
import traceback
from pathlib import Path
from typing import Any, List, Optional

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, Field

# --------------------------------------------------------------------------
# Optional-import guards: the dashboard must still boot (and tell the judge
# what's missing) even if one of the two libraries isn't installed yet.
# --------------------------------------------------------------------------
NATSORT_AVAILABLE = False
R2DNSORT_AVAILABLE = False
NATSORT_VERSION = None
R2DNSORT_VERSION = None
NATSORT_IMPORT_ERROR = None
R2DNSORT_IMPORT_ERROR = None

try:
    import natsort
    from natsort import natsorted, humansorted, realsorted, os_sorted, ns as natsort_ns

    NATSORT_AVAILABLE = True
    NATSORT_VERSION = getattr(natsort, "__version__", "unknown")
except Exception as e:  # pragma: no cover
    NATSORT_IMPORT_ERROR = str(e)

try:
    import r2dnsort
    from r2dnsort import natsorted as rs_natsorted
    from r2dnsort import humansorted as rs_humansorted
    from r2dnsort import realsorted as rs_realsorted
    from r2dnsort import os_sorted as rs_os_sorted
    from r2dnsort import ns as r2dnsort_ns

    R2DNSORT_AVAILABLE = True
    R2DNSORT_VERSION = getattr(r2dnsort, "__version__", "0.1.0")
except Exception as e:  # pragma: no cover
    R2DNSORT_IMPORT_ERROR = str(e)


# --------------------------------------------------------------------------
# ns flag name -> bit value (mirrors natsort.ns / r2dnsort.ns, both expose
# the same IntFlag-compatible surface, so a single map drives both sides).
# --------------------------------------------------------------------------
NS_FLAGS = {
    "FLOAT": 0x0001,
    "SIGNED": 0x0002,
    "NOEXP": 0x0004,
    "PATH": 0x0008,
    "LOCALEALPHA": 0x0010,
    "LOCALENUM": 0x0020,
    "IGNORECASE": 0x0040,
    "LOWERCASEFIRST": 0x0080,
    "GROUPLETTERS": 0x0100,
    "UNGROUPLETTERS": 0x0200,
    "NANLAST": 0x0400,
    "COMPATIBILITYNORMALIZE": 0x0800,
    "NUMAFTER": 0x1000,
    "PRESORT": 0x2000,
}

ALGORITHMS = {"natsorted", "humansorted", "realsorted", "os_sorted"}

SAMPLE_DATASETS = {
    "versions": [
        "Version 1.9", "Version 1.10", "Version 1.2", "Version 2.0",
        "Version 1.1", "Version 10.0", "Version 1.0-alpha",
    ],
    "filenames": [
        "img12.png", "img10.png", "img2.png", "img1.png", "img55.png",
        "img9.png", "IMG3.png", "img_final.png", "img100.png",
    ],
    "mixed_units": [
        "10 apples", "2 bananas", "1 kiwi", "20 grapes", "3 oranges",
        "100 mangoes", "12 pears",
    ],
    "scientific": [
        "1.5e3", "2.1e-2", "3.14e0", "-1.2e5", "6.02e23", "1e-10", "NaN",
    ],
    "paths": [
        "/data/run1/file10.txt", "/data/run10/file2.txt",
        "/data/run2/file1.txt", "/data/run1/file2.txt",
        "/data/run1/file1.txt",
    ],
    "signed_numbers": [
        "-5", "3", "-100", "0", "42", "-1", "7",
    ],
    "unicode_numerals": [
        "item①", "item③", "item②", "item①①", "item⑤",
    ],
    "large_integers": [
        "99999999999999999999999999",
        "1",
        "99999999999999999999999998",
        "2.5e30",
        "100",
    ],
}


class SortRequest(BaseModel):
    items: List[str] = Field(..., description="Raw string items to sort")
    algorithm: str = Field("natsorted", description="natsorted|humansorted|realsorted|os_sorted")
    reverse: bool = False
    flags: List[str] = Field(default_factory=list, description="ns flag names, e.g. ['IGNORECASE']")
    libraries: List[str] = Field(default_factory=lambda: ["natsort", "r2dnsort"])
    runs: int = Field(1, ge=1, le=200, description="Repeat count for benchmark timing")


class LibraryResult(BaseModel):
    library: str
    available: bool
    ok: bool
    result: Optional[List[Any]] = None
    error: Optional[str] = None
    elapsed_ms: Optional[float] = None
    elapsed_ms_avg: Optional[float] = None


class SortResponse(BaseModel):
    input_count: int
    algorithm: str
    reverse: bool
    flags: List[str]
    alg_value: int
    match: Optional[bool] = None
    results: List[LibraryResult]


def flags_to_int(flag_names: List[str]) -> int:
    value = 0
    for name in flag_names:
        value |= NS_FLAGS.get(name.upper(), 0)
    return value


def run_library(lib_name: str, algorithm: str, items: List[str], alg_value: int,
                 reverse: bool, runs: int) -> LibraryResult:
    available = NATSORT_AVAILABLE if lib_name == "natsort" else R2DNSORT_AVAILABLE
    if not available:
        err = NATSORT_IMPORT_ERROR if lib_name == "natsort" else R2DNSORT_IMPORT_ERROR
        return LibraryResult(library=lib_name, available=False, ok=False,
                              error=err or f"{lib_name} is not installed")

    if lib_name == "natsort":
        funcs = {
            "natsorted": natsorted, "humansorted": humansorted,
            "realsorted": realsorted, "os_sorted": os_sorted,
        }
    else:
        funcs = {
            "natsorted": rs_natsorted, "humansorted": rs_humansorted,
            "realsorted": rs_realsorted, "os_sorted": rs_os_sorted,
        }

    fn = funcs.get(algorithm)
    if fn is None:
        return LibraryResult(library=lib_name, available=True, ok=False,
                              error=f"unknown algorithm '{algorithm}'")

    try:
        if algorithm == "os_sorted":
            call = lambda: fn(items, reverse=reverse)
        else:
            call = lambda: fn(items, reverse=reverse, alg=alg_value)

        # warm-up call (excluded from timing) + timed run(s)
        result = call()
        durations = []
        for _ in range(runs):
            t0 = time.perf_counter()
            result = call()
            durations.append((time.perf_counter() - t0) * 1000.0)

        return LibraryResult(
            library=lib_name, available=True, ok=True,
            result=list(result), elapsed_ms=durations[0],
            elapsed_ms_avg=sum(durations) / len(durations),
        )
    except Exception as e:
        return LibraryResult(library=lib_name, available=True, ok=False,
                              error=f"{type(e).__name__}: {e}\n{traceback.format_exc(limit=2)}")


app = FastAPI(title="r2dnsort vs natsort — Live Dashboard")
app.add_middleware(
    CORSMiddleware, allow_origins=["*"], allow_methods=["*"], allow_headers=["*"],
)

FRONTEND_DIR = Path(__file__).resolve().parent.parent / "frontend"


@app.get("/api/status")
def status():
    return {
        "natsort": {"available": NATSORT_AVAILABLE, "version": NATSORT_VERSION,
                     "error": None if NATSORT_AVAILABLE else NATSORT_IMPORT_ERROR},
        "r2dnsort": {"available": R2DNSORT_AVAILABLE, "version": R2DNSORT_VERSION,
                      "error": None if R2DNSORT_AVAILABLE else R2DNSORT_IMPORT_ERROR},
        "algorithms": sorted(ALGORITHMS),
        "flags": list(NS_FLAGS.keys()),
    }


@app.get("/api/samples")
def samples():
    return SAMPLE_DATASETS


@app.post("/api/sort", response_model=SortResponse)
def sort_endpoint(req: SortRequest):
    alg_value = flags_to_int(req.flags)
    libs = [l for l in req.libraries if l in ("natsort", "r2dnsort")] or ["natsort", "r2dnsort"]

    results = [
        run_library(lib, req.algorithm, req.items, alg_value, req.reverse, req.runs)
        for lib in libs
    ]

    match = None
    ok_results = [r for r in results if r.ok]
    if len(ok_results) == 2:
        match = ok_results[0].result == ok_results[1].result

    return SortResponse(
        input_count=len(req.items), algorithm=req.algorithm, reverse=req.reverse,
        flags=req.flags, alg_value=alg_value, match=match, results=results,
    )


# --------------------------------------------------------------------------
# Static frontend (served from the same origin -> true one-click demo:
# `python backend/main.py` and open http://localhost:8000)
# --------------------------------------------------------------------------
app.mount("/static", StaticFiles(directory=str(FRONTEND_DIR)), name="static")


@app.get("/")
def index():
    return FileResponse(str(FRONTEND_DIR / "index.html"))


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=False)
