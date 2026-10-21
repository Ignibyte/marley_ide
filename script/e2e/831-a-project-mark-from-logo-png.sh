# shellcheck shell=bash
# #831's visual check: a project's mark from its logo.png. Two projects in one window: `logo`,
# whose top folder holds a 64 px orange logo.png with an "L", opened first, and `plain`, with no
# image at all, handed over second.
# - The rail in Marley Night: `logo`'s header with its image, `plain`'s with a "P" on a tint
#   (`831-01-rail-night`, REQ-001, REQ-002); `logo` shown, the title bar's mark before its name
#   (`831-02-title`, REQ-005).
# - The rail shrunk: the image tile and the letter tile (`831-03-strip`, REQ-003).
# - Home from the status bar: both marks in its recent projects (`831-04-home`, REQ-004).
# - logo.png replaced by a green one while Marley runs: the header and the title bar follow
#   (`831-05-replaced`, REQ-006).
# - "Use a Letter" from `logo`'s header menu: its header shows "L" on a tint, and
#   `marley.project_logos` printed from the profile (`831-06-letter`, REQ-007).
# - Marley Day: both marks (`831-07-day`, REQ-002).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the shots: `logo`'s header, the status bar's Home button, and its menu's Use a Letter.
LOGO_HEADER_X=${LOGO_HEADER_X:-70}
LOGO_HEADER_Y=${LOGO_HEADER_Y:-240}
HOME_X=${HOME_X:-1422}
HOME_Y=${HOME_Y:-985}
USE_A_LETTER_X=${USE_A_LETTER_X:-118}
USE_A_LETTER_Y=${USE_A_LETTER_Y:-513}

setup() {
  local home=$E2E_WORK/home name
  mkdir -p "$home"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in logo plain; do
    git init -q -b main "$E2E_WORK/$name"
  done
  logo_png '#ff8c42'
  # The box's own tmux sessions would add rows under Default, and move every point below them.
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/logo"
}

# Draws `logo/logo.png`: a 64 px square of the color given with a white "L".
logo_png() {
  magick -size 64x64 "xc:$1" -fill white -gravity center -pointsize 44 -annotate 0 L \
    "PNG:$E2E_WORK/logo/logo.png.new"
  mv -f "$E2E_WORK/logo/logo.png.new" "$E2E_WORK/logo/logo.png"
}

# Hands `path` to the running Marley, as a second launch does (#513).
hand_over() {
  timeout 20 env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >>"$E2E_WORK/second.log" 2>&1 </dev/null || true
}

# `marley.project_logos` in the run's copy of the settings, as Marley wrote it.
logos_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, pathlib, re, sys

text = pathlib.Path(sys.argv[1]).read_text()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
print(json.loads(text).get("marley", {}).get("project_logos"))
PY
}

steps() {
  settle 12
  # Trusts logo.
  press "" Return
  settle 3

  echo "== plain, handed over; logo shown again"
  hand_over "$E2E_WORK/plain"
  settle 5
  press "" Return
  settle 3
  hand_over "$E2E_WORK/logo"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 831-01-rail-night
  shot 831-02-title

  echo "== shrunk"
  profile_setting marley.rail_shrunk true
  settle 3
  shot 831-03-strip
  profile_setting marley.rail_shrunk false
  settle 3

  echo "== Home"
  click "$HOME_X" "$HOME_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 831-04-home

  echo "== logo.png replaced"
  logo_png '#3f9d2a'
  settle 4
  shot 831-05-replaced

  echo "== Use a Letter"
  click "$LOGO_HEADER_X" "$LOGO_HEADER_Y" right
  settle 2
  shot 831-06-menu
  click "$USE_A_LETTER_X" "$USE_A_LETTER_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 831-06-letter
  echo "project_logos: $(logos_setting)"

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 831-07-day
}
