# shellcheck shell=bash
# #701's visual check: Home's own page. A fresh profile opens `repo`, then quits; Marley starts
# again on `repo2`, so `repo` is a recent project not open. A fake `claude`, first on the terminals'
# PATH through the scenario's own `.bashrc`, prints a line every half second for a minute, so its
# terminal reads as an agent at work; no real Claude Code runs (PR-687).
#
# `701-01-home-page`: Home's header clicked: the page, its five cards (REQ-001).
# `701-02-recent`: `repo` clicked under Recent Projects: it opens in the rail (REQ-002).
# `701-03-terminal`: New Terminal: a terminal tab in Home (REQ-003).
# `701-04-agent`: Claude Code under New Agent: the fake `claude` in a terminal in Home (REQ-004).
# `701-05-at-work`: back on the page: Agents at Work lists that terminal, working (REQ-005).
# `701-06-emptied`: `pane: close all items` in Home, the agent's close allowed: the page, not Zed's
# Welcome page (REQ-006).
compositor sway

# In the window's logical pixels, from the first run's shots.
HOME_X=${HOME_X:-100}
HOME_Y=${HOME_Y:-89}
RECENT_X=${RECENT_X:-330}
RECENT_Y=${RECENT_Y:-345}
TERMINAL_X=${TERMINAL_X:-354}
TERMINAL_Y=${TERMINAL_Y:-134}
AGENT_X=${AGENT_X:-350}
AGENT_Y=${AGENT_Y:-225}
HOME_TAB_X=${HOME_TAB_X:-370}
HOME_TAB_Y=${HOME_TAB_Y:-51}
TRUST_X=${TRUST_X:-1003}
TRUST_Y=${TRUST_Y:-377}
CLOSE_X=${CLOSE_X:-798}
CLOSE_Y=${CLOSE_Y:-522}
AWAY_X=${AWAY_X:-1000}
AWAY_Y=${AWAY_Y:-900}
# A first run that stops after the page's shot, to read the positions.
STOP_AT=${STOP_AT:-}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/repo2" "$E2E_WORK/home"
  cat >"$E2E_WORK/home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$E2E_WORK/home"
  rm -rf "$E2E_PROFILE/db"
  # A Python script, which the rail names by its file, as #519's stand-in is.
  cat >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in for Claude Code in #701's scenario: it prints a line every half second for a minute.
import time

for tick in range(1, 121):
    print("fake claude at work: %d" % tick, flush=True)
    time.sleep(0.5)
FAKE
  chmod +x "$bin/claude"
  profile_setting marley.rusty.enabled false
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  printf '# repo2\n' >"$E2E_WORK/repo2/README.md"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 3

  echo "== repo once, then Marley again on repo2"
  # The rail's Home header, so the quit's keys leave the terminal (L-700).
  click "$HOME_X" "$HOME_Y"
  settle 2
  quit_marley
  open_path "$E2E_WORK/repo2"
  launch_marley
  settle 15
  # Trusts repo2 with the prompt's button: after a relaunch the window takes no keys until a
  # click (L-700).
  click "$TRUST_X" "$TRUST_Y"
  settle 3

  echo "== Home's page"
  click "$HOME_X" "$HOME_Y"
  settle 4
  away
  shot 701-01-home-page
  [[ $STOP_AT == 1 ]] && return 0

  echo "== a recent project"
  click "$RECENT_X" "$RECENT_Y"
  settle 5
  away
  shot 701-02-recent

  echo "== New Terminal"
  click "$HOME_X" "$HOME_Y"
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 4
  away
  shot 701-03-terminal

  echo "== Claude Code under New Agent"
  click "$HOME_TAB_X" "$HOME_TAB_Y"
  settle 2
  click "$AGENT_X" "$AGENT_Y"
  settle 6
  away
  shot 701-04-agent

  echo "== Agents at Work"
  click "$HOME_TAB_X" "$HOME_TAB_Y"
  settle 3
  away
  shot 701-05-at-work

  echo "== every tab closed"
  click "$HOME_TAB_X" "$HOME_TAB_Y"
  settle 1
  palette "pane: close all items"
  settle 2
  # Marley asks before a working agent's terminal closes (#550): Close.
  click "$CLOSE_X" "$CLOSE_Y"
  settle 3
  away
  shot 701-06-emptied
}
