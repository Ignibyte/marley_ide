# shellcheck shell=bash
# #773's visual check: panel tabs come back after a restart. A scratch git repo with a Rust file;
# `marley.rail_containers` on, which the runner's copy turns off (#669).
# - Files, Git, lib.rs, Outline and Containers open as tabs, then Files split into a pane on the
#   right (`before`).
# - Marley quits and starts again: the same tabs in the same panes (`restored`, REQ-001).
# - The Outline button: the restored Outline tab forward, listing lib.rs's symbols (`outline`,
#   REQ-002, REQ-003).
# - The Files button: the restored Files tab forward on the right, no second one, and the Outline
#   tab still listing lib.rs while Files has the focus (`again`, REQ-002, REQ-003).
# - A start with `marley.rail_containers` off: Files, Git and Outline back, no Containers tab, and
#   after the next quit no Containers row (`disabled`, REQ-004).
# - A start in Zed's layout: no panel tab (`zed-layout`, REQ-005).
compositor sway

# The status bar's Files, Git and Outline buttons, as #747's shots found them.
FILES_X=${FILES_X:-1558}
GIT_X=${GIT_X:-1532}
OUTLINE_X=${OUTLINE_X:-1482}
BAR_Y=${BAR_Y:-985}

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
  profile_setting marley.rail_containers true
  git -C "$repo" init -q
  git -C "$repo" add .
  git -C "$repo" -c user.name=e2e -c user.email=e2e@example.com commit -q -m start
  open_path "$repo"
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Opens src/lib.rs through the file finder.
open_lib() {
  palette "file finder: toggle"
  settle 1
  type_text "lib.rs"
  settle 2
  press "" Return
}

# The panel names saved for the scratch repo's panel tabs, one a line. The profile is a copy of the
# user's, so only this run's workspace is read.
saved_panels() {
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 -readonly "$db" "select t.panel from marley_panel_tabs t join workspaces w on w.workspace_id = t.workspace_id where w.paths like '%$E2E_WORK%' order by t.panel" 2>/dev/null || true
  done
}

# Whether no Containers row is saved, with Files' row still there.
no_containers_row() {
  local panels
  panels=$(saved_panels)
  echo "saved panel tabs: ${panels//$'\n'/, }"
  grep -qx "Project Panel" <<<"$panels" && ! grep -qx "MarleyContainersPanel" <<<"$panels"
}

# Quits and starts Marley again on the same project.
restart() {
  quit_marley
  launch_marley
  settle 15
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Files, Git, lib.rs, Outline and Containers as tabs; Files split to the right"
  click "$FILES_X" "$BAR_Y"
  settle 2
  click "$GIT_X" "$BAR_Y"
  settle 3
  open_lib
  settle 3
  click "$OUTLINE_X" "$BAR_Y"
  settle 3
  palette "marley: toggle containers"
  settle 3
  click "$FILES_X" "$BAR_Y"
  settle 2
  palette "pane: split right"
  settle 3
  shot 773-01-before
  echo "saved panel tabs before the quit: $(saved_panels | paste -sd, -)"

  echo "== Marley quits and starts again"
  restart
  shot 773-02-restored

  echo "== the Outline button"
  click "$OUTLINE_X" "$BAR_Y"
  settle 3
  shot 773-03-outline

  echo "== the Files button"
  click "$FILES_X" "$BAR_Y"
  settle 2
  shot 773-04-again

  echo "== a start with Containers turned off"
  quit_marley
  profile_setting marley.rail_containers false
  launch_marley
  settle 15
  shot 773-05-disabled
  quit_marley
  expect "the Containers tab's row was dropped" no_containers_row

  echo "== a start in Zed's layout"
  profile_setting marley.rail_containers true
  profile_setting marley.layout '"zed"'
  launch_marley
  settle 15
  shot 773-06-zed-layout
}
