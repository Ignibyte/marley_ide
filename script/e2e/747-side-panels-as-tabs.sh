# shellcheck shell=bash
# #747's visual check: Files, Git and Outline open as tabs. A scratch git repo with a Rust file; the
# right dock starts open on the project panel (Zed's `starts_open`) and must not show it.
# - At start: no right dock, no Files tab (`start`, REQ-008's rule for a dock that opens without
#   the focus).
# - The status bar's Files button: a Files tab in the active pane, focused (`files`, REQ-001).
# - The Git button: a Git tab (`git`, REQ-001).
# - Ctrl+Shift+E: the Files tab forward, no second one (`key`, `again`, REQ-002, REQ-003).
# - The Outline button with lib.rs in front: an Outline tab listing lib.rs's symbols (`outline`,
#   REQ-001, REQ-006).
# - lib.rs open, then `pane: reveal in project panel`: the Files tab forward (`reveal`, REQ-004).
# - The Files tab dragged to the pane's right edge: Files in a pane of its own (`drag`, REQ-005).
# - `pane: split down` on the Git tab: Git moved into a new pane, no copy (`split-key`, REQ-005).
# - Ctrl+Alt+B (the Agent Panel in the right dock), then the Files button: the dock stays on the
#   Agent Panel (`agent-stays`, REQ-007).
# - `marley: use zed layout`: no panel tab left (`zed-layout`, REQ-011).
compositor sway

# The status bar's Files, Git and Outline buttons and the Files tab, as the first run's shots
# found them; `STOP_AT_START=1` ends the run at the first shot.
FILES_X=${FILES_X:-1558}
GIT_X=${GIT_X:-1532}
OUTLINE_X=${OUTLINE_X:-1482}
BAR_Y=${BAR_Y:-985}
FILES_TAB_X=${FILES_TAB_X:-560}
TAB_Y=${TAB_Y:-51}
DROP_X=${DROP_X:-1570}
DROP_Y=${DROP_Y:-500}
STOP_AT_START=${STOP_AT_START:-0}

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

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 4
  shot 747-00-start
  if [[ $STOP_AT_START == 1 ]]; then
    return
  fi

  echo "== the Files button"
  click "$FILES_X" "$BAR_Y"
  settle 2
  shot 747-01-files

  echo "== the Git button"
  click "$GIT_X" "$BAR_Y"
  settle 3
  shot 747-02-git

  echo "== Ctrl+Shift+E again"
  press "CTRL SHIFT" e
  settle 2
  shot 747-03-again

  echo "== lib.rs, then the Outline button"
  open_lib
  settle 3
  click "$OUTLINE_X" "$BAR_Y"
  settle 3
  shot 747-04-outline

  echo "== Reveal in Project Panel from lib.rs"
  open_lib
  settle 2
  palette "pane: reveal in project panel"
  settle 2
  shot 747-05-reveal

  echo "== the Files tab dragged to the right edge"
  local step
  pointer_to "$FILES_TAB_X" "$TAB_Y"
  settle 0.3
  pointer_down
  settle 0.3
  for step in 1 2 3 4 5 6 7 8; do
    pointer_to "$((FILES_TAB_X + (DROP_X - FILES_TAB_X) * step / 8))" \
      "$((TAB_Y + (DROP_Y - TAB_Y) * step / 8))"
    sleep 0.1
  done
  settle 0.5
  shot 747-06a-dragging
  pointer_up
  settle 2
  shot 747-06-drag

  echo "== pane: split down on the Git tab"
  click "$GIT_X" "$BAR_Y"
  settle 2
  palette "pane: split down"
  settle 2
  shot 747-07-split-key

  echo "== the Agent Panel open, then the Files button"
  press "CTRL ALT" b
  settle 3
  click "$FILES_X" "$BAR_Y"
  settle 2
  shot 747-08-agent-stays

  echo "== Zed's layout"
  palette "marley: use zed layout"
  settle 4
  shot 747-09-zed-layout
}
