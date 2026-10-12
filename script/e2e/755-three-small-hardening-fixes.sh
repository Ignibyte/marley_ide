# shellcheck shell=bash
# #755's visual check: three hardening fixes. Marley runs with `CODEX_HOME` and `XDG_CONFIG_HOME`
# in the run's own folders, Codex's `config.toml` a symlink into a dotfiles folder at mode 0600,
# and stand-in `codex` and `opencode` in the terminal (#552's set-up).
# - A: Turn on Codex notifications writes the link's target, keeps the link, the mode and a
#   backup, and leaves no temp file (`codex-written`, REQ-001 to REQ-003); Connect OpenCode to
#   Marley writes the plugin with no backup and no temp file (`plugin-written`, REQ-003).
# - B: `tools-ask`, on the terminal's PATH, calls `terminal_list` through the plugin's bridge: from
#   a terminal with its own id and token its row is `self` (`honest`, REQ-005, REQ-006); with
#   another terminal's id and its own token no row is `self` (`spoofed`, REQ-007).
# - C: a project's Browser tab on a calm page; its unit's `OOMPolicy` is `continue`
#   (`calm`, with the unit's properties in the run's log, REQ-008); with the unit's memory capped,
#   a page that eats memory in a second tab (`hungry`) has its renderer killed: its tab closes
#   while the calm tab still draws and the unit stays `active` (`after-oom`, REQ-009).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-}
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# The agent bar's chip, as #552's shots found it.
CHIP_X=${CHIP_X:-460}
CHIP_Y=${CHIP_Y:-954}

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

# Types `$*` into the terminal in front and waits for it.
run() {
  type_text "clear; $*"
  press "" Return
  settle "${RUN_WAIT:-3}"
}

write_client() {
  cat >"$E2E_WORK/tools-client.py" <<'PY'
# One call of Marley's terminal_list through the plugin's bridge, from the terminal it runs in:
# each terminal's id, and which row is the caller's own.
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL, text=True)
next_id = 0


def call(method, params):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method,
                                   "params": params}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        message = json.loads(line)
        if message.get("id") == next_id:
            return message
    sys.exit(f"{method}: the bridge closed")


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                    "clientInfo": {"name": "e2e-agent", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
result = call("tools/call", {"name": "terminal_list", "arguments": {}}).get("result") or {}
terminals = (result.get("structuredContent") or {}).get("terminals", [])
lines = [f"{'self ' if terminal.get('self') else '     '}{terminal.get('id')} {terminal.get('title')}"
         for terminal in terminals]
out = "\n".join(lines) + f"\nself rows: {sum(1 for t in terminals if t.get('self'))}"
print(out)
with open(os.environ["ANSWERS"], "a", encoding="utf-8") as saved:
    saved.write(out + "\n")
bridge.stdin.close()
bridge.wait(timeout=10)
PY
  cat >"$E2E_WORK/bin/tools-ask" <<SH
#!/bin/sh
BRIDGE='$BRIDGE' MARLEY_MCP_ENDPOINT='$E2E_PROFILE/mcp-endpoint.json' ANSWERS='$E2E_WORK/answers.txt' \\
  exec python3 '$E2E_WORK/tools-client.py' "\$@"
SH
  chmod +x "$E2E_WORK/bin/tools-ask"
}

write_pages() {
  mkdir -p "$E2E_WORK/calm" "$E2E_WORK/hog"
  cat >"$E2E_WORK/calm/index.html" <<'HTML'
<!doctype html><title>calm page</title>
<body style="font: 32px sans-serif; background: #2a6"><p>The calm page still draws.</p></body>
HTML
  cat >"$E2E_WORK/hog/index.html" <<'HTML'
<!doctype html><title>hungry page</title>
<body style="font: 32px sans-serif; background: #a62"><p id="n">A page that eats memory.</p>
<script>
const kept = [];
// Stops at 2 GiB, so a cap that does not hold cannot eat the machine.
const eating = setTimeout(() => setInterval(() => {
  if (kept.length >= 64) return;
  const block = new Uint8Array(32 * 1024 * 1024);
  block.fill(kept.length % 255);
  kept.push(block);
  document.getElementById("n").textContent = `${kept.length * 32} MiB eaten`;
}, 100), 3000);
</script></body>
HTML
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo dotfiles=$E2E_WORK/dotfiles
  mkdir -p "$home" "$bin" "$repo" "$E2E_WORK/codex" "$E2E_WORK/xdg" "$dotfiles"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  export CODEX_HOME=$E2E_WORK/codex XDG_CONFIG_HOME=$E2E_WORK/xdg
  cat >"$dotfiles/codex.toml" <<'TOML'
# The run's Codex config, kept in a dotfiles folder: a comment, a model, and a [tui] key.
model = "gpt-5"

[tui]
theme = "dark"
TOML
  chmod 600 "$dotfiles/codex.toml"
  ln -s "$dotfiles/codex.toml" "$CODEX_HOME/config.toml"
  local agent
  for agent in codex opencode; do
    printf '#!/usr/bin/env bash\nexec -a %s python3 -u -c %q\n' "$agent" \
      "import sys; print('stand-in $agent'); sys.stdin.read()" >"$bin/$agent"
    chmod +x "$bin/$agent"
  done
  write_client
  write_pages
  : >"$E2E_WORK/answers.txt"
  offline_chromium
  # In a command substitution, as the fixture's own scenarios call it: its first poll finds no port.
  CALM=http://127.0.0.1:$(serve_site calm)
  HOG=http://127.0.0.1:$(serve_site hog)
  git init -q -b main "$repo"
  open_path "$repo"
}

