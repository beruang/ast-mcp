#!/bin/bash
# Smoke test for ast-mcp stdio transport.
# Pipes 4 JSON-RPC requests into the binary and checks responses.

set -e

WORKTREE="/Volumes/Workspace/rnd/workflow/mcp/.agent-worktrees/ast-mcp-v1/task-1"
MANIFEST="$WORKTREE/Cargo.toml"
BIN="$WORKTREE/target/release/ast-mcp"
PASS=0
FAIL=0

function send() {
    local method="$1"
    local id="$2"
    local params="${3:-{}}"
    echo "Sending: $method (id=$id)"
    # Use printf instead of echo to avoid trailing newline issues
    RESP=$(printf '{"jsonrpc":"2.0","id":%s,"method":"%s","params":%s}\n' "$id" "$method" "$params" | $BIN 2>/dev/null)
    echo "Response: $RESP"
    if [ -n "$RESP" ]; then
        echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); print('OK')" 2>/dev/null
        ((PASS++)) || true
    else
        echo "FAIL: no response"
        ((FAIL++)) || true
    fi
}

# Build release first (already built, but run anyway for idempotency)
echo "Building release binary..."
cargo build --manifest-path "$MANIFEST" --release

echo ""
echo "=== Smoke Test ==="

# 1. initialize
send "initialize" 1 "{}"

# 2. notifications/initialized (no response expected, but no error)
echo "Sending: notifications/initialized (no response expected)"
# Notifications don't have an id field
RESP=$(printf '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}\n' | $BIN 2>/dev/null || true)
echo "Response: $RESP (should be empty)"

# 3. tools/list
send "tools/list" 2 "{}"

# 4. tools/call ast_health_check
RESP=$(printf '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ast_health_check","arguments":{}}}\n' | $BIN 2>/dev/null)
echo "Response: $RESP"
# serde_json outputs {"ok":true} (no space after colon), but the JSON is semantically correct
echo "$RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); text=d['result']['content'][0]['text']; parsed=json.loads(text); assert parsed.get('ok') == True; print('OK')" 2>/dev/null
((PASS++)) || true

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ] || exit 1