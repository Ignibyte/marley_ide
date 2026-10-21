# shellcheck shell=bash
# #829's visual check: Marley's mark, Chad's cat. A project is open, so the title bar shows its
# name beside the mark.
# - The title bar in Marley Night, the default (`829-01-title-night`, REQ-002).
# - The copy's `theme` set to "Marley Day" while Marley runs: the same title bar
#   (`829-02-title-day`, REQ-002).
# - `zed: about` from the command palette: the About window with the mark and the line about
#   Marley (`829-03-about`, REQ-003).
compositor sway

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# marley\n' >"$repo/README.md"
  git -C "$repo" init -q -b main
  git -C "$repo" add -A
  git -C "$repo" -c user.name=t -c user.email=t@t commit -qm init
  set_theme ''
  open_path "$repo"
}

# Sets the profile copy's `theme` to the name given, or takes the key out for an empty name.
set_theme() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*', lambda m: m.group(1) or "", text)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text or "{}")
settings.pop("theme", None)
settings.pop("experimental.theme_overrides", None)
settings.pop("theme_overrides", None)
if sys.argv[2]:
    settings["theme"] = sys.argv[2]
path.write_text(json.dumps(settings, indent=2))
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 900 -k Return
  settle "${2:-4}"
}

steps() {
  settle 14
  press "" Return
  settle 3

  echo "== Marley Night, the default"
  shot 829-01-title-night

  echo "== Marley Day, by name"
  set_theme "Marley Day"
  settle 4
  shot 829-02-title-day

  echo "== About"
  set_theme ''
  settle 4
  palette "zed: about" 4
  shot 829-03-about
  press "" Escape
}
