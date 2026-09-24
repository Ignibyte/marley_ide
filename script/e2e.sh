#!/usr/bin/env bash
# e2e.sh <scenario>: a ticket's e2e visualization test on a Hyprland dev box (CONSTITUTION §7).
#
# Runs the debug `marley` on a copy of the user's Marley profile, on hidden workspace 9, and runs
# the scenario against it. Keys go to Marley's window only (Hyprland's `send_key_state` with the
# window named), so the user's focus stays where it is, and each shot is the window by its
# toplevel (`grim -T` with Hyprland's `stableId`), so nothing shows on the screen in use
# (L-claude-467-capture-one-window-by-its-toplevel-001). No mouse: a click would move the user's
# pointer. The profile copy and the scenario's scratch folder are removed when the run ends.
#
# The scenario, `script/e2e/<ticket>-<slug>.sh`, is sourced. It defines `steps`, run once the
# window maps, and may define `setup`, run before the launch with E2E_PROFILE (the profile copy)
# and E2E_WORK (a scratch folder) set: it builds its fixtures there, may name a path for Marley
# to open with `open_path <path>`, may set variables for Marley's terminals with
# `terminal_env <name> <value>` (a HOME whose .bashrc is the scenario's, say, so no test depends
# on the user's shell), and may prepend to PATH (fakes that Marley and its terminals find first).
# OPEN in the environment names the path when the scenario names none.
#
# Steps: `settle <seconds>`; `press <mods> <key>`, with mods as Hyprland names them ("" for none,
# "CTRL SHIFT" for two) and the key by its xkb name (`Return`, `Escape`, `g`); `type_text <text>`;
# and `shot <name>`, which writes SHOT_DIR/<name>.png and prints its path.
#
#   SHOT_DIR where the PNGs and Marley's log land ($TMPDIR/marley-shots); never the repository
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

export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-/run/user/$(id -u)}
export WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-wayland-1}
if [[ -z ${HYPRLAND_INSTANCE_SIGNATURE:-} ]]; then
  HYPRLAND_INSTANCE_SIGNATURE=$(find "$XDG_RUNTIME_DIR/hypr" -mindepth 1 -maxdepth 1 \
    -printf '%T@ %f\n' | sort -rn | head -1 | cut -d' ' -f2)
  export HYPRLAND_INSTANCE_SIGNATURE
fi

# The first Marley window's `field` (`pid`, `stableId`, `address`), or nothing.
marley_window() {
  hyprctl clients -j | jq -r --arg class "$class" --arg field "$1" \
    '[.[] | select(.class == $class)][0][$field] // empty'
}

# The user's active window and workspace, to report whether the run moved them.
user_focus() {
  printf '%s %s' "$(hyprctl activewindow -j | jq -r '.address // "none"')" \
    "$(hyprctl activeworkspace -j | jq -r '.id')"
}

open_path() { OPEN=$1; }

TERMINAL_ENV=()
terminal_env() { TERMINAL_ENV+=("$1" "$2"); }

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

settle() { sleep "$1"; }

# One key, down then up: Omarchy's bindings send the halves apart because a whole
# `send_shortcut` can leave a key stuck.
press() {
  local mods=$1 key=$2 state address
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

type_text() {
  local text=$1 i character
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

shot() {
  grim -T "$(marley_window stableId)" "$shots/$1.png"
  echo "$shots/$1.png"
}

if [[ ! -x $marley ]]; then
  echo "no $marley: run \`just build\` first" >&2
  exit 1
fi
# The dev channel skips the single-instance check, so a second Marley would start beside it.
if [[ -n $(marley_window pid) ]]; then
  echo "a Marley window is open; close it first" >&2
  exit 1
fi

mkdir -p "$shots"
E2E_PROFILE=$(mktemp -d "$shots/profile.XXXXXX")
E2E_WORK=$(mktemp -d "$shots/work.XXXXXX")
export E2E_PROFILE E2E_WORK
mkdir -p "$E2E_PROFILE/config"
cp "$config/settings.json" "$E2E_PROFILE/config/"
cp -r "$data/db" "$E2E_PROFILE/db"
if [[ -d $data/threads ]]; then
  cp -r "$data/threads" "$E2E_PROFILE/threads"
fi

OPEN=${OPEN:-}
# shellcheck source=/dev/null
. "$scenario"
declare -F steps >/dev/null || { echo "$scenario defines no steps" >&2; exit 2; }

hyprctl eval "hl.window_rule({ match = { class = \"$class\" }, workspace = \"9 silent\", render_unfocused = true })" >/dev/null
hyprctl eval 'hl.config({ misc = { focus_on_activate = false } })' >/dev/null
cleanup() {
  local pid
  pid=$(marley_window pid)
  if [[ -n $pid ]]; then
    kill -TERM "$pid" 2>/dev/null || true
  fi
  sleep 1
  # Drops the rule and restores focus_on_activate.
  hyprctl reload >/dev/null
  rm -rf "$E2E_PROFILE" "$E2E_WORK"
}
trap cleanup EXIT

if declare -F setup >/dev/null; then
  setup
fi
write_terminal_env
name=$(basename "$scenario" .sh)
before=$(user_focus)
setsid -f "$marley" --user-data-dir "$E2E_PROFILE" ${OPEN:+"$OPEN"} >"$shots/$name.log" 2>&1 </dev/null
for _ in $(seq 90); do
  [[ -n $(marley_window stableId) ]] && break
  sleep 1
done
if [[ -z $(marley_window stableId) ]]; then
  echo "no Marley window within 90 seconds; see $shots/$name.log" >&2
  exit 1
fi
steps
after=$(user_focus)
if [[ $before == "$after" ]]; then
  echo "focus: the user's window and workspace are as they were ($after)"
else
  echo "focus: moved from $before to $after, by the run or by the user"
fi
