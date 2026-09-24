# shellcheck shell=bash
# #481's e2e test. At the shell's plain prompt, Ctrl-G reaches the program (`cat -v` shows
# `^G`). Then the shell becomes a stand-in Claude Code (`exec -a claude`, so the terminal's
# foreground process is `claude` by name) that prints each line it reads. Ctrl-G opens the
# rich input, whose text reaches the stand-in as one paste and Enter, and Escape keeps a draft.

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  cat > "$home/.bashrc" <<'RC'
PS1='$ '
stand_in() {
  exec -a claude bash -c 'printf "ready\n"; while IFS= read -r line; do printf "claude got: %s\n" "$line"; done'
}
RC
  terminal_env HOME "$home"
  git init -q -b rich-input "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "cat -v"
  press "" Return
  settle 1
  press CTRL g
  press "" Return
  settle 1
  shot 481-01-ctrl-g-passes
  press CTRL d
  settle 1
  type_text "stand_in"
  press "" Return
  settle 3
  shot 481-02-bar
  press CTRL g
  settle 1
  shot 481-03-open
  type_text "hello rich input"
  press SHIFT Return
  type_text "second line"
  settle 1
  shot 481-04-two-lines
  press "" Return
  settle 2
  shot 481-05-sent
  # The terminal has the focus back: this reaches the stand-in, not the editor.
  type_text "typed after"
  press "" Return
  settle 1
  shot 481-05-focus-back
  press CTRL g
  settle 1
  type_text "draft"
  press "" Escape
  settle 1
  shot 481-06-escaped
  press CTRL g
  settle 1
  shot 481-07-draft
}
