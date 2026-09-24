# shellcheck shell=bash
# #483's e2e test, of the runner itself: a scratch repository opens behind Zed's trust prompt,
# Enter trusts it, and a command typed into the terminal prints. The terminal's shell reads a
# .bashrc of the scenario's own, with a plain prompt.

setup() {
  mkdir -p "$E2E_WORK/home"
  printf '%s\n' "PS1='\$ '" > "$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  git init -q -b e2e-runner "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  shot 483-01-trust-prompt
  press "" Return
  settle 2
  shot 483-02-trusted
  type_text "echo 'Marley e2e: A-Z ok!'"
  press "" Return
  settle 2
  shot 483-03-typed
}
