# shellcheck shell=bash
# #761's visual check: Marley's keyring label. Marley runs on a private session bus with a
# `gnome-keyring-daemon` of the run's own (its keyrings in the run's folder, its control socket in a
# short folder under the runtime directory), so nothing reaches the user's keyring. Before Marley
# starts, that keyring holds System One's key for the run's `compatible` endpoint under stock Zed's
# old label, `zed-github-account`.
# Shots: System One calls reads the key, and the keyring holds it under `Marley` too (REQ-002);
# Forget Key leaves no item for the URL under either label (REQ-003); Set Key writes one item,
# labelled `Marley` (REQ-001).
compositor sway

ENDPOINT=http://127.0.0.1:9/v1/systemone
# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
FORGET_X=${FORGET_X:-460}
FORGET_Y=${FORGET_Y:-209}
SET_X=${SET_X:-386}
SET_Y=${SET_Y:-209}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  # The keyring's control socket must fit a Unix socket's 108 bytes.
  KEYRING_RUN=$(mktemp -d "$XDG_RUNTIME_DIR/k761.XXXXXX")
  mkdir -p "$E2E_WORK/keyring-data"
  printf 'e2e' | env XDG_DATA_HOME="$E2E_WORK/keyring-data" XDG_RUNTIME_DIR="$KEYRING_RUN" \
    gnome-keyring-daemon --login >"$E2E_WORK/keyring-login.log" 2>&1 &
  sleep 1
  env XDG_DATA_HOME="$E2E_WORK/keyring-data" XDG_RUNTIME_DIR="$KEYRING_RUN" \
    gnome-keyring-daemon --start --components=secrets >"$E2E_WORK/keyring-start.log" 2>&1
  printf 'old-key-761' | timeout 10 secret-tool store --label=zed-github-account \
    url "$ENDPOINT" username system-one
  keyring_items | tee "$E2E_WORK/keyring-before.txt"
  profile_setting marley.system_one "{\"enabled\": true, \"provider\": \"compatible\", \"endpoint\": \"$ENDPOINT\"}"
  mkdir -p "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  if [[ -n ${KEYRING_RUN:-} && -d $KEYRING_RUN ]]; then
    for pid in $(pgrep -x gnome-keyring-d || true); do
      tr '\0' '\n' <"/proc/$pid/environ" 2>/dev/null | grep -qx "XDG_RUNTIME_DIR=$KEYRING_RUN" &&
        kill "$pid"
    done
    rm -rf -- "$KEYRING_RUN"
  fi
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The keyring's items for the endpoint: one `label secret` line each, sorted.
keyring_items() {
  timeout 10 secret-tool search --all url "$ENDPOINT" 2>/dev/null | python3 -c 'import sys
items, item = [], {}
for line in sys.stdin:
    if line.startswith("["):
        item = {}
        items.append(item)
    elif " = " in line:
        key, value = line.rstrip("\n").split(" = ", 1)
        item[key] = value
print("\n".join(sorted(str(i.get("label")) + " " + str(i.get("secret")) for i in items)))'
}

items_are() { [[ $(keyring_items) == "$1" ]]; }

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 2

  echo "== the key under the old label is read and copied"
  palette "marley: open system one calls"
  settle 4
  take 761-01-read || return 0
  keyring_items
  expect "the keyring holds the key under both labels" items_are \
    "$(printf 'Marley old-key-761\nzed-github-account old-key-761')"
  if ((FORGET_Y == 0)); then
    echo "Forget Key is not placed yet: 761-01 is the last shot"
    return 0
  fi

  echo "== Forget Key"
  click "$FORGET_X" "$FORGET_Y"
  settle 3
  take 761-02-forgotten || return 0
  keyring_items
  expect "no item is left for the endpoint" items_are ""

  echo "== Set Key"
  click "$SET_X" "$SET_Y"
  settle 2
  type_text "new-key-761"
  press "" Return
  settle 3
  take 761-03-set || return 0
  keyring_items
  expect "the one item is labelled Marley" items_are "Marley new-key-761"
}
