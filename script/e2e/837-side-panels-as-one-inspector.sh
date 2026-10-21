# shellcheck shell=bash
# #837's visual check: Files, Changes and Outline in one Inspector column. A scratch git repo with a
# Rust file, committed and then changed; `marley.side_inspector` on.
# - lib.rs open, then `project panel: toggle focus`: the Inspector at the right of the editor, its
#   switcher on Files, the tree under it (`837-01-files`, REQ-001, REQ-002).
# - The switcher's Changes: the Git panel, src/lib.rs changed (`837-02-changes`, REQ-002).
# - The switcher's Outline: lib.rs's symbols (`837-03-outline`, REQ-002).
# - The setting off, then `project panel: toggle focus`: no Inspector, #747's Files tab
#   (`837-04-tabs`, REQ-003).
# - The setting on, Marley Day, Files again (`837-05-day`, REQ-001, REQ-002).
compositor sway

AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-700}
# The switcher's buttons, from the probe run's shots.
CHANGES_X=${CHANGES_X:-1356}
CHANGES_Y=${CHANGES_Y:-82}
OUTLINE_X=${OUTLINE_X:-1418}
OUTLINE_Y=${OUTLINE_Y:-82}
STOP_AT_FILES=${STOP_AT_FILES:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$repo/src/lib.rs" <<'RS'
pub struct Ledger {
    entries: Vec<u32>,
}

impl Ledger {
    pub fn total(&self) -> u32 {
        self.entries.iter().sum()
    }
}

pub fn open_ledger() -> Ledger {
    Ledger { entries: Vec::new() }
}
RS
  printf '# repo\n' >"$repo/README.md"
  git -C "$repo" init -q
  git -C "$repo" add .
  git -C "$repo" -c user.name=e2e -c user.email=e2e@example.com commit -q -m start
  printf '\n// changed\n' >>"$repo/src/lib.rs"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.side_inspector 'true'
  open_path "$repo"
}

# Runs `$1` from the command palette.
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

  echo "== lib.rs, then Files"
  palette "file finder: toggle" 3
  type_text "lib.rs"
  settle 3
  press "" Return
  settle 3
  palette "project panel: toggle focus" 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 837-01-files
  if [[ $STOP_AT_FILES == 1 ]]; then
    return
  fi

  echo "== Changes"
  click "$CHANGES_X" "$CHANGES_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 837-02-changes

  echo "== Outline"
  click "$OUTLINE_X" "$OUTLINE_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 837-03-outline

  echo "== the setting off, then Files"
  profile_setting marley.side_inspector 'false'
  settle 3
  palette "project panel: toggle focus" 3
  shot 837-04-tabs

  echo "== Marley Day, the Inspector on Files"
  profile_setting marley.side_inspector 'true'
  profile_setting theme '"Marley Day"'
  settle 3
  palette "project panel: toggle focus" 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 837-05-day
}
