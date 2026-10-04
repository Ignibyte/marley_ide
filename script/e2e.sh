#!/usr/bin/env bash
# e2e.sh <scenario>: a ticket's e2e visualization test on the dev box (CONSTITUTION §7).
#
# Runs the debug `marley` on a copy of the user's Marley profile and runs the scenario against
# it, on the backend the scenario names with `compositor <name>` at its top level (or
# COMPOSITOR in the environment):
#
# - `hyprland`, the default: Marley on the user's Hyprland, on hidden workspace 9. Keys go to
#   Marley's window only (Hyprland's `send_key_state` with the window named), so the user's
#   focus stays where it is, and each shot is the window by its toplevel (`grim -T` with
#   Hyprland's `stableId`), so nothing shows on the screen in use
#   (L-claude-467-capture-one-window-by-its-toplevel-001). No mouse: Hyprland cannot send a
#   pointer event to one window, and a click would move the user's pointer.
# - `sway`: Marley in a headless sway of its own, one output of SIZE (1600x1000), whose seat is
#   a virtual pointer (script/e2e/seat-pointer.c, built on first use) and a virtual keyboard
#   (`wtype`) nothing else sees. A scenario that clicks, drags or scrolls runs here; the user's
#   desktop and Hyprland are never touched.
#
# The scenario, `script/e2e/<ticket>-<slug>.sh`, is sourced. It defines `steps`, run once the
# window maps, and may define `setup`, run before the launch with E2E_PROFILE (the profile copy)
# and E2E_WORK (a scratch folder) set: it builds its fixtures there, may name a path for Marley
# to open with `open_path <path>`, may set variables for Marley's terminals with
# `terminal_env <name> <value>` (a HOME whose .bashrc is the scenario's, say, so no test depends
# on the user's shell), and may prepend to PATH (fakes that Marley and its terminals find first).
# It may define `teardown`, run however the run ends and before Marley stops, for anything it
# started outside Marley. OPEN in the environment names the path when the scenario names none.
# `binary <path>`, at the top level or in `setup`, runs that build instead of the debug one (the
# installed release build, #502). From `steps` on, E2E_MARLEY names the binary the run starts,
# E2E_CLASS its windows' class and E2E_LOG the log its output goes to (#513).
#
# Steps: `settle <seconds>`; `press <mods> <key>`, with mods as Hyprland names them ("" for none,
# "CTRL SHIFT" for two) and the key by its xkb name (`Return`, `Escape`, `g`); `press_keys
# <key>...`, several keys in one go (a compose sequence); `type_text <text>`;
# `shot <name>`, which writes SHOT_DIR/<name>.png and prints its path; and `quit_marley`, which
# quits Marley through its palette and waits for it to exit, and `launch_marley`, which starts it
# again on the same profile with the same path, for what Marley restores (#494); and `expect
# <what> <command...>`, a check that prints its verdict and fails the run when the command fails,
# with `holds <file> <text>...` for the common one (#517), so a run passes or fails with no one
# reading its shots. Under sway also
# `click <x> <y> [button]`, `click_with <mods> <x> <y>` (a click with modifiers held, #503),
# `pointer_to <x> <y>`, `pointer_down [button]`, `pointer_up [button]` and `scroll <steps>`
# (wheel detents at the pointer, positive down), in the window's pixels, with the buttons left,
# middle and right. A step started in the
# background is waited for by its pid (`wait "$pid"`): the run's own helpers are background jobs
# of the same shell, so a bare `wait` waits for them too, until the run is killed.
#
#   SHOT_DIR where the PNGs and Marley's log land ($TMPDIR/marley-shots); never the repository
#   E2E_BINARY the build to run instead of the debug one, unless the scenario names its own (#517)
set -euo pipefail

