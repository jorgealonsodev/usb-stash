#!/usr/bin/env bash
set -euo pipefail

# ── USB Stash Launcher ──────────────────────────────────────────────────────
# Runs the USB Stash binary from a USB drive, even on filesystems mounted
# with noexec (FAT32, exFAT). Copies the binary to a temp directory when
# needed, runs it, and cleans up on exit.
# ────────────────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Step 1: Find the binary ─────────────────────────────────────────────────
# Priority: GUI binary > CLI binary

BINARY=""
for candidate in "usb-stash-gui-linux" "usb-stash-linux"; do
    if [[ -f "$SCRIPT_DIR/$candidate" ]]; then
        BINARY="$SCRIPT_DIR/$candidate"
        break
    fi
done

if [[ -z "$BINARY" ]]; then
    echo "ERROR: No USB Stash binary found in $SCRIPT_DIR" >&2
    echo "Expected: usb-stash-linux or usb-stash-gui-linux" >&2
    exit 1
fi

# ── Step 2: Check if we can execute directly ─────────────────────────────────
# FAT32 / exFAT are typically mounted noexec on Linux. If exec is available,
# run directly (faster, no copy to temp).

can_exec() {
    local dir="$1"
    # Try creating and running a tiny test script
    local testfile="$dir/.usbstash_exec_test_$$"
    if printf '#!/bin/sh\nexit 0\n' > "$testfile" 2>/dev/null; then
        chmod +x "$testfile" 2>/dev/null || true
        if "$testfile" 2>/dev/null; then
            rm -f "$testfile"
            return 0
        fi
        rm -f "$testfile"
    fi
    return 1
}

if can_exec "$SCRIPT_DIR"; then
    exec "$BINARY" "$@"
    # Unreachable if exec succeeds
fi

# ── Step 3: noexec — copy to temp and run ────────────────────────────────────

TEMP_DIR="${TMPDIR:-/tmp}/usb-stash-${USER:-user}-$$"
mkdir -p "$TEMP_DIR"
chmod 700 "$TEMP_DIR"

cleanup() {
    # Kill child process if we're interrupted
    if [[ -n "${CHILD_PID:-}" ]]; then
        kill "$CHILD_PID" 2>/dev/null || true
        wait "$CHILD_PID" 2>/dev/null || true
    fi
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT INT TERM

TEMP_BIN="$TEMP_DIR/$(basename "$BINARY")"
cp "$BINARY" "$TEMP_BIN"
chmod +x "$TEMP_BIN"

# Run in background so trap can handle signals
"$TEMP_BIN" "$@" &
CHILD_PID=$!
wait "$CHILD_PID"
