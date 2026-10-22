# shellcheck shell=bash
# #890's visual check: port rows, the Containers panel and Collab as the Splits board draws them.
# Project `repo`; its terminal serves `python3 -m http.server` on a free port. A fake `docker`
# first on the PATH (L-claude-603) lists `postgres-890`, whose Compose folder is `repo`, and
# `mailpit-890`, held by no project, publishing two ports; stand-in `docker-proxy` processes (Perl
# with `$0` set to the real command line's shape, binding nothing) hold their command lines. The
# box's own Docker ports, root's real `docker-proxy`, show in the panel too, unnamed.
# - `890-01-port-row`: under `repo`, the server's row and `postgres-890`'s: the port on a tile,
#   the URL as a link over what serves it, Open or Copy at the end (REQ-001).
# - `890-02-containers`: the Containers panel: one row for `mailpit-890` with both ports, and
#   Stop (REQ-002).
# - `890-03-collab`: the Collab panel signed out: the card (REQ-003).
# - `890-04-day`: the panel and the rail in Marley Day (REQ-001, REQ-002).
# - `890-05-collab-day`: Collab in Marley Day (REQ-003).
# - `stopped.txt`: `mailpit-890`'s Stop clicked: the fake docker asked to stop it (REQ-002).
compositor sway

PROXY_PIDS=()
WEB_PORT=""

AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-500}
# From the probe run's shots: the bottom row's Containers and Collab buttons.
BOX_X=${BOX_X:-1402}
BOX_Y=${BOX_Y:-985}
COLLAB_X=${COLLAB_X:-1478}
COLLAB_Y=${COLLAB_Y:-985}
# The Containers tab's first row's Stop, `mailpit-890`'s.
STOP_X=${STOP_X:-1567}
STOP_Y=${STOP_Y:-131}
# A first run that stops after the rail's shot, to read the positions.
STOP_AT=${STOP_AT:-}

free_port() {
  python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}

proxy() {
  perl -e "\$0 = 'docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port $1 -container-ip $2 -container-port $3'; sleep 900" &
  PROXY_PIDS+=("$!")
}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/bin"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  WEB_PORT=$(free_port)
  profile_setting marley.rail_containers true
  profile_setting marley.tmux_hosts '[]'
  cat >"$E2E_WORK/bin/docker" <<FAKE
#!/bin/sh
# A fake docker for #890's scenario: two containers listed, any stop recorded.
case "\$1" in
  ps)
    printf '%s\n' '{"ID":"pg890","Names":"postgres-890","Ports":"0.0.0.0:890->5432/tcp","Labels":"com.docker.compose.project=repo,com.docker.compose.project.working_dir=$E2E_WORK/repo","Image":"postgres"}'
    printf '%s\n' '{"ID":"mp890","Names":"mailpit-890","Ports":"0.0.0.0:891->8025/tcp, 0.0.0.0:892->1025/tcp","Labels":"","Image":"mailpit"}'
    ;;
  stop) echo "stop \$2" >>"$E2E_WORK/stopped.txt" ;;
esac
FAKE
  chmod +x "$E2E_WORK/bin/docker"
  export PATH="$E2E_WORK/bin:$PATH"
  proxy 890 172.18.0.90 5432
  proxy 891 172.18.0.91 8025
  proxy 892 172.18.0.91 1025
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  for pid in "${PROXY_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
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

  echo "== a web server in repo's terminal"
  type_text "python3 -m http.server $WEB_PORT --bind 127.0.0.1"
  press "" Return
  settle 10
  away
  shot 890-01-port-row
  [[ $STOP_AT == 1 ]] && return 0

  echo "== the Containers panel"
  click "$BOX_X" "$BOX_Y"
  settle 3
  away
  shot 890-02-containers

  echo "== Collab"
  click "$COLLAB_X" "$COLLAB_Y"
  settle 3
  away
  shot 890-03-collab

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 3
  click "$BOX_X" "$BOX_Y"
  settle 3
  away
  shot 890-04-day
  click "$COLLAB_X" "$COLLAB_Y"
  settle 3
  away
  shot 890-05-collab-day

  echo "== mailpit-890's Stop"
  click "$BOX_X" "$BOX_Y"
  settle 2
  click "$STOP_X" "$STOP_Y"
  settle 3
  if [[ -f $E2E_WORK/stopped.txt ]]; then
    cat "$E2E_WORK/stopped.txt"
  else
    echo "the fake docker was not asked to stop anything"
  fi | tee "$(shot_file stopped.txt)"
}
