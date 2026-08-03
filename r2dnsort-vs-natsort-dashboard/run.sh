#!/usr/bin/env bash
# One-click launcher: r2dnsort vs natsort dashboard
# Usage:  ./run.sh
set -e

cd "$(dirname "$0")"

if [ ! -d ".venv" ]; then
  echo "→ creating virtual environment (.venv)..."
  python3 -m venv .venv
fi

source .venv/bin/activate

echo "→ installing dependencies (fastapi, uvicorn, natsort, r2dnsort)..."
pip install --quiet --upgrade pip
pip install --quiet -r backend/requirements.txt

echo "→ starting dashboard at http://localhost:8000"

# Open the browser automatically once the server is up
( sleep 1.5
  if command -v open >/dev/null 2>&1; then open http://localhost:8000
  elif command -v xdg-open >/dev/null 2>&1; then xdg-open http://localhost:8000
  fi
) &

cd backend
python -m uvicorn main:app --host 0.0.0.0 --port 8000
