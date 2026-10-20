# shellcheck shell=bash
# #828's visual check: Marley's own themes. The profile copy's `theme` key is taken out, so
# `default.json` decides; the dev box's appearance is dark, so Marley starts in Marley Night.
# - A terminal with a passing and a failing command, the rail and the bottom row
#   (`828-01-night-terminal`, REQ-001, REQ-003); an editor with Rust and a git change
#   (`828-02-night-editor`, REQ-003); the component preview (`828-03-night-components`, REQ-004).
# - The copy's `theme` set to "Marley Day" while Marley runs (Zed reloads its settings file): the
#   terminal and the component preview again (`828-04-day-terminal`, `828-05-day-components`,
#   REQ-002, REQ-004).
# - The theme selector filtered on "marley" (`828-06-selector`, REQ-005).
compositor sway

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$repo/src/main.rs" <<'RS'
/// The boards Marley draws.
struct Board {
    title: String,
    width: u32,
}

fn main() {
    let names = ["rail", "thread", "terminal"];
    for name in names {
        println!("{name}");
    }
}
RS
  git -C "$repo" init -q -b main
  git -C "$repo" add -A
  git -C "$repo" -c user.name=t -c user.email=t@t commit -qm init
  printf '\n// the theme is the cat\n' >>"$repo/src/main.rs"
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
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 15 "git status --short" -k Return
  settle 3
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 15 "false" -k Return
  settle 3
  shot 828-01-night-terminal

  press "CTRL" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "main.rs"
  settle 2
  press "" Return
  settle 4
  shot 828-02-night-editor

  palette "workspace: open component preview" 6
  shot 828-03-night-components

  echo "== Marley Day, by name"
  set_theme "Marley Day"
  settle 4
  shot 828-05-day-components
  palette "workspace: new terminal" 5
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 15 "git status --short" -k Return
  settle 3
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 15 "false" -k Return
  settle 3
  shot 828-04-day-terminal

  palette "theme selector: toggle" 2
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "marley"
  settle 2
  shot 828-06-selector
  press "" Escape
}
