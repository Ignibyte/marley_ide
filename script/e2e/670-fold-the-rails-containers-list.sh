# shellcheck shell=bash
# #670's visual check: the rail's Containers list folds under a header with a chevron and a count,
# starts folded, and each window remembers it. A fake `docker` first on the PATH lists no container
# and records any `stop`, and a stand-in `docker-proxy` (Perl with its `$0` set to the real command
# line's shape, binding nothing) publishes port 670, the lowest, so its row comes first; the
# machine's own containers are there too, so the count is the machine's plus one.
# `marley.rail_containers`, which each run's copy turns off (#669), is turned back on.
#
# `670-01-folded`: a fresh window, the header alone with the chevron right and the count (REQ-001,
# REQ-002, REQ-004). `670-02-open`: a click on the header, the rows under it with :670 first and
# the chevron down (REQ-001, REQ-003). `670-03-folded`: a click on the chevron, folded once
# (REQ-002, REQ-003). `670-04-restored`: unfolded, then Marley quit and started again: still open
# (REQ-005).
compositor sway

PROXY_PID=""

# In the window's logical pixels, from the first run's shots: the Containers header's label and
# its chevron.
HEADER_X=${HEADER_X:-70}
HEADER_Y=${HEADER_Y:-182}
CHEVRON_X=${CHEVRON_X:-16}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/bin"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  profile_setting marley.rail_containers true
  cat >"$E2E_WORK/bin/docker" <<FAKE
#!/bin/sh
# A fake docker for #670's scenario: no container listed, any stop recorded.
case "\$1" in
  stop) echo "stop \$2" >>"$E2E_WORK/stopped.txt" ;;
esac
FAKE
  chmod +x "$E2E_WORK/bin/docker"
  export PATH="$E2E_WORK/bin:$PATH"
  perl -e "\$0 = 'docker-proxy -proto tcp -host-ip 0.0.0.0 -host-port 670 -container-ip 172.18.0.70 -container-port 80'; sleep 900" &
  PROXY_PID=$!
  echo "the stand-in proxy: pid $PROXY_PID for port 670"
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
  shot 670-01-folded

  echo "== the header clicked"
  click "$HEADER_X" "$HEADER_Y"
  settle 2
  pointer_to 900 400
  settle 1
  shot 670-02-open

  echo "== the chevron clicked"
  click "$CHEVRON_X" "$HEADER_Y"
  settle 2
  pointer_to 900 400
  settle 1
  shot 670-03-folded

  echo "== unfolded, then a quit and a launch"
  click "$HEADER_X" "$HEADER_Y"
  settle 2
  quit_marley
  # With no path, as a launch from the menu: a path is an open request, not a restore (L-601).
  open_path ""
  launch_marley
  settle 15
  pointer_to 900 400
  settle 1
  shot 670-04-restored
  if [[ -f $E2E_WORK/stopped.txt ]]; then
    echo "stopped: $(cat "$E2E_WORK/stopped.txt")"
  else
    echo "nothing was stopped"
  fi
}
