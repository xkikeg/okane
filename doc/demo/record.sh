#!/usr/bin/env bash
#
# Records the asciinema demos embedded in README.md / README.ja.md.
#
#   ./doc/demo/record.sh            # record every scene
#   ./doc/demo/record.sh ui         # record a single scene (ui | import)
#
# See doc/demo/README.md for the why behind the tmux indirection.

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEMO_DIR="${REPO}/doc/demo"
BIN="${REPO}/target/release/okane"

# The casts are recorded at this geometry, so this is what viewers see.
# 80 columns is too narrow: the balance footer hint gets truncated mid-word.
COLS=100
ROWS=30

# A private tmux server, configured from scratch, so neither the user's
# ~/.tmux.conf nor a running tmux session can influence the recording.
SOCKET="okane-demo"
SESSION="rec"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/okane-demo.XXXXXX")"
trap 'tmux -L "$SOCKET" kill-server 2>/dev/null || true; rm -rf "$WORK"' EXIT

tmux_() { tmux -L "$SOCKET" "$@"; }

# --- driving helpers --------------------------------------------------------

# Sends keys verbatim (tmux key names such as Enter, Escape, C-n, Space).
key() {
  local k
  for k in "$@"; do
    tmux_ send-keys -t "$SESSION" -- "$k"
    sleep 0.35
  done
}

