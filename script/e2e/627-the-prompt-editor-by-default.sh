# shellcheck shell=bash
# #627's e2e test: the prompt editor by default. In bash, with `marley.prompt_editor` at its
# default, the editor docks at the prompt and `echo hi` typed lands in it (`prompt`, REQ-001);
# Enter runs it as a block and the editor comes back empty at the next prompt (`ran`, REQ-001).
# `vim` takes the screen and the editor goes, `:q` reaching vim raw (`vim`, REQ-002); vim gone, the
# editor is back (`back`, REQ-002).
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
  settle 1.5

  echo "== at the prompt"
  type_text "echo hi"
  settle 1
  shot prompt
  press "" Return
  settle 3
  shot ran

  echo "== vim, raw"
  type_text "vim"
  press "" Return
  settle 3
  shot vim
  type_text ":q"
  press "" Return
  settle 3
  shot back
}
