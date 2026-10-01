# shellcheck shell=bash
# #626's e2e test: a command's colours at the prompt. In bash, `echo "hi" | grep h $HOME` typed at
# the prompt is drawn in the theme's syntax colours: the command words, the string and the
# variable apart (`prompt`, REQ-001). Ctrl+G's editor holds it in the same colours (`editor`,
# REQ-002); Enter runs it, the block keeping the shell's own drawing (`ran`, REQ-003).
compositor sway

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 1
  type_text "echo \"hi\" | grep h \$HOME"
  settle 1.5
  pointer_to 700 300
  settle 0.5
  shot prompt

  echo "== the prompt editor"
  press "CTRL" g
  settle 1.5
  shot editor

  echo "== Enter"
  press "" Return
  settle 3
  shot ran
}
