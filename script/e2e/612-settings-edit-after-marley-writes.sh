# shellcheck shell=bash
# #612's e2e test: an edit to settings.json after Marley has written the file reloads. Marley
# opens `repo`; a round trip through Zed's layout and back has Marley write the run's
# settings.json, as Zed's writers do, by replacing the file. Then `ui_font_size` is edited three
# times outside Marley: to 22 in place (`edited`, REQ-001), to 12 in place (`edited-again`,
# REQ-001), and to 18 by writing a new file and renaming it over the old, as many editors save
# (`replaced`, REQ-002). Each shot shows the rail's and the title bar's text at the new size.
compositor sway

SETTINGS=""

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  SETTINGS=$E2E_PROFILE/config/settings.json
  open_path "$E2E_WORK/repo"
}

# Sets the run's top-level `ui_font_size` to `$1`, in place, or by a new file renamed over the
# old when `$2` is `replace`.
set_font_size() {
  python3 - "$SETTINGS" "$1" "${2:-}" <<'PY'
import os, re, sys
path, size, how = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(path).read()
text, count = re.subn(r'^(\s*)"ui_font_size"\s*:\s*[0-9.]+', r'\g<1>"ui_font_size": ' + size, text, count=1, flags=re.M)
if count == 0:
    at = text.index("{") + 1
    text = text[:at] + '\n  "ui_font_size": ' + size + ',' + text[at:]
if how == "replace":
    with open(path + ".new", "w") as new:
        new.write(text)
    os.replace(path + ".new", path)
else:
    with open(path, "r+") as file:
        file.seek(0)
        file.write(text)
        file.truncate()
print(f"ui_font_size {size} ({how or 'in place'}), inode {os.stat(path).st_ino}")
PY
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  echo "settings.json before: inode $(stat -c %i "$SETTINGS")"
  shot before

  echo "== a round trip through Zed's layout: Marley writes settings.json"
  palette "marley: use zed layout"
  settle 3
  palette "marley: use marley layout"
  settle 3
  echo "settings.json after Marley's writes: inode $(stat -c %i "$SETTINGS")"

  echo "== ui_font_size edited outside Marley"
  set_font_size 22
  settle 3
  pointer_to 700 600
  settle 1
  shot edited
  set_font_size 12
  settle 3
  shot edited-again
  set_font_size 18 replace
  settle 3
  shot replaced
}
