# shellcheck shell=bash
# #484's e2e test: a bash with a plain prompt and a history file of its own. Typing a prefix of
# a command in it shows the rest dimmed after the cursor; → takes it; a prefix nothing starts
# shows nothing; and a command run in the session is suggested over the file's.

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  printf '%s\n' "PS1='\$ '" > "$home/.bashrc"
  printf '%s\n' 'git status' 'echo hello world' 'ls -la' > "$home/.bash_history"
  terminal_env HOME "$home"
  git init -q -b autosuggest "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "ech"
  settle 1
  shot 484-01-ghost
  press "" Right
  settle 1
  shot 484-02-taken
  press "" Return
  settle 1
  shot 484-03-ran
  type_text "zzz"
  settle 1
  shot 484-04-nothing
  # Without a suggestion, → is readline's: Left and then → put the cursor back after the text.
  press "" Left
  settle 1
  shot 484-04-left
  press "" Right
  settle 1
  shot 484-04-right
  press CTRL u
  type_text "echo from this session"
  press "" Return
  settle 1
  type_text "ec"
  settle 1
  shot 484-05-session-first
}
