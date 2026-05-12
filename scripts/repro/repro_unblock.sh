#!/usr/bin/env bash
#
# Tests the "press a key to unblock the paste" symptom from the bug
# report: paste stalls partway through, a keystroke unblocks it.
#
# We capture the pane (1) immediately after `paste-buffer` settles,
# (2) after sending a single Down key, (3) again after a few more
# seconds. If pressing a key really does unblock additional content,
# captures (2) and (3) will contain more lines than (1).

set -u
cd "$(dirname "$0")/../.."

FRESH_BIN="$PWD/target/release/fresh"
SAMPLE="$PWD/scripts/repro/sample.md"
SESSION="fresh_unblock_$$"

TMP_HOME="$(mktemp -d)"
export HOME="$TMP_HOME"
export XDG_CONFIG_HOME="$TMP_HOME/.config"
export XDG_DATA_HOME="$TMP_HOME/.local/share"
export XDG_CACHE_HOME="$TMP_HOME/.cache"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME"

cleanup() {
    tmux kill-session -t "$SESSION" 2>/dev/null || true
    rm -rf "$TMP_HOME"
}
trap cleanup EXIT

LOG="$TMP_HOME/fresh.log"
RUST_LOG="${RUST_LOG:-info}"
export RUST_LOG
tmux new-session -d -s "$SESSION" -x 120 -y 40 \
    "$FRESH_BIN --no-restore --no-plugins --log-file '$LOG'"
sleep 1.5

tmux load-buffer -b paste_repro "$SAMPLE"
tmux paste-buffer -b paste_repro -t "$SESSION"

# Wait for the stall to occur.
sleep 3

# --- Capture 1: post-paste, no extra input ---
out1="$(tmux capture-pane -t "$SESSION" -p)"
status1="$(echo "$out1" | tail -1)"
echo "===== Capture 1 (after paste-buffer, before any keypress) ====="
echo "$status1"

# Send a single Down key.
tmux send-keys -t "$SESSION" Down
sleep 1.5

# --- Capture 2: right after the keypress ---
out2="$(tmux capture-pane -t "$SESSION" -p)"
status2="$(echo "$out2" | tail -1)"
echo
echo "===== Capture 2 (1.5s after sending Down) ====="
echo "$status2"

# Give it more time in case unblocking is slow.
sleep 3

out3="$(tmux capture-pane -t "$SESSION" -p)"
status3="$(echo "$out3" | tail -1)"
echo
echo "===== Capture 3 (4.5s after sending Down) ====="
echo "$status3"

echo
echo "===== Summary ====="
extract_pos() { echo "$1" | grep -oE 'Ln [0-9]+, Col [0-9]+' | head -1; }
p1="$(extract_pos "$status1")"
p2="$(extract_pos "$status2")"
p3="$(extract_pos "$status3")"
echo "cursor after paste alone : ${p1:-?}"
echo "cursor 1.5s after Down   : ${p2:-?}"
echo "cursor 4.5s after Down   : ${p3:-?}"

echo
echo "===== event-drain log lines ====="
grep -F "event drain:" "$LOG" 2>/dev/null || echo "(no drain lines logged)"
# Move log somewhere persistent so the user can inspect it.
PERSIST_LOG="$PWD/scripts/repro/fresh.log"
cp -f "$LOG" "$PERSIST_LOG" 2>/dev/null || true
echo
echo "full log saved to $PERSIST_LOG"
