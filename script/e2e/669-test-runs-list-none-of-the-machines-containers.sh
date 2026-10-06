# shellcheck shell=bash
# #669's visual check: the rail's Containers list follows `marley.rail_containers`, and each run's
# copy turns it off. A fake `docker` first on the PATH lists no container and records any `stop`,
# and a stand-in `docker-proxy` (Perl with its `$0` set to the real command line's shape, binding
# nothing) publishes port 669, so the run has a container port no project holds whatever the
# machine runs; the machine's own containers, root's real `docker-proxy`, are there too.
#
# `669-01-off`: the copy's setting, off: no Containers section (REQ-001). `669-02-on`: the setting
# turned on in the run's settings: Containers, port 669 first (REQ-002). `669-03-setting`: the
# Settings window's Marley page, searched for Containers (REQ-003).
compositor sway

PROXY_PID=""

# In the window's logical pixels, from the first run's shots: the Settings window's search field.
SETTINGS_SEARCH_X=${SETTINGS_SEARCH_X:-912}
SETTINGS_SEARCH_Y=${SETTINGS_SEARCH_Y:-59}

# Whether the run's copy holds the JSON value `$2` at the dotted key path `$1`.
setting_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, sys

node = json.loads(pathlib.Path(sys.argv[1]).read_text())
for key in sys.argv[2].split("."):
    node = node.get(key) if isinstance(node, dict) else None
sys.exit(0 if node == json.loads(sys.argv[3]) else 1)
SETTINGS
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/bin"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  expect "the run's copy turns the Containers list off" setting_is marley.rail_containers false
  cat >"$E2E_WORK/bin/docker" <<FAKE
#!/bin/sh
# A fake docker for #669's scenario: no container listed, any stop recorded.
case "\$1" in
  stop) echo "stop \$2" >>"$E2E_WORK/stopped.txt" ;;
esac
FAKE
  chmod +x "$E2E_WORK/bin/docker"
  export PATH="$E2E_WORK/bin:$PATH"
  perl -e "\$0 = 'docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port 669 -container-ip 172.18.0.69 -container-port 80'; sleep 900" &
  PROXY_PID=$!
  echo "the stand-in proxy: pid $PROXY_PID for port 669"
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -n $PROXY_PID ]]; then
    kill "$PROXY_PID" 2>/dev/null || true
  fi
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 8
  shot 669-01-off

  echo "== the list turned on"
  profile_setting marley.rail_containers true
  settle 6
  shot 669-02-on

  echo "== the setting"
  palette "marley: open settings"
  settle 4
  click "$SETTINGS_SEARCH_X" "$SETTINGS_SEARCH_Y"
  settle 1
  type_text "Containers"
  settle 3
  shot 669-03-setting
  if [[ -f $E2E_WORK/stopped.txt ]]; then
    echo "stopped: $(cat "$E2E_WORK/stopped.txt")"
  else
    echo "nothing was stopped"
  fi
}
