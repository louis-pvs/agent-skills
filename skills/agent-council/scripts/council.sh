#!/usr/bin/env bash
#
# Agent Council Shell Entrypoint (Delegates to council.py)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_SCRIPT="$SCRIPT_DIR/council.py"

exec python3 "$PYTHON_SCRIPT" "$@"
