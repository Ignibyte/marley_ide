# shellcheck shell=bash
# #634's visual check of the focus fix. While the shell's prompt editor (#627) has the keys, the
# terminal it sits in counts as focused (`rich_input::holds_focus`). In bash at a prompt, `echo hi`
# typed lands in the editor (`634-01-prompt`); README.md opened from the file finder takes the
# center (`634-02-readme`); ctrl-` goes back to the terminal, whose editor has the keys again
# (`634-03-terminal`); ctrl-` from there goes back to the code (`634-04-code`), where before the
# fix the terminal read as unfocused and the key refocused it.
compositor sway

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n\nThe code the toggle goes back to.\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 1.5

  echo "== at the prompt"
  type_text "echo hi"
  settle 1
  shot 634-01-prompt

  echo "== the code"
  press "CTRL" p
  settle 1.5
  type_text "README"
  settle 1.5
  press "" Return
  settle 2
  shot 634-02-readme

  echo "== back to the terminal"
  press "CTRL" grave
  settle 2
  shot 634-03-terminal

  echo "== and back to the code"
  press "CTRL" grave
  settle 2
  shot 634-04-code
}
