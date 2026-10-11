# shellcheck shell=bash
# #751's visual check: a restored terminal shows its earlier output, dimmed, above a new prompt. A
# scratch repository; a HOME whose `.bashrc` sets a plain prompt and puts the scenario's `bin`
# first, where `ask` prints three terminal queries (cursor position, device attributes, the
# background colour) and reads their answers away, so the live shell stays clean.
# - The first terminal: `seq 1 20000`, a red and a green line, `echo before`, `ask` (`before`).
# - A second terminal: `echo second`, then `less README.md`, still showing at the quit.
# - Each kept file is mode 600 in a folder of mode 700 (REQ-007).
# - A quit and a launch: the first terminal shows the lines and `before` dimmed above a bright
#   prompt, no block on them (`restored`, REQ-001, REQ-002, REQ-006); `echo clean` runs as typed
#   and makes a block (`clean`, REQ-003, REQ-006); scrolled to its top, its first number is past
#   15000 (`cap`, REQ-004). The second shows `second` and none of `less`'s screen (`fullscreen`,
#   REQ-005); closed, its file is gone (REQ-008).
# - `marley.restore_scrollback` false between a quit and a launch: an empty terminal, and no
#   file kept (`off`, REQ-009).
compositor sway

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  cat >"$bin/ask" <<'ASK'
#!/usr/bin/env bash
# Three queries a terminal answers, their answers read away within a second.
saved=$(stty -g)
stty -echo -icanon min 0 time 10
printf '\033[6n\033[c\033]11;?\007'
cat >/dev/null
stty "$saved"
echo asked
ASK
  chmod +x "$bin/ask"
  terminal_env HOME "$home"
  for line in $(seq 1 60); do printf 'README line %s\n' "$line"; done >"$repo/README.md"
  git init -q -b main "$repo"
  open_path "$repo"
}

history_dir() { echo "$E2E_PROFILE/terminal_history"; }

# The kept files, one name a line.
kept() { find "$(history_dir)" -name '*.log' -printf '%f\n' 2>/dev/null | sort; }

# Whether the folder is mode 700 and every kept file mode 600, with at least one file.
private_files() {
  local folder modes
  folder=$(stat -c %a "$(history_dir)")
  modes=$(find "$(history_dir)" -name '*.log' -printf '%m\n' | sort -u)
  echo "folder $folder, files ${modes//$'\n'/ }; $(kept | wc -l) kept"
  [[ $folder == 700 && $modes == 600 ]]
}

# Whether the second terminal's file is gone and the first one's kept.
second_gone() {
  local second
  second=$(cat "$E2E_WORK/second.id")
  kept
  [[ ! -e "$(history_dir)/$second.log" && $(kept | wc -l) -eq 1 ]]
}

no_files() {
  kept
  [[ -z $(kept) ]]
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

cue() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== the first terminal"
  cue "seq 1 20000" 4
  cue "printf '\\033[31mred line\\033[0m\\n\\033[32mgreen line\\033[0m\\n'"
  cue "echo before"
  cue "ask" 3
  shot 751-01-before
  echo "== the second terminal, in less at the quit"
  palette "workspace: new terminal" 4
  cue "echo second"
  cue "echo \$MARLEY_TERMINAL_ID > $E2E_WORK/second.id"
  cue "less README.md" 3
  expect "each kept file is private" private_files
  echo "== a quit and a launch"
  quit_marley
  launch_marley
  settle 16
  shot 751-05-fullscreen
  palette "pane: activate previous item"
  shot 751-02-restored
  cue "echo clean"
  shot 751-03-clean
  palette "terminal: scroll to top"
  shot 751-04-cap
  echo "== the second terminal closed"
  palette "pane: activate next item"
  press "CTRL SHIFT" w
  settle 3
  expect "the closed terminal's file is gone" second_gone
  echo "== the setting off"
  quit_marley
  profile_setting marley.restore_scrollback false
  launch_marley
  settle 16
  shot 751-06-off
  expect "no file is kept" no_files
}
