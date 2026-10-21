# shellcheck shell=bash
# #839's visual check: the pickers. A repository with a few files; stand-in `claude` and `codex`
# first on Marley's PATH, so the New Agent picker lists them.
# - The palette after `marley: open threads` ran from it: RECENT with it, then MARLEY, then EDITOR
#   AND WORKSPACE (`839-01-palette`, REQ-001).
# - "open" typed: no headings (`839-02-typed`, REQ-001).
# - `marley: new agent`: each agent a card with its line and place (`839-03-agents`, REQ-002).
# - The file finder, "lib" typed: lib.rs selected, its preview at the right (`839-04-files`,
#   REQ-003).
# - Marley Day, the palette (`839-05-day`, REQ-001).
compositor sway

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name
  mkdir -p "$home" "$bin" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in claude codex; do
    printf '#!/bin/sh\nexec sleep 600\n' >"$bin/$name"
    chmod +x "$bin/$name"
  done
  export PATH=$bin:$PATH
  cat >"$repo/src/lib.rs" <<'RS'
/// The rail's order of projects.
pub struct Rail {
    pub order: Vec<String>,
}

impl Rail {
    pub fn first(&self) -> Option<&String> {
        self.order.first()
    }
}
RS
  printf 'fn main() {}\n' >"$repo/src/main.rs"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== a command run from the palette, then the palette again"
  press "CTRL SHIFT" p
  settle 3
  type_text "marley: open threads"
  settle 3
  press "" Return
  settle 3
  press "CTRL SHIFT" p
  settle 3
  shot 839-01-palette

  echo "== typed"
  type_text "open"
  settle 5
  shot 839-02-typed
  press "" Escape
  settle 1

  echo "== the New Agent picker"
  press "CTRL SHIFT" p
  settle 2
  type_text "marley: new agent"
  settle 3
  press "" Return
  settle 3
  shot 839-03-agents
  press "" Escape
  settle 1

  echo "== the file finder"
  press "CTRL" p
  settle 3
  type_text "lib"
  settle 3
  shot 839-04-files
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 3
  press "CTRL SHIFT" p
  settle 3
  shot 839-05-day
}
