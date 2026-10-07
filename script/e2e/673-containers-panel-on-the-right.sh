# shellcheck shell=bash
# #673's visual check: the machine's container ports leave the rail for the Containers panel on the
# right, opened from its button in the status bar. A fake `docker` first on the PATH lists no
# container and records any `stop`, and a stand-in `docker-proxy` (Perl with its `$0` set to the
# real command line's shape, binding nothing) publishes port 673, as #670's scenario does; the
# machine's own containers are listed too. `marley.rail_containers`, which each run's copy turns
# off (#669), is turned back on.
#
# `673-01-rail`: a fresh window, the rail with no Containers section and the status bar's box
# button (REQ-001, REQ-002). `673-02-panel`: the button clicked, the panel on the right with :673
# first (REQ-003). `673-03-browser`: :673's row double-clicked, a Browser tab on its URL
# (REQ-004). `673-04-off`: the setting off with the panel open: no button, and the panel closed
# (REQ-005).
compositor sway

PROXY_PID=""

# In the window's logical pixels, from the first run's shots: the status bar's box button, the
# panel's first row, and a point clear of both.
BOX_X=${BOX_X:-1343}
BOX_Y=${BOX_Y:-977}
ROW_X=${ROW_X:-1450}
ROW_Y=${ROW_Y:-120}
AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-500}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/bin"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  profile_setting marley.rail_containers true
  cat >"$E2E_WORK/bin/docker" <<FAKE
#!/bin/sh
# A fake docker for #673's scenario: no container listed, any stop recorded.
case "\$1" in
  stop) echo "stop \$2" >>"$E2E_WORK/stopped.txt" ;;
esac
FAKE
  chmod +x "$E2E_WORK/bin/docker"
  export PATH="$E2E_WORK/bin:$PATH"
  perl -e "\$0 = 'docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port 673 -container-ip 172.18.0.73 -container-port 80'; sleep 900" &
  PROXY_PID=$!
  echo "the stand-in proxy: pid $PROXY_PID for port 673"
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
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 673-01-rail

  echo "== the status bar's box button"
  click "$BOX_X" "$BOX_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 673-02-panel

  echo "== :673's row double-clicked"
  click "$ROW_X" "$ROW_Y"
  click "$ROW_X" "$ROW_Y"
  settle 6
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 673-03-browser

  echo "== the setting off"
  profile_setting marley.rail_containers false
  settle 4
  shot 673-04-off
  if [[ -f $E2E_WORK/stopped.txt ]]; then
    echo "stopped: $(cat "$E2E_WORK/stopped.txt")"
  else
    echo "nothing was stopped"
  fi
}
