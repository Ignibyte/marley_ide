# shellcheck shell=bash
# #846's visual check: MCP Servers as a card per agent, as the Tools board draws them. A scratch
# repository; stand-in `claude` and `codex` first on the PATH; the run's Claude Code config is its
# own (#877), so the page lists none of the user's servers.
# - `marley: open mcp servers`: Claude Code's, Codex's and Zed's cards (`846-01-servers`, REQ-001).
# - `marley: open agent activity`, shot as it is (`846-02-activity`).
# - Marley Day, MCP Servers (`846-03-day`, REQ-001).
compositor sway

STOP_AT=${STOP_AT:-0}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name
  mkdir -p "$home" "$bin" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in claude codex; do
    printf '#!/bin/sh\nexec sleep 600\n' >"$bin/$name"
    chmod +x "$bin/$name"
  done
  export PATH=$bin:$PATH
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== MCP Servers"
  palette "marley: open mcp servers" 5
  pointer_to 1500 900
  settle 1
  shot 846-01-servers

  echo "== Agent Activity"
  palette "marley: open agent activity" 4
  shot 846-02-activity
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Marley Day"
  palette "marley: open mcp servers" 3
  profile_setting theme '"Marley Day"'
  settle 4
  shot 846-03-day
}
