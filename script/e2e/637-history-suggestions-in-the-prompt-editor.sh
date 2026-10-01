# shellcheck shell=bash
# #637's e2e test: #484's history suggestion in the shell's prompt editor, at its default. In bash
# with a history file of its own, typing a prefix of a command in the editor shows the rest dimmed
# after it; → takes it and Enter runs it; a prefix nothing starts shows nothing; with the cursor
# before the end nothing shows and → only moves the cursor; and a command run in the session is
# suggested over the file's.
compositor sway

# The blocks, read through Marley's MCP server, name the commands that ran (#517).
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# Whether a block's command is exactly the command named, as the stand-in agent reads it.
ran() {
  mcp_agent terminal-read "$1" >"$E2E_WORK/ran.txt" || return 1
  cat "$E2E_WORK/ran.txt"
  head -1 "$E2E_WORK/ran.txt" | grep -qF ": '$1'"
}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  printf '%s\n' 'git status' 'echo hello world' 'ls -la' >"$home/.bash_history"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  # The terminal takes the focus, and the prompt editor docks.
  click 700 500
  settle 1.5

  echo "== a prefix of a history command"
  type_text "ech"
  settle 1
  shot 637-01-ghost
  press "" Right
  settle 1
  shot 637-02-taken
  press "" Return
  settle 2
  shot 637-03-ran

  echo "== nothing starts it"
  type_text "zzz"
  settle 1
  shot 637-04-nothing

  echo "== the cursor before the end"
  press CTRL c
  type_text "git st"
  settle 1
  press "" Left
  settle 1
  shot 637-05-left
  press "" Right
  settle 1
  shot 637-06-right

  echo "== the session's command first"
  press CTRL c
  type_text "echo from this session"
  press "" Return
  settle 2
  type_text "ec"
  settle 1
  shot 637-07-session-first
  expect "the suggestion taken with → ran as the history's command" ran "echo hello world"
  expect "this session's command ran" ran "echo from this session"
}
