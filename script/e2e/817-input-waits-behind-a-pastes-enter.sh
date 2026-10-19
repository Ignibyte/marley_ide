# shellcheck shell=bash
# #817's visual check: keys typed right after a prompt sent from an agent's editor reach the agent
# after its Enter. A stand-in `claude`, first on the terminal's PATH, prints and logs each line it
# reads (`got: <line>`). Ctrl-G opens its editor; one burst of keys, 20 ms apart, types `first`,
# Return (the editor sends it, its Enter 200 ms later), `second`, Return (to the terminal).
# - The stand-in read `first` and `second`, each a line of its own (`817-01-agent`, REQ-001,
#   REQ-002).
compositor sway

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  sed -e "s|@LOG@|$E2E_WORK/claude.log|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in agent: each line it reads, printed and logged.
import sys

print("claude (stand-in): type a line", flush=True)
for line in sys.stdin:
    line = line.rstrip("\n")
    with open("@LOG@", "a", encoding="utf-8") as log:
        log.write(f"got: {line}\n")
    print(f"got: {line}", flush=True)
FAKE
  chmod +x "$bin/claude"
  : >"$E2E_WORK/claude.log"
  git init -q -b agent "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

# Whether the stand-in's last two lines are `got: first` and `got: second`.
read_in_order() {
  [[ $(tail -2 "$E2E_WORK/claude.log") == $'got: first\ngot: second' ]]
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  type_text "claude"
  press "" Return
  settle 3

  echo "== the agent's editor, then the next line at once"
  press CTRL g
  settle 2
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "first" -k Return "second" -k Return
  settle 3
  shot 817-01-agent
  cat "$E2E_WORK/claude.log"
  expect "the stand-in read first and second, each a line of its own" read_in_order
}