scenario=${1:?usage: e2e.sh <scenario>}
[[ -f $scenario ]] || { echo "no scenario $scenario" >&2; exit 2; }
scenario=$(realpath "$scenario")
cd "$(dirname "$0")/.."
shots=${SHOT_DIR:-${TMPDIR:-/tmp}/marley-shots}
marley=$(realpath -m "${CARGO_TARGET_DIR:-target}/debug/marley")
config=${XDG_CONFIG_HOME:-$HOME/.config}/marley
data=${XDG_DATA_HOME:-$HOME/.local/share}/marley
class=dev.zed.Zed-Dev
name=$(basename "$scenario" .sh)

export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-/run/user/$(id -u)}

open_path() { OPEN=$1; }

# E2E_BINARY in the environment names the build for every scenario (script/regress, #517); a
# scenario's own `binary` still wins.
MARLEY_BIN=${E2E_BINARY:-}
binary() { MARLEY_BIN=$1; }

# The scenario's backend, `hyprland` or `sway`, named at its top level; SIZE may follow it.
compositor() { COMPOSITOR=$1; }

TERMINAL_ENV=()
terminal_env() { TERMINAL_ENV+=("$1" "$2"); }

OPEN=${OPEN:-}
COMPOSITOR=${COMPOSITOR:-hyprland}
SIZE=${SIZE:-1600x1000}
# shellcheck source=/dev/null
. "$scenario"
declare -F steps >/dev/null || { echo "$scenario defines no steps" >&2; exit 2; }

case $COMPOSITOR in
  hyprland) needed=(hyprctl grim jq python3) ;;
  sway) needed=(sway swaymsg wtype grim jq python3 cc wayland-scanner pkg-config) ;;
  *)
    echo "COMPOSITOR is hyprland or sway, not $COMPOSITOR" >&2
    exit 2
    ;;
esac
missing=()
for tool in "${needed[@]}"; do
  command -v "$tool" >/dev/null || missing+=("$tool")
