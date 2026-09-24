# shellcheck shell=bash
# #480's e2e test: a stand-in Claude Code in the terminal, and a fake Voxtype first on Marley's
# PATH. The fake's status follows a file, and its `record toggle` moves that file on as Voxtype
# moves: idle, recording, transcribing, and idle again a few seconds later. The microphone shows
# each state, and `marley: toggle dictation` from the command palette toggles.

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin"
  # The terminal's shell becomes `claude` by name and prints a dot a second, so the terminal
  # keeps reading its foreground process.
  printf '%s\n' "exec -a claude bash -c 'while :; do printf .; sleep 1; done'" > "$home/.bashrc"
  terminal_env HOME "$home"
  cat > "$bin/voxtype" <<'FAKE'
#!/bin/sh
dir=$(dirname "$0")
state() { printf '{"text": "", "alt": "%s", "class": "%s", "tooltip": ""}\n' "$1" "$1" >> "$dir/status"; }
case "$1" in
status) exec tail -n +1 -f "$dir/status" ;;
record)
  if tail -n 1 "$dir/status" | grep -q '"class": "recording"'; then
    state transcribing
    (sleep 6; state idle) &
  else
    state recording
  fi ;;
esac
FAKE
  chmod +x "$bin/voxtype"
  printf '{"text": "", "alt": "idle", "class": "idle", "tooltip": ""}\n' > "$bin/status"
  PATH=$bin:$PATH
  git init -q -b voice "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

toggle_from_the_palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "toggle dictation"
  settle 1
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 4
  shot 480-01-idle
  toggle_from_the_palette
  settle 3
  shot 480-02-recording
  toggle_from_the_palette
  settle 2
  shot 480-03-transcribing
  settle 7
  shot 480-04-idle-again
}
