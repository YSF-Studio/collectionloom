#!/usr/bin/env bash
# Generate binary test fixtures for forensic_test.rs
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIX="$ROOT/test-fixtures"
mkdir -p "$FIX"

# 10 MiB high-entropy image (for hashing + entropy tests)
if [[ ! -f "$FIX/evidence.dd" ]]; then
  echo "Generating 10 MiB evidence.dd…"
  dd if=/dev/urandom of="$FIX/evidence.dd" bs=1048576 count=10 status=none 2>/dev/null \
    || python3 -c "import os; open('$FIX/evidence.dd','wb').write(os.urandom(10*1024*1024))"
fi

# MBR-style stub (512 zero bytes + padding)
if [[ ! -f "$FIX/raw_evidence.bin" ]]; then
  python3 - <<'PY'
from pathlib import Path
p = Path(__file__).resolve().parent.parent / "test-fixtures" / "raw_evidence.bin"
buf = bytearray(4096)
# First 512 bytes zero (MBR)
p.write_bytes(buf)
PY
fi

echo "Fixtures ready in $FIX"
