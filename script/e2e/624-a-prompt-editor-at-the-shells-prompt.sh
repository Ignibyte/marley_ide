# shellcheck shell=bash
# #624's e2e test: the footer editor at a shell's prompt. In bash, `ech` is typed at the prompt;
# Ctrl+G opens the editor holding `ech` (`editor`, REQ-001); `o hi` typed there makes `echo hi`
# (`typed`); Enter clears the shell's line and runs it: a block `echo hi` with `hi`, the editor
# closed (`ran`, REQ-002).
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
  type_text "ech"
  settle 1

  echo "== Ctrl+G at the prompt"
  press "CTRL" g
  settle 1.5
  shot editor
  type_text "o hi"
  settle 1
  shot typed

  echo "== Enter"
  press "" Return
  settle 3
  pointer_to 700 300
  settle 1
  shot ran
}
