# shellcheck shell=bash
# #748's visual check: Collab, Fleet, Knowledge and Containers open as tabs. Rusty on through
# `marley_rusty`'s stand-in over a scratch vault (#646's pages); Fleet on its pseudo provider
# (#607); a fake `docker` and a stand-in `docker-proxy` publishing port 748 with
# `marley.rail_containers` on (#673). Collab is signed out.
# - The status bar's Collab button: a Collab tab, no right dock (`collab`, REQ-001).
# - Fleet's button: a Fleet tab listing the pseudo agents (`fleet`, REQ-001, REQ-002).
# - The Demo page open, then Knowledge's button: a Knowledge tab, in front, still showing Demo's
#   tags and links (`knowledge`, REQ-001, REQ-003).
# - Containers' button: a Containers tab listing :748 (`containers`, REQ-001).
# - Rusty turned off: the Knowledge tab closes (`rusty-off`, REQ-004).
compositor sway

# The status bar's buttons, as the first run's shots found them; `STOP_AT_BAR=1` ends the run at
# the first shot.
COLLAB_X=${COLLAB_X:-1507}
FLEET_X=${FLEET_X:-1455}
KNOWLEDGE_X=${KNOWLEDGE_X:-1428}
CONTAINERS_X=${CONTAINERS_X:-1402}
BAR_Y=${BAR_Y:-985}
STOP_AT_BAR=${STOP_AT_BAR:-0}

# A made-up page `$1` in the stand-in's scratch vault, its text from stdin.
vault_page() {
  local path=$E2E_WORK/rusty/vault/$1.md
  mkdir -p "$(dirname "$path")"
  cat >"$path"
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-d": ["rusty::OpenPage", {"slug": "projects/demo"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  vault_page projects/demo <<'PAGE'
---
title: Demo
type: project
tags: [project, area/demo]
---
# Demo

A demo project, still a #draft.

It orbits [[concepts/orbit|Orbit]] and waits on [[Missing Page]].
PAGE
  vault_page concepts/orbit <<'PAGE'
---
title: Orbit
tags: [area/demo]
---
# Orbit

The idea [[projects/demo]] runs on.
PAGE
  vault_page decisions/demo-uses-orbit <<'PAGE'
---
title: Demo uses Orbit
---
# Demo uses Orbit

We decided that [[projects/demo|Demo]] keeps orbit.
PAGE
  profile_setting marley.fleet '{"providers": [{"kind": "pseudo"}]}'
  profile_setting marley.rail_containers true
  cat >"$bin/docker" <<FAKE
#!/bin/sh
# A fake docker for #748's scenario: no container listed.
exit 0
FAKE
  chmod +x "$bin/docker"
  export PATH="$bin:$PATH"
  perl -e "\$0 = 'docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port 748 -container-ip 172.18.0.48 -container-port 80'; sleep 900" &
  PROXY_PID=$!
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -n ${PROXY_PID:-} ]]; then
    kill "$PROXY_PID" 2>/dev/null || true
  fi
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 6
  shot 748-00-bar
  if [[ $STOP_AT_BAR == 1 ]]; then
    return
  fi

  echo "== Collab"
  click "$COLLAB_X" "$BAR_Y"
  settle 2
  shot 748-01-collab

  echo "== Fleet"
  click "$FLEET_X" "$BAR_Y"
  settle 4
  shot 748-02-fleet

  echo "== the Demo page, then Knowledge"
  press "CTRL ALT SHIFT" d
  settle 4
  click "$KNOWLEDGE_X" "$BAR_Y"
  settle 4
  shot 748-03-knowledge

  echo "== Containers"
  click "$CONTAINERS_X" "$BAR_Y"
  settle 4
  shot 748-04-containers

  echo "== Rusty off"
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
settings["marley"]["rusty"]["enabled"] = False
json.dump(settings, open(path, "w"), indent=2)
PY
  settle 4
  shot 748-05-rusty-off
}
