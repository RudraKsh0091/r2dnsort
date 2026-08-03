# r2dnsort vs natsort — Live Comparison Dashboard

A one-click, judge-facing demo that runs **Python's `natsort`** and our
**Rust-powered `r2dnsort`** (PyO3 extension, [on PyPI](https://pypi.org/project/r2dnsort/),
[source on GitHub](https://github.com/RudraKsh0091/r2dnsort)) side by side on the
same input, so you can see correctness parity and Rust's speed advantage in
real time — no code required.

Stack: **FastAPI** backend (Python) + **vanilla JS** frontend, served from one
process so the whole thing is a single `http://localhost:8000` page.

---

## ▶ One-click run

**Requirements:** Python 3.8+ on your machine, with internet access (both
libraries install from PyPI on first run).

### macOS / Linux
```bash
./run.sh
```

### Windows
```
run.bat
```

That's it. The script:
1. Creates a local virtual environment (`.venv`)
2. Installs `fastapi`, `uvicorn`, `natsort`, and `r2dnsort` (our published PyPI package)
3. Starts the server and opens **http://localhost:8000** in your browser automatically

If the browser doesn't open by itself, just visit **http://localhost:8000** manually.

---

## What you can do in the dashboard

- **Pick a sample dataset** (version strings, filenames, mixed-unit text,
  scientific notation, filesystem paths, signed numbers, unicode numerals,
  astronomically large integers) or paste your own list, one item per line.
- **Choose an algorithm**: `natsorted`, `realsorted`, `humansorted`, `os_sorted`
  — mirrors natsort's real API surface, because r2dnsort implements the same one.
- **Toggle `ns` flags** (`IGNORECASE`, `SIGNED`, `FLOAT`, `LOCALEALPHA`, `NANLAST`, …)
  — identical bit-flags on both sides.
- **Run both libraries simultaneously** and see:
  - The sorted output from each, with any *mismatched* lines highlighted
  - A green "✓ identical output" banner when both agree
  - Per-run and averaged timing (configurable run count) with a live bar chart
  - A computed speed multiplier between the two

This is meant to be genuinely interactive — judges can type their own edge
cases (empty strings, `NaN`, huge integers, mixed unicode digits, negative
numbers) and watch both implementations respond live.

---

## Project layout

```
dashboard/
├── run.sh / run.bat        one-click launchers
├── backend/
│   ├── main.py              FastAPI app: /api/status, /api/samples, /api/sort
│   └── requirements.txt
└── frontend/
    ├── index.html
    ├── style.css
    └── app.js                served by FastAPI as static files
```

## Manual run (if you prefer not to use the script)

```bash
cd backend
python -m venv .venv && source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install -r requirements.txt
python -m uvicorn main:app --reload --port 8000
```

Then open http://localhost:8000.

## Notes

- If `r2dnsort` fails to install (e.g. no prebuilt wheel for your OS/Python
  version yet), the dashboard still boots — the status pill will show
  "r2dnsort not installed" and that column will explain the `pip install`
  command needed, while natsort continues to work standalone.
- The backend never shells out to `cargo`/`maturin` — it imports the
  already-compiled `r2dnsort` PyO3 extension exactly the way any Python
  caller would (`pip install r2dnsort`), so what you see here is the real
  published package, not a rebuilt copy.
