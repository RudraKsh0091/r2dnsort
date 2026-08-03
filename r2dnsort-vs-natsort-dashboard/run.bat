@echo off
REM One-click launcher: r2dnsort vs natsort dashboard (Windows)
cd /d "%~dp0"

if not exist ".venv" (
    echo Creating virtual environment...
    python -m venv .venv
)

call .venv\Scripts\activate.bat

echo Installing dependencies...
python -m pip install --quiet --upgrade pip
pip install --quiet -r backend\requirements.txt

echo Starting dashboard at http://localhost:8000
start "" http://localhost:8000

cd backend
python -m uvicorn main:app --host 0.0.0.0 --port 8000
