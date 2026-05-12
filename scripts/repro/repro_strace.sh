#!/usr/bin/env bash
# Trace tmux's writes to the pty during a paste-buffer into fresh.
set -u
cd "$(dirname "$0")/../.."

FRESH_BIN="$PWD/target/release/fresh"
SAMPLE="$PWD/scripts/repro/sample.md"
SESSION="fresh_strace_$$"
TMP_HOME="$(mktemp -d)"
export HOME="$TMP_HOME"
export XDG_CONFIG_HOME="$TMP_HOME/.config" XDG_DATA_HOME="$TMP_HOME/.local/share" XDG_CACHE_HOME="$TMP_HOME/.cache"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME"

cleanup() {
    tmux kill-session -t "$SESSION" 2>/dev/null || true
    [ -n "${TMUX_PID:-}" ] && kill -CONT "$TMUX_PID" 2>/dev/null || true
    rm -rf "$TMP_HOME"
}
trap cleanup EXIT

tmux new-session -d -s "$SESSION" -x 120 -y 40 "$FRESH_BIN --no-restore --no-plugins"
sleep 1.5

# Find the tmux server pid (which owns the master pty fds) and the fresh pid
TMUX_PID=$(pgrep -f "tmux .* new-session -d -s $SESSION" 2>/dev/null || pgrep -nf "^tmux .*server" || pgrep -n tmux)
FRESH_PID=$(pgrep -nf "target/release/fresh")
echo "tmux server pid: $TMUX_PID"
echo "fresh pid:       $FRESH_PID"

# Strace tmux's writes only, in background; let it stop when we kill it
strace -p "$TMUX_PID" -e trace=write,writev,read -o /tmp/tmux_strace.log -f -s 32 2>/dev/null &
STRACE_PID=$!
sleep 0.3

tmux load-buffer -b paste_repro "$SAMPLE"
tmux paste-buffer -b paste_repro -t "$SESSION"

sleep 3
echo "==== status after 3s paste settle ===="
tmux capture-pane -t "$SESSION" -p | tail -1

tmux send-keys -t "$SESSION" Down
sleep 1.5
echo "==== status 1.5s after Down ===="
tmux capture-pane -t "$SESSION" -p | tail -1

kill "$STRACE_PID" 2>/dev/null
wait "$STRACE_PID" 2>/dev/null

# Filter strace output: writes >= 100 bytes (i.e. paste chunks, not single keys/control codes)
echo
echo "==== tmux writes >= 100 bytes (filtered, last 40) ===="
grep -oE 'write\([0-9]+, ".*", [0-9]+\)[[:space:]]*=[[:space:]]*[0-9]+' /tmp/tmux_strace.log \
    | awk -F'[=,]' '{ n=$NF+0; if (n >= 100) print n " bytes" }' \
    | tail -40
echo
echo "==== summary: bytes written per syscall to fresh's pty master ===="
grep -oE 'write\([0-9]+, .*\) *= *[0-9]+' /tmp/tmux_strace.log \
    | awk '{ for (i=NF; i>0; i--) if ($i+0 > 0) { print $i; break } }' \
    | sort -n | uniq -c | sort -rn | head -10
