#!/usr/bin/env bash
# Constellation Lexicon — one-command regenerate.
#
# Usage: ./regenerate.sh
#
# Steps:
#   1. python build.py     — concepts.json → src-tauri/src/lexicon/data/lexicon_v1.tsv
#   2. python validate.py  — sanity checks against the emitted TSV
#
# Exit non-zero if either step fails so CI / the user notices immediately.

set -euo pipefail

cd "$(dirname "$0")"

PY=${PYTHON:-python3}
if ! command -v "$PY" >/dev/null 2>&1; then
    PY=python
fi

echo "→ building lexicon_v1.tsv from concepts.json"
"$PY" build.py

echo
echo "→ validating emitted TSV"
"$PY" validate.py
