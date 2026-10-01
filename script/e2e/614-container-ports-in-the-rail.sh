# shellcheck shell=bash
# #614's e2e test: container ports in the rail. A fake `docker` first on the PATH (L-claude-603)
# lists a container `web-614` publishing a free port, its Compose folder the scratch project, and
# records `stop`; a stand-in `docker-proxy` (Perl with its `$0` set to the real command line's
# shape) holds that port's command line. The dev box's own Docker ports, root's real
# `docker-proxy`, show too, under Containers, since the fake does not list them.
#
# The port under `repo`, named "container web-614" (`named`, REQ-001); Stop runs `docker stop
# web-614` (`stopped.txt`, REQ-002); with the fake refusing, the port under Containers with its
# target, and Stop's toast with the reason and Copy Command (`refused`, REQ-003).
compositor sway

PORT=""
PROXY_PID=""

# The rail on the 1600 x 1000 output: `web-614`'s row under `repo`.
ROW_X=130
WEB_Y=188
# The row's Stop button, shown on hover at its end.
STOP_X=233
# The stand-in's row under Containers once the engine refuses: the third, after the box's two.
APART_Y=${APART_Y:-338}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/bin"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  PORT=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
  echo answer >"$E2E_WORK/docker-mode"
  cat >"$E2E_WORK/bin/docker" <<FAKE
#!/bin/sh
# A fake docker for #614's scenario.
if [ "\$(cat $E2E_WORK/docker-mode)" = refuse ]; then
  echo "permission denied while trying to connect to the docker API at unix:///var/run/docker.sock" >&2
  exit 1
fi
case "\$1" in
  ps)
    printf '%s\n' '{"ID":"abc614","Names":"web-614","Ports":"0.0.0.0:$PORT->80/tcp, [::]:$PORT->80/tcp","Labels":"com.docker.compose.project=repo,com.docker.compose.project.working_dir=$E2E_WORK/repo","Image":"nginx"}'
    ;;
  stop)
    echo "stop \$2" >>"$E2E_WORK/stopped.txt"
    echo "\$2"
    ;;
esac
FAKE
  chmod +x "$E2E_WORK/bin/docker"
  export PATH="$E2E_WORK/bin:$PATH"
  perl -e "\$0 = 'docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port $PORT -container-ip 172.18.0.5 -container-port 80'; sleep 900" &
  PROXY_PID=$!
  echo "the stand-in proxy: pid $PROXY_PID for port $PORT"
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
  shot named
  pointer_to "$ROW_X" "$WEB_Y"
  settle 1.5
  shot hover

  echo "== Stop on web-614"
  click "$STOP_X" "$WEB_Y"
  settle 3
  if [[ -f $E2E_WORK/stopped.txt ]]; then
    cat "$E2E_WORK/stopped.txt"
  else
    echo "the fake docker was not asked to stop anything"
  fi | tee "$(shot_file stopped.txt)"

  echo "== the engine refusing"
  echo refuse >"$E2E_WORK/docker-mode"
  # The engine's answer is kept 30 s while its ports stay the same.
  settle 36
  shot apart
  pointer_to "$ROW_X" "$APART_Y"
  settle 1.5
  click "$STOP_X" "$APART_Y"
  settle 2
  shot refused
}

