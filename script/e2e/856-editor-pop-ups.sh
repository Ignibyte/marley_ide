# shellcheck shell=bash
# #856's visual check: the editor's pop-ups in the EditorPopups board's look. A scratch crate
# rust-analyzer reads, with a `Rail` struct and `rail.wid` left unfinished at line 17.
# - Ctrl+Space after `rail.wid`: each completion's kind letter in its syntax colour, the detail at
#   the right (`856-01-completions`, REQ-001).
# - `editor: hover` on `wid`: the error under the pointer (`856-02-hover`, REQ-003).
# - Ctrl+. on `Rail`: "Code actions" and its key over the actions (`856-03-actions`, REQ-002).
# - Marley Day, completions again (`856-04-day`, REQ-001).
compositor sway
setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '[package]\nname = "rail"\nversion = "0.1.0"\nedition = "2021"\n' >"$repo/Cargo.toml"
  cat >"$repo/src/main.rs" <<'RS'
/// How wide the rail draws.
pub struct Rail {
    width: u32,
    shrunk: bool,
}

impl Rail {
    /// The rail's width, shrunk or not.
    pub fn width_for(&self, shrunk: bool) -> u32 {
        if shrunk { 48 } else { self.width }
    }
}

fn main() {
    let rail = Rail { width: 260, shrunk: false };
    let rows = rail.width_for(rail.shrunk);
    rail.wid
}
RS
  git init -q -b main "$repo"
  open_path "$repo"
}
palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 2
  press "" Return
  settle 3
}
steps() {
  settle 15
  # Trusts the scratch crate.
  press "" Return
  settle 4
  press CTRL p
  settle 2
  type_text "main.rs"
  settle 2
  press "" Return
  # rust-analyzer reads the crate and std.
  settle 40
  completions 856-01-completions

  echo "== hover on the error"
  goto 17:12
  palette "editor: hover"
  settle 3
  shot 856-02-hover
  press "" Escape

  echo "== code actions on the struct"
  goto 2:12
  press CTRL period
  settle 4
  shot 856-03-actions
  press "" Escape

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  completions 856-04-day
}

# Moves the caret to `$1`, line:column.
goto() {
  palette "go to line"
  type_text "$1"
  press "" Return
  settle 2
}

# Opens completions after `rail.wid` and shoots them as `$1`.
completions() {
  echo "== completions"
  goto 17:13
  press CTRL space
  settle 4
  shot "$1"
  press "" Escape
  settle 1
}
