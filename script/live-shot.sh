#!/usr/bin/env bash
# live-shot.sh <name>: the Test phase's live drive on a Hyprland dev box, with no input sent.
#
# Runs the debug `marley` on a copy of the user's Marley profile, on hidden workspace 9, and
# shoots its window by its toplevel (`grim -T` with Hyprland's `stableId`), so nothing shows on
# the screen in use (L-claude-467-capture-one-window-by-its-toplevel-001). The copy is removed
# when the run ends.
#
#   SEED     a script run with the copy's directory before the launch, to edit the copy
#   OPEN     a path to open, as `marley <path>` does (the profile's last session otherwise)
#   SETTLE   seconds to wait after the window maps (12)
#   INSPECT  a command run after the shot, while Marley still runs
#   SHOT_DIR where the PNG and the log land ($TMPDIR/marley-shots)
set -euo pipefail

name=${1:?usage: live-shot.sh <name>}
cd "$(dirname "$0")/.."
shots=${SHOT_DIR:-${TMPDIR:-/tmp}/marley-shots}
marley=${CARGO_TARGET_DIR:-target}/debug/marley
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

# The first Marley window's `field` (`pid`, `stableId`), or nothing.
marley_window() {
  hyprctl clients -j | jq -r --arg class "$class" --arg field "$1" \
    '[.[] | select(.class == $class)][0][$field] // empty'
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
profile=$(mktemp -d "$shots/profile.XXXXXX")
mkdir -p "$profile/config"
cp "$config/settings.json" "$profile/config/"
cp -r "$data/db" "$profile/db"
if [[ -d $data/threads ]]; then
  cp -r "$data/threads" "$profile/threads"
fi
if [[ -n ${SEED:-} ]]; then
  "$SEED" "$profile"
fi

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
  rm -rf "$profile"
}
trap cleanup EXIT

setsid -f "$marley" --user-data-dir "$profile" ${OPEN:+"$OPEN"} >"$shots/$name.log" 2>&1 </dev/null
for _ in $(seq 90); do
  [[ -n $(marley_window stableId) ]] && break
  sleep 1
done
if [[ -z $(marley_window stableId) ]]; then
  echo "no Marley window within 90 seconds; see $shots/$name.log" >&2
  exit 1
fi
sleep "${SETTLE:-12}"
grim -T "$(marley_window stableId)" "$shots/$name.png"
if [[ -n ${INSPECT:-} ]]; then
  bash -c "$INSPECT"
fi
echo "$shots/$name.png"