teardown() {
  browser_teardown
}

# Whether nothing named like the helper's temp file is left in folder `$1`.
no_temp_in() {
  ! find "$1" -maxdepth 1 -name '*.marley-*.tmp' | grep -q .
}

steps() {
  local dotfiles=$E2E_WORK/dotfiles plugins=$E2E_WORK/xdg/opencode/plugins
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== A: Codex's config, a symlink"
  type_text "codex"
  press "" Return
  settle 4
  click "$CHIP_X" "$CHIP_Y"
  settle 3
  press "CTRL" d
  settle 2
  run "ls -l $CODEX_HOME | cut -c1-10,40-; stat -c '%a %n' $dotfiles/codex.toml; grep -c notification $dotfiles/codex.toml; ls -a $dotfiles $CODEX_HOME | grep -c 'marley-.*tmp'; head -3 $CODEX_HOME/config.toml.marley-backup"
  take 755-01-codex-written || return 0
  expect "the link is still a link" test -L "$CODEX_HOME/config.toml"
  expect "the target took the three keys" grep -q 'notification_method = "osc9"' "$dotfiles/codex.toml"
  expect "the target kept its comment" grep -q "kept in a dotfiles folder" "$dotfiles/codex.toml"
  expect "the target kept mode 600" test "$(stat -c %a "$dotfiles/codex.toml")" = 600
  expect "the backup holds the old text" bash -c "grep -q 'theme = \"dark\"' '$CODEX_HOME/config.toml.marley-backup' && ! grep -q notifications '$CODEX_HOME/config.toml.marley-backup'"
  expect "no temp file beside the target" no_temp_in "$dotfiles"
  expect "no temp file beside the link" no_temp_in "$CODEX_HOME"

  echo "== A: OpenCode's plugin"
  type_text "opencode"
  press "" Return
  settle 4
  click "$CHIP_X" "$CHIP_Y"
  settle 3
  press "CTRL" d
  settle 2
  run "ls -la $plugins | cut -c1-10,40-"
  take 755-02-plugin-written || return 0
  expect "the plugin is in place at 644" test "$(stat -c %a "$plugins/marley.js")" = 644
  expect "no backup beside the plugin" test ! -e "$plugins/marley.js.marley-backup"
  expect "no temp file beside the plugin" no_temp_in "$plugins"

  echo "== B: a terminal's own id and token"
  # The left terminal's id, which the right one borrows below.
  run "echo \$MARLEY_TERMINAL_ID >$E2E_WORK/left.id"
  palette "pane: split right" 3
  run "env | grep -c '^MARLEY_TERMINAL_TOKEN=[0-9a-f]\{32\}$'; tools-ask"
  take 755-03-honest || return 0
  expect "the caller's own row is self" grep -q "^self rows: 1$" "$E2E_WORK/answers.txt"
  local other
  other=$(cat "$E2E_WORK/left.id")
  echo "the left terminal's id: $other"

  echo "== B: another terminal's id with this terminal's token"
  : >"$E2E_WORK/answers.txt"
  run "MARLEY_TERMINAL_ID=$other tools-ask"
  take 755-04-spoofed || return 0
  expect "no row is self with a borrowed id" grep -q "^self rows: 0$" "$E2E_WORK/answers.txt"

  echo "== C: the Browser tab's unit"
  local unit
  unit=$(browser_unit "$E2E_WORK/repo")
  palette "marley: open browser" 6
  press "CTRL" l
  type_text "$CALM/"
  press "" Return
  settle 4
  take 755-05a-calm || return 0
  systemctl --user show -p OOMPolicy -p ActiveState "$unit" | tee "$E2E_WORK/unit.txt"
  expect "the unit carries OOMPolicy=continue" grep -qx "OOMPolicy=continue" "$E2E_WORK/unit.txt"

  echo "== C: one renderer killed for memory"
  local current cap
  current=$(systemctl --user show -p MemoryCurrent --value "$unit")
  cap=$((current + 400 * 1024 * 1024))
  echo "memory now $current, capped at $cap"
  # Swap off too, or the hungry renderer swaps instead of being killed.
  systemctl --user set-property --runtime "$unit" "MemoryMax=$cap" MemorySwapMax=0 ||
    echo "set-property failed"
  palette "marley: new browser tab" 4
  press "CTRL" l
  type_text "$HOG/"
  press "" Return
  # Before the page starts eating, three seconds in: its tab open beside the calm one.
  settle 2
  take 755-05b-hungry || return 0
  # Marley closes a tab whose renderer dies (`Target.targetCrashed`).
  settle 18
  take 755-06-after-oom || return 0
  systemctl --user show -p ActiveState -p NRestarts "$unit" | tee "$E2E_WORK/after.txt"
  { journalctl --user -u "$unit" --since "-2min" --no-pager 2>/dev/null | grep -i -m3 "oom\|killed" || true; } |
    tee "$E2E_WORK/oom.txt"
  expect "the unit stayed active" grep -qx "ActiveState=active" "$E2E_WORK/after.txt"
  expect "the kernel killed a process for memory" test -s "$E2E_WORK/oom.txt"
}
