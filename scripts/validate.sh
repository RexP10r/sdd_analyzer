#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass()  { echo -e "${GREEN}PASS${NC} $*"; }
fail()  { echo -e "${RED}FAIL${NC} $*"; exit 1; }

echo "=== SDD Navigator Self-Validation ==="
echo

# 1. Formatting
echo "--- cargo fmt --check ---"
if cargo fmt --check; then
    pass "cargo fmt"
else
    fail "cargo fmt"
fi

# 2. Clippy
echo
echo "--- cargo clippy ---"
if cargo clippy -- -D warnings; then
    pass "cargo clippy"
else
    fail "cargo clippy"
fi

# 3. Tests
echo
echo "--- cargo test ---"
if cargo test; then
    pass "cargo test"
else
    fail "cargo test"
fi

# 4. Build release binary
echo
echo "--- cargo build --release ---"
if cargo build --release; then
    pass "cargo build --release"
else
    fail "cargo build --release"
fi

# 5. Strict self-hosting scan
echo
echo "--- sdd-coverage scan --strict ---"
if ./target/release/sdd-coverage scan --requirements requirements.yaml --source . --strict; then
    pass "self-hosting strict scan (zero violations)"
else
    fail "self-hosting strict scan"
fi

# 6. Traceability coverage check
echo
echo "--- @req annotation coverage ---"
REQ_COUNT=$(grep -c '^  - id:' requirements.yaml)
ANNOTATED_REQS=$(grep -roh '@req\s\+[A-Z]\+-[A-Z]\+-[0-9]\+' . --include="*.rs" 2>/dev/null | sed 's/@req *//' | sort -u)

MISSING=0
while IFS= read -r req_id; do
    if ! echo "$ANNOTATED_REQS" | grep -qF "$req_id"; then
        echo "  MISSING @req for: $req_id"
        MISSING=$((MISSING + 1))
    fi
done < <(grep '^  - id:' requirements.yaml | sed 's/.*id: //')

if [ "$MISSING" -eq 0 ]; then
    pass "all $REQ_COUNT requirements have @req annotations"
else
    fail "$MISSING requirement(s) lack @req annotations"
fi

echo "=== ALL CHECKS PASSED ==="
