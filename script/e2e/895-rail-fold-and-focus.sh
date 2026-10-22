# shellcheck shell=bash
# #895's visual check: the rail's fold in motion and the focus ring, as #830 and the theme already
# draw them. A scratch repository.
# - `marley: toggle rail strip`, a burst of shots 60 ms apart: the rail's width easing from open to
#   the strip over 180 ms (`895-01-fold-a` to `-e`, REQ-001).
# - The Settings window's search field focused: its border in the coat's orange
#   (`895-02-focus`, REQ-002).
compositor sway

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the rail folds"
  pointer_to 900 600
  press "CTRL SHIFT" p
  settle 2
  type_text "marley: toggle rail strip"
  settle 3
  press "" Return
  local index
  for index in a b c d e; do
    settle 0.06
    shot "895-01-fold-$index"
  done
  settle 2

  echo "== a focused field"
  press CTRL comma
  settle 4
  shot 895-02-focus
}