done
if [[ ${#missing[@]} -gt 0 ]]; then
  echo "the $COMPOSITOR backend needs ${missing[*]}" >&2
  exit 1
fi
if [[ ! $SIZE =~ ^([0-9]+)x([0-9]+)$ ]]; then
  echo "SIZE is <width>x<height>, not $SIZE" >&2
  exit 2
fi
width=${BASH_REMATCH[1]}
height=${BASH_REMATCH[2]}

# --- Hyprland ------------------------------------------------------------------------------

# The user's Hyprland instance, for hyprctl, when the environment does not name it.
hyprland_signature() {
  if [[ -z ${HYPRLAND_INSTANCE_SIGNATURE:-} && -d $XDG_RUNTIME_DIR/hypr ]]; then
    HYPRLAND_INSTANCE_SIGNATURE=$(find "$XDG_RUNTIME_DIR/hypr" -mindepth 1 -maxdepth 1 \
      -printf '%T@ %f\n' | sort -rn | head -1 | cut -d' ' -f2)
    export HYPRLAND_INSTANCE_SIGNATURE
  fi
}

# The first Marley window's `field` (`pid`, `stableId`, `address`) on Hyprland, or nothing.
marley_window() {
  hyprctl clients -j | jq -r --arg class "$class" --arg field "$1" \
    '[.[] | select(.class == $class)][0][$field] // empty'
}

# The user's active window and workspace, to report whether the run moved them.
user_focus() {
  printf '%s %s' "$(hyprctl activewindow -j | jq -r '.address // "none"')" \
    "$(hyprctl activeworkspace -j | jq -r '.id')"
}

# How many Marley windows the user's Hyprland shows, read only; "unknown" without a Hyprland.
hyprland_marley_windows() {
  if [[ -n ${HYPRLAND_INSTANCE_SIGNATURE:-} ]] && command -v hyprctl >/dev/null; then
    hyprctl clients -j | jq --arg class "$class" '[.[] | select(.class == $class)] | length'
  else
    echo unknown
  fi
}

# --- Sway ----------------------------------------------------------------------------------

SWAY_DIR=
SWAY_DISPLAY=
SWAY_SOCK=
KEY_HOLDER=
SEAT_POINTER_PID=
# The pid of the run's Marley as its window last answered, and sway's exit status when it exited
# before the run asked it to (#588).
MARLEY_PID=
COMPOSITOR_EXIT=

sway_msg() { swaymsg -s "$SWAY_SOCK" "$@"; }

# The pid of Marley's window in the sway, or nothing.
sway_marley_pid() {
  sway_msg -t get_tree | jq -r --arg class "$class" \
    '[.. | objects | select(.app_id? == $class)][0].pid // empty'
}

# The pointer helper, built from its source into the shots folder unless a build of this very
# source is there.
seat_pointer() {
  local source=script/e2e/seat-pointer.c xml=script/e2e/wlr-virtual-pointer-unstable-v1.xml
  local build flags
  build=$shots/.seat-pointer-$(cat "$source" "$xml" | sha256sum | cut -c1-12)
  if [[ ! -x $build/seat-pointer ]]; then
    mkdir -p "$build"
    wayland-scanner client-header "$xml" "$build/wlr-virtual-pointer-unstable-v1-client-protocol.h"
    wayland-scanner private-code "$xml" "$build/wlr-virtual-pointer-unstable-v1-protocol.c"
    read -ra flags < <(pkg-config --cflags --libs wayland-client)
    cc -O2 -Wall -Wextra -Wno-unused-parameter -I"$build" -o "$build/seat-pointer" "$source" \
      "$build/wlr-virtual-pointer-unstable-v1-protocol.c" "${flags[@]}"
  fi
  echo "$build/seat-pointer"
}

# Starts the headless sway, then gives its seat a pointer, before Marley starts: a seat with no
# devices never hands a client a wl_pointer or a wl_keyboard. `launch_marley` gives it the
# keyboard.
sway_start() {
  local helper ready
  helper=$(seat_pointer)
  SWAY_DIR=$(mktemp -d "$shots/sway.XXXXXX")
  cat >"$SWAY_DIR/config" <<EOF
output HEADLESS-1 mode $SIZE position 0 0
default_border none
default_floating_border none
focus_follows_mouse no
exec sh -c 'printf "%s\\n%s\\n" "\$WAYLAND_DISPLAY" "\$SWAYSOCK" > "$SWAY_DIR/session"'
EOF
  # A wrapper keeps sway's exit status, since `setsid -f` leaves no child to wait for; verbose,
  # so an exit the run did not ask for has its reason in the log (#588).
  cat >"$SWAY_DIR/run" <<'RUN'
#!/bin/sh
sway --verbose -c "$1"
echo "$?" >"$2"
RUN
  chmod +x "$SWAY_DIR/run"
  env -u WAYLAND_DISPLAY -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE -u SWAYSOCK \
    WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 \
    setsid -f "$SWAY_DIR/run" "$SWAY_DIR/config" "$SWAY_DIR/exit" \
    >"$shots/$name.sway.log" 2>&1 </dev/null
  for _ in $(seq 50); do
    [[ -s $SWAY_DIR/session ]] && break
    sleep 0.1
  done
  if [[ ! -s $SWAY_DIR/session ]]; then
    echo "sway did not start; see $shots/$name.sway.log" >&2
    return 1
  fi
  { read -r SWAY_DISPLAY; read -r SWAY_SOCK; } <"$SWAY_DIR/session"
  coproc SEAT_POINTER {
    WAYLAND_DISPLAY=$SWAY_DISPLAY exec "$helper" "$width" "$height" 2>>"$shots/$name.sway.log"
  }
  if ! IFS= read -r -t 5 ready <&"${SEAT_POINTER[0]}" || [[ $ready != ready ]]; then
    echo "the pointer helper did not start; see $shots/$name.sway.log" >&2
    return 1
  fi
}

# Gives the sway's seat a keyboard that stays: a virtual keyboard that holds on while it sleeps.
# Each step's `wtype` makes a keyboard of its own and drops it, so a Marley that binds the seat's
# keyboard after one is gone gets no keymap, which gpui cannot run without (#494); a new holder
# before each launch is the seat's live keyboard.
hold_keyboard() {
  if [[ -n $KEY_HOLDER ]]; then
    kill "$KEY_HOLDER" 2>/dev/null || true
  fi
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -s 86400000 &
  KEY_HOLDER=$!
  sleep 0.2
}

sway_stop() {
  local pid=
  # A status in the run's sway folder before the run asks sway to exit: it exited on its own.
  if [[ -n $SWAY_DIR && -s $SWAY_DIR/exit ]]; then
    COMPOSITOR_EXIT=$(cat "$SWAY_DIR/exit")
  fi
  if [[ -n $SWAY_SOCK ]]; then
    pid=$(sway_marley_pid 2>/dev/null || true)
  fi
  # Without its compositor the run's Marley has no window to be found by: the pid its window last
  # gave, while that process still runs on the run's profile.
  if [[ -z $pid && -n $MARLEY_PID ]] &&
    grep -qaF -- "$E2E_PROFILE" "/proc/$MARLEY_PID/cmdline" 2>/dev/null; then
    pid=$MARLEY_PID
  fi
  if [[ -n $pid ]]; then
    kill -TERM "$pid" 2>/dev/null || true
    sleep 1
  fi
  # Bash unsets a coprocess's pid once it reaps it, as when the compositor took the helper with it.
  if [[ -n ${SEAT_POINTER_PID:-} ]]; then
    kill "$SEAT_POINTER_PID" 2>/dev/null || true
  fi
  if [[ -n $KEY_HOLDER ]]; then
    kill "$KEY_HOLDER" 2>/dev/null || true
  fi
  if [[ -n $SWAY_SOCK ]]; then
    sway_msg exit >/dev/null 2>&1 || true
    sleep 0.5
  elif [[ -n $SWAY_DIR ]]; then
    # It started but never said where it listens.
    pkill -f -- "-c $SWAY_DIR/config" || true
    sleep 0.5
  fi
  if [[ -n $SWAY_DIR ]] && pgrep -f -- "-c $SWAY_DIR/config" >/dev/null; then
    echo "sway: still running after the run" >&2
    return
  fi
  # A sway that died leaves its sockets in the user's runtime folder.
  if [[ -n $COMPOSITOR_EXIT && -n $SWAY_SOCK ]]; then
    local display=$SWAY_DISPLAY
    [[ $display == /* ]] || display=$XDG_RUNTIME_DIR/$display
    rm -f "$SWAY_SOCK" "$display" "$display.lock"
  fi
  if [[ -n $SWAY_SOCK && -e $SWAY_SOCK ]]; then
    echo "sway: its socket $SWAY_SOCK is still there" >&2
  elif [[ -n $COMPOSITOR_EXIT ]]; then
    echo "sway: gone before the run's end; its sockets removed, the run's Marley, pointer and keyboard stopped"
  elif [[ -n $SWAY_DIR ]]; then
    echo "sway: stopped, with the run's Marley, pointer and keyboard"
  fi
}

# The pid of the run's Marley, while its window is open.
marley_pid() {
  if [[ $COMPOSITOR == sway ]]; then
    sway_marley_pid
  else
    marley_window pid
  fi
}

# Starts Marley on the run's profile with the path the scenario named, its output after any
# earlier launch's in the run's log, and waits up to 90 seconds for its window.
launch_marley() {
  if [[ $COMPOSITOR == sway ]]; then
    hold_keyboard
    env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
      SWAYSOCK="$SWAY_SOCK" setsid -f "$marley" --user-data-dir "$E2E_PROFILE" ${OPEN:+"$OPEN"} \
      >>"$shots/$name.log" 2>&1 </dev/null
  else
    setsid -f "$marley" --user-data-dir "$E2E_PROFILE" ${OPEN:+"$OPEN"} >>"$shots/$name.log" 2>&1 </dev/null
  fi
  for _ in $(seq 90); do
    if [[ -n $(window) ]]; then
      if [[ $COMPOSITOR == sway ]]; then
        MARLEY_PID=$(sway_marley_pid || true)
      fi
      return 0
    fi
    sleep 1
  done
  echo "no Marley window within 90 seconds; see $shots/$name.log" >&2
  return 1
}

# Quits Marley through its command palette, as a user does, so it saves what it saves at a
# quit, and waits up to 30 seconds for the process to end.
quit_marley() {
  local pid
  pid=$(marley_pid)
  if [[ -z $pid ]]; then
    echo "quit_marley: no Marley window" >&2
    return 1
  fi
  press "CTRL SHIFT" p
  sleep 1
  type_text "zed: quit"
  sleep 1
  press "" Return
  for _ in $(seq 30); do
    kill -0 "$pid" 2>/dev/null || return 0
    sleep 1
  done
  echo "quit_marley: Marley still runs 30 seconds after the quit" >&2
  return 1
}

# One command to the pointer helper; fails unless it answers ok.
pointer() {
  local reply
  if [[ -z ${SEAT_POINTER[1]:-} ]]; then
    echo "pointer: the helper is gone; see $shots/$name.sway.log" >&2
    return 1
  fi
  echo "$*" >&"${SEAT_POINTER[1]}"
  if ! IFS= read -r -t 5 reply <&"${SEAT_POINTER[0]}"; then
    echo "pointer: no answer to '$*'" >&2
    return 1
  fi
  if [[ $reply != ok ]]; then
    echo "pointer: '$*' answered $reply" >&2
    return 1
  fi
}

needs_sway() {
  if [[ $COMPOSITOR != sway ]]; then
    echo "$1: the pointer needs COMPOSITOR=sway (Hyprland cannot click one window)" >&2
    return 1
  fi
}

wtype_modifier() {
  case $1 in
    CTRL) echo ctrl ;;
    SHIFT) echo shift ;;
    ALT) echo alt ;;
    SUPER) echo logo ;;
    *)
      echo "press: no modifier $1" >&2
      return 1
      ;;
  esac
}

# --- Steps ---------------------------------------------------------------------------------

settle() { sleep "$1"; }

# `holds <file> <text>...`: whether the file holds each text, as a fixed string (#517).
holds() {
  local file=$1 text
  shift
  for text in "$@"; do
    grep -qF -- "$text" "$file" || return 1
  done
}

# `expect <what> <command...>`: runs the command, prints `check <what>: pass` when it succeeds and
# `check <what>: FAIL` when it does not, and then returns 1, which ends the run (#517).
expect() {
  local what=$1
  shift
  if "$@"; then
    echo "check $what: pass"
  else
    echo "check $what: FAIL"
    return 1
  fi
}

# One key, down then up. Under Hyprland Omarchy's bindings send the halves apart because a
# whole `send_shortcut` can leave a key stuck.
press() {
  local mods=$1 key=$2 state address modifier
  local -a mod_names args=() releases=()
  read -ra mod_names <<<"$mods"
  if [[ $COMPOSITOR == sway ]]; then
    for modifier in "${mod_names[@]}"; do
      modifier=$(wtype_modifier "$modifier")
      args+=(-M "$modifier")
      releases+=(-m "$modifier")
    done
    WAYLAND_DISPLAY=$SWAY_DISPLAY wtype "${args[@]}" -k "$key" "${releases[@]}"
    sleep 0.05
    return
  fi
  address=$(marley_window address)
  [[ -n $address ]] || { echo "press: no Marley window" >&2; return 1; }
  for state in down up; do
    hyprctl eval "hl.dispatch(hl.dsp.send_key_state({ mods = \"$mods\", key = \"$key\", state = \"$state\", window = \"address:$address\" }))" >/dev/null
    sleep 0.05
  done
}

# The unshifted key of each shifted character on a US layout, and the xkb name of each other
# character that is not a letter or a digit.
declare -A SHIFTED=(
  ['!']=1 ['@']=2 ['#']=3 ['$']=4 ['%']=5 ['^']=6 ['&']=7 ['*']=8 ['(']=9 [')']=0
  ['_']=minus ['+']=equal ['{']=bracketleft ['}']=bracketright ['|']=backslash
  [':']=semicolon ['"']=apostrophe ['<']=comma ['>']=period ['?']=slash ['~']=grave
)
declare -A NAMED=(
  [' ']=space ['-']=minus ['=']=equal ['[']=bracketleft [']']=bracketright
  ["\\"]=backslash [';']=semicolon ["'"]=apostrophe [',']=comma ['.']=period ['/']=slash
  ['`']=grave
)

# Types text. Under sway `wtype` types any character; under Hyprland each character is a key
# of the US layout.
type_text() {
  local text=$1 i character
  if [[ $COMPOSITOR == sway ]]; then
    WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 -- "$text"
    return
  fi
  for ((i = 0; i < ${#text}; i++)); do
    character=${text:i:1}
    if [[ $character == [[:lower:][:digit:]] ]]; then
      press "" "$character"
    elif [[ $character == [[:upper:]] ]]; then
      press SHIFT "${character,}"
    elif [[ -n ${NAMED[$character]:-} ]]; then
      press "" "${NAMED[$character]}"
    elif [[ -n ${SHIFTED[$character]:-} ]]; then
      press SHIFT "${SHIFTED[$character]}"
    else
      echo "type_text: no key for '$character'" >&2
      return 1
    fi
  done
}

# Presses keys one after another, with no modifiers. Under sway one `wtype` sends them all:
# each `wtype` brings a keymap of its own, and gpui drops a compose sequence when the keymap
# changes, so a sequence (Multi_key, then its keys) composes only when sent together.
press_keys() {
  local key
  local -a args=()
  if [[ $COMPOSITOR == sway ]]; then
    for key in "$@"; do
      args+=(-k "$key")
    done
    WAYLAND_DISPLAY=$SWAY_DISPLAY wtype "${args[@]}"
    sleep 0.05
    return
  fi
  for key in "$@"; do
    press "" "$key"
  done
}

shot() {
  if [[ $COMPOSITOR == sway ]]; then
    WAYLAND_DISPLAY=$SWAY_DISPLAY grim "$shots/$1.png"
  else
    grim -T "$(marley_window stableId)" "$shots/$1.png"
  fi
  echo "$shots/$1.png"
}

# The path of a file a scenario keeps beside its shots: #492's frame an agent got, say.
shot_file() {
  echo "$shots/$1"
}

pointer_to() {
  needs_sway pointer_to
  pointer move "$1" "$2"
}

pointer_down() {
  needs_sway pointer_down
  pointer down "${1:-left}"
}

pointer_up() {
  needs_sway pointer_up
  pointer up "${1:-left}"
}

click() {
  needs_sway click
  pointer move "$1" "$2"
  sleep 0.05
  pointer down "${3:-left}"
  sleep 0.05
  pointer up "${3:-left}"
  sleep 0.1
}

# `click_with <mods> <x> <y>`: a left click while the modifiers named (`CTRL`, `"SHIFT CTRL"`)
# are held. One `wtype` presses them, holds them through the click and lets them go, since each
# `wtype` is a keyboard of its own and its keys go with it (#503).
click_with() {
  local mods=$1 x=$2 y=$3 modifier holder
  local -a mod_names args=() releases=()
  needs_sway click_with
  read -ra mod_names <<<"$mods"
  for modifier in "${mod_names[@]}"; do
    modifier=$(wtype_modifier "$modifier")
    args+=(-M "$modifier")
    releases+=(-m "$modifier")
  done
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype "${args[@]}" -s 1500 "${releases[@]}" &
  holder=$!
  sleep 0.4
  click "$x" "$y"
  wait "$holder"
}

scroll() {
  local steps=$1 step=1 i
  needs_sway scroll
  if ((steps < 0)); then
    step=-1
    steps=$((-steps))
  fi
  for ((i = 0; i < steps; i++)); do
    pointer scroll "$step"
    sleep 0.03
  done
}

# `profile_setting <key.path> <json>`: sets a key of the run's copy of the settings, a dotted path
# from the top, to a JSON value (#635). For a scenario's `setup`, before Marley starts. The
# scenario is sourced before this file's helpers, so a scenario's own function of the same name
# would be replaced: a scenario's helpers keep other names.
profile_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTING'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
*parents, last = sys.argv[2].split(".")
node = settings
for key in parents:
    node = node.setdefault(key, {})
node[last] = json.loads(sys.argv[3])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTING
}

# Writes the variables `terminal_env` collected into the copy's settings, as a `terminal.env`
# block that opens the file. A copy with a `terminal` block of its own is refused: its later key
# would win.
write_terminal_env() {
  [[ ${#TERMINAL_ENV[@]} -gt 0 ]] || return 0
  local settings=$E2E_PROFILE/config/settings.json
  if grep -q '"terminal"' "$settings"; then
    echo "terminal_env: the settings have a terminal block of their own" >&2
    return 1
  fi
  python3 - "$settings" "${TERMINAL_ENV[@]}" <<'PY'
import json, sys
path, pairs = sys.argv[1], sys.argv[2:]
env = dict(zip(pairs[0::2], pairs[1::2]))
lines = open(path).read().split("\n")
at = next(i for i, line in enumerate(lines) if line.startswith("{"))
lines.insert(at + 1, '  "terminal": %s,' % json.dumps({"env": env}))
open(path, "w").write("\n".join(lines))
PY
}

# --- The run -------------------------------------------------------------------------------

if [[ -z $MARLEY_BIN && ! -x $marley ]]; then
  echo "no $marley: run \`just build\` first" >&2
  exit 1
fi
hyprland_signature
if [[ $COMPOSITOR == hyprland ]]; then
  export WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-wayland-1}
  # The dev channel skips the single-instance check, so a second Marley would start beside it,
  # and keys meant for the run's window could reach the user's.
  if [[ -n $(marley_window pid) ]]; then
    echo "a Marley window is open; close it first, or run the scenario under COMPOSITOR=sway" >&2
    exit 1
  fi
fi

mkdir -p "$shots"
# The profile goes under the runtime directory rather than SHOT_DIR: Marley's instance socket
# lives in it, and a Unix socket's path must stay under 108 bytes (#513).
mkdir -p "$XDG_RUNTIME_DIR/marley-e2e"
E2E_PROFILE=$(mktemp -d "$XDG_RUNTIME_DIR/marley-e2e/profile.XXXXXX")
E2E_WORK=$(mktemp -d "$shots/work.XXXXXX")
export E2E_PROFILE E2E_WORK
mkdir -p "$E2E_PROFILE/config"
cp "$config/settings.json" "$E2E_PROFILE/config/"
# A run for a user who turned Rusty on would start their own rusty-mcp on their data, or reach their
# running Rusty service, and Zed starts each enabled context server when a project opens; one who
# turned Voice on would start their voxtype and draw a microphone on every agent bar. Both are off
# by default (#642, #643); the copy turns Rusty off and points its service at a port nothing
# listens on, takes out an old `marley.rusty_tools`, and turns Voice off, for a user who turned
# them on (#633, #642, #643). A scenario that wants either turns it back on with a stand-in: Rusty
# through `MARLEY_RUSTY_MCP`, which names no file until the scenario names its own. The copy also
# allows Claude Code's prompt tags on any version (#648): no scenario's stand-in `claude` answers
# `--version`, and the user's own Claude Code updates itself, so without it a scenario's prompts
# would read differently from one install to the next.
python3 - "$E2E_PROFILE/config/settings.json" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
marley = settings.setdefault("marley", {})
marley.pop("rusty_tools", None)
marley["rusty"] = {"enabled": False, "service_url": "http://127.0.0.1:9/mcp"}
marley.setdefault("voice", {})["enabled"] = False
marley.setdefault("allow_untested_versions", {})["claude_prompt_tags"] = True
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
export MARLEY_RUSTY_MCP=$E2E_WORK/no-rusty-mcp
# Marley reads the version of the codex it would run (#648); a run reads no codex of the user's
# unless its scenario names a stand-in.
export MARLEY_CODEX=$E2E_WORK/no-codex
cp -r "$data/db" "$E2E_PROFILE/db"
if [[ -d $data/threads ]]; then
  cp -r "$data/threads" "$E2E_PROFILE/threads"
fi

# Stops the user units of the run's Browser tabs: each Chromium runs in a unit named by its profile
# that outlives Marley, and would write into the run's profile after the cleanup removed it (#588).
stop_browser_units() {
  local data profile
  [[ -d $E2E_PROFILE/browser ]] || return 0
  data=$(realpath "$E2E_PROFILE")
  for profile in "$data"/browser/projects/*/profile "$data/browser/profile"; do
    [[ -e $profile ]] || continue
    systemctl --user stop "marley-browser-$(printf '%s' "$profile" | sha256sum | cut -c1-12)" \
      2>/dev/null || true
  done
}

# `$1`, an exit status, with the signal's name when a signal ended the process.
exit_status() {
  if (($1 > 128)); then
    echo "$1 (SIG$(kill -l $(($1 - 128)) 2>/dev/null || echo "$(($1 - 128))"))"
  else
    echo "$1"
  fi
}

cleanup() {
  local pid
  if declare -F teardown >/dev/null; then
    teardown || echo "teardown failed" >&2
  fi
  if [[ $COMPOSITOR == sway ]]; then
    sway_stop
  else
    pid=$(marley_window pid)
    if [[ -n $pid ]]; then
      kill -TERM "$pid" 2>/dev/null || true
    fi
    sleep 1
    # Drops the rule and restores focus_on_activate.
    hyprctl reload >/dev/null
  fi
  stop_browser_units
  # Marley writes its log into the profile, which goes next.
  if [[ -f $E2E_PROFILE/logs/Marley.log ]]; then
    cp "$E2E_PROFILE/logs/Marley.log" "$shots/$name.marley.log"
  fi
  rm -rf "$E2E_PROFILE" "$E2E_WORK" ${SWAY_DIR:+"$SWAY_DIR"}
  # Shots after the compositor went are of nothing, so its exit fails the run (#588).
  if [[ -n $COMPOSITOR_EXIT ]]; then
    echo "sway: the compositor exited during the run with status $(exit_status "$COMPOSITOR_EXIT"), so the run failed; see $shots/$name.sway.log" >&2
    exit 1
  fi
}
trap cleanup EXIT
# A signal ends the run through its EXIT trap, so a run cut short still stops what it started.
trap 'exit 130' INT TERM HUP

if [[ $COMPOSITOR == hyprland ]]; then
  hyprctl eval "hl.window_rule({ match = { class = \"$class\" }, workspace = \"9 silent\", render_unfocused = true })" >/dev/null
  hyprctl eval 'hl.config({ misc = { focus_on_activate = false } })' >/dev/null
fi

if declare -F setup >/dev/null; then
  setup
fi
# Without a folder named, Marley restores the copied profile's last session: the user's own
# projects, and Agent Panel threads that start their agent (#588).
if [[ -z $OPEN ]]; then
  echo "no folder to open: the scenario's setup names one with open_path" >&2
  exit 1
fi
if [[ -n $MARLEY_BIN ]]; then
  marley=$(realpath -m "$MARLEY_BIN")
  if [[ ! -x $marley ]]; then
    echo "no $marley, the binary the scenario names" >&2
    exit 1
  fi
fi
# The binary, the window class and the run's log, which gets Marley's output, for a scenario
# that starts a Marley of its own or reads what the run's Marley printed (#513).
export E2E_MARLEY=$marley E2E_CLASS=$class E2E_LOG=$shots/$name.log
write_terminal_env

if [[ $COMPOSITOR == sway ]]; then
  hyprland_before=$(hyprland_marley_windows)
  sway_start
  window() { sway_marley_pid; }
else
  before=$(user_focus)
  window() { marley_window stableId; }
fi
: >"$shots/$name.log"
launch_marley
steps
if [[ $COMPOSITOR == sway ]]; then
  echo "hyprland: $hyprland_before Marley windows before the run, $(hyprland_marley_windows) after; the run added no rule and did not reload it"
else
  after=$(user_focus)
  if [[ $before == "$after" ]]; then
    echo "focus: the user's window and workspace are as they were ($after)"
  else
    echo "focus: moved from $before to $after, by the run or by the user"
  fi
fi