# Types a string character by character, so the cast looks hand-typed.
type_str() {
  local s=$1 i
  for ((i = 0; i < ${#s}; i++)); do
    tmux_ send-keys -t "$SESSION" -l -- "${s:i:1}"
    sleep 0.04
  done
}

# Types a shell command and runs it.
type_line() {
  type_str "$1"
  sleep 0.5
  tmux_ send-keys -t "$SESSION" -- Enter
}

# Blocks until the visible pane matches a regex — far more reliable than
# guessing how long a build-dependent startup takes.
wait_for() {
  local pattern=$1 timeout=${2:-15} waited=0
  while ! tmux_ capture-pane -p -t "$SESSION" 2>/dev/null | grep -qE -- "$pattern"; do
    sleep 0.25
    waited=$((waited + 1))
    if ((waited > timeout * 4)); then
      echo "timed out waiting for /${pattern}/; pane was:" >&2
      tmux_ capture-pane -p -t "$SESSION" >&2 || true
      return 1
    fi
  done
}

# --- session lifecycle ------------------------------------------------------

# start_session <cast-path> <title>
start_session() {
  local cast=$1 title=$2

  cat >"${WORK}/tmux.conf" <<EOF
set -g status off
set -g default-terminal "xterm-256color"
set -g escape-time 0
EOF

  # --norc/--noprofile keeps the recording independent of the user's shell
  # setup; the rcfile only pins the prompt, PATH and history behaviour.
  cat >"${WORK}/bashrc" <<EOF
PS1='\[\033[32m\]\$\[\033[0m\] '
PATH="${REPO}/target/release:\$PATH"
unset HISTFILE
stty -echoctl
EOF

  rm -f "$cast"
  tmux_ -f "${WORK}/tmux.conf" new-session -d -s "$SESSION" -x "$COLS" -y "$ROWS" \
    -c "${WORK}/scene" \
    "asciinema rec --overwrite --idle-time-limit 1.5 --title '${title}' --env TERM \
       --command 'bash --noprofile --rcfile ${WORK}/bashrc -i' '${cast}'"

  # capture-pane strips trailing blanks, so the ready prompt is a bare "$".
  wait_for '^\$$' 10
  sleep 0.8
}

# Ends the recorded shell and waits for asciinema to flush the cast.
#
# The shell is hung up rather than sent an `exit` command: typing one would
# put a stray `exit` on the last frame of every recording. The pane process
# is asciinema itself, so the shell is its child.
end_session() {
  sleep 1.2
  local pane_pid
  pane_pid=$(tmux_ display-message -p -t "$SESSION" '#{pane_pid}')
  pkill -HUP -P "$pane_pid" || true
  local waited=0
  while tmux_ has-session -t "$SESSION" 2>/dev/null; do
    sleep 0.25
    waited=$((waited + 1))
    ((waited > 40)) && break
  done
  tmux_ kill-server 2>/dev/null || true
  sleep 0.3
}

# Keeps the last frame on screen for a moment after playback would otherwise
# stop: a cast ends at its final event, and the recorded shell is hung up the
# instant the scene is over. The appended events produce no output, they only
# extend the duration. Each gap is one `--idle-time-limit` step, since the
# player caps idle time at that.
hold_final_frame() {
  local cast=$1 steps=${2:-2}
  python3 - "$cast" "$steps" <<'PY'
import json, sys

path, steps = sys.argv[1], int(sys.argv[2])
with open(path) as f:
    lines = f.read().splitlines()
limit = json.loads(lines[0]).get("idle_time_limit", 1.5)
t = json.loads(lines[-1])[0]
with open(path, "a") as f:
    for _ in range(steps):
        t += limit * 0.95
        f.write(json.dumps([round(t, 6), "o", ""]) + "\n")
PY
}

# Fails loudly rather than leaving a truncated cast in the tree.
check_cast() {
  local cast=$1 min_events=$2
  [[ -s $cast ]] || {
    echo "error: ${cast} was not written" >&2
    return 1
  }
  local events
  events=$(($(wc -l <"$cast") - 1))
  if ((events < min_events)); then
    echo "error: ${cast} has only ${events} events (expected >= ${min_events})" >&2
    return 1
  fi
  echo "wrote ${cast} (${events} events, $(du -h "$cast" | cut -f1))"
}

fresh_scene() {
  rm -rf "${WORK}/scene"
  mkdir -p "${WORK}/scene"
}

# --- scene 1: okane ui ------------------------------------------------------

scene_ui() {
  local cast="${DEMO_DIR}/okane-ui.cast"
  fresh_scene
  # multi_commodity, not many_commodities: the balance draws one row per amount
  # line, and many_commodities' 26 stock lots bury the account column under a
  # screenful of amounts. This one has enough commodities to show the same
  # thing while staying readable.
  cp "${REPO}/testdata/report/multi_commodity.ledger" "${WORK}/scene/"

  start_session "$cast" "okane ui — interactive balance and register"

  type_line "okane ui multi_commodity.ledger"
  wait_for 'okane ui —' 20
  sleep 1.5

  # The footer points at `?`, so open the key help before using the keys.
  key '?'
  wait_for 'Key bindings' 10
  sleep 3.5
  key Escape
  sleep 1.2

  # Flat balance: `J` steps by account, over the amount lines a multi-commodity
  # balance spreads each one across.
  key J J J
  sleep 1.2

  # Tree mode, then fold the selected subtree and every subtree.
  key t
  sleep 1.5
  key j j Space
  sleep 1.5
  key x
  sleep 1.5
  key x
  sleep 1.2

  # Incremental search, then jump between matches.
  key /
  type_str "Brokers"
  sleep 1
  key Enter
  sleep 1
  key n
  sleep 1
  key Escape
  sleep 1

  # Drill into the register of the selected account. `J` rather than `j`: the
  # `(total)` row at the top spans one line per commodity, so `j` would still
  # be inside it.
  key g J J
  sleep 0.8
  key Enter
  wait_for 'register:' 10
  sleep 1.5
  # The register opens on the newest entry, so walk up from there rather than
  # pressing `j` into a cursor that is already at the bottom.
  key g
  sleep 1.2
  key j j j
  sleep 1.2
  key J J
  sleep 1.2
  key G
  sleep 1.5
  key q
  sleep 1.2

  # Quit through the confirmation overlay.
  key q
  sleep 1
  key y
  sleep 1

  end_session
  hold_final_frame "$cast"
  check_cast "$cast" 40
}

# --- scene 2: okane import --interactive ------------------------------------

scene_import() {
  local cast="${DEMO_DIR}/okane-import.cast"
  fresh_scene
  # Keep the original file names: import config documents are selected by
  # matching their `path:` against the source path (cli/src/import/config.rs).
  cp "${REPO}/testdata/import/index_amount.csv" "${WORK}/scene/"
  cp "${REPO}/testdata/import/test_config.yml" "${WORK}/scene/"
  cp "${REPO}/testdata/report/multi_commodity.ledger" "${WORK}/scene/accounts.ledger"
  : >"${WORK}/scene/out.ledger"

  start_session "$cast" "okane import --interactive — review before writing"

  type_line "cat index_amount.csv"
  sleep 2.5

  type_line "okane import --config test_config.yml --interactive \\"
  type_line "    --ledger accounts.ledger -o out.ledger index_amount.csv"
  wait_for 'okane import —' 20
  sleep 2

  # Walk down to an Expenses:Unknown row, reading the previews.
  key j
  sleep 1.2
  key j
  sleep 1.5

  # Give it an account through the completing prompt. Accepting a decision
  # advances the cursor, so every beat below starts on the next row.
  key e
  sleep 1.2
  type_str "expenses:"
  sleep 1.5
  key C-n C-n
  sleep 1.2
  key Tab
  sleep 1.2
  key Enter
  sleep 1.5

  # Leave the last one alone for now.
  key s
  sleep 1.5

  # Writing with an undecided row left over asks first; back out of it.
  key w
  sleep 1.8
  key n
  sleep 1.2

  # Accept the pending transaction at the top, then write for real.
  key g
  sleep 1.2
  key a
  sleep 1.5
  key w
  wait_for 'appended' 10
  sleep 1.5

  type_line "cat out.ledger"
  sleep 3

  end_session
  hold_final_frame "$cast"
  check_cast "$cast" 40
}

# --- entry point ------------------------------------------------------------

[[ -x $BIN ]] || {
  echo "building ${BIN} ..." >&2
  (cd "$REPO" && cargo build --release)
}

case "${1:-all}" in
ui) scene_ui ;;
import) scene_import ;;
all)
  scene_ui
  scene_import
  ;;
*)
  echo "usage: $0 [ui|import|all]" >&2
  exit 2
  ;;
esac
