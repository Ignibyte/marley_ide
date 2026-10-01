# shellcheck shell=bash
# #625's e2e test: completions in the shell's prompt editor. `repo` holds `Cargo.toml` and `src/`;
# the shell's history file holds `cargo test --workspace`. Ctrl+G at the prompt opens the editor;
# `cat Car` and Tab list `Cargo.toml` (`paths`, REQ-001); Enter takes it, the line not run
# (`taken`, REQ-002); the editor's text made `cargo t`, Tab lists the history's command
# (`history`, REQ-003).
compositor sway

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  printf 'cargo test --workspace\n' >"$home/.bash_history"
  terminal_env HOME "$home"
  printf '[package]\nname = "repo"\n' >"$E2E_WORK/repo/Cargo.toml"
  printf 'fn main() {}\n' >"$E2E_WORK/repo/src/main.rs"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 1
  press "CTRL" g
  settle 1.5

  echo "== a path"
  type_text "cat Car"
  settle 0.5
  press "" Tab
  settle 1.5
  shot paths
  press "" Return
  settle 1
  shot taken

  echo "== a command from the history"
  press "CTRL" a
  type_text "cargo t"
  settle 0.5
  press "" Tab
  settle 1.5
  shot history
}
