# shellcheck shell=bash
# #783's e2e test: Up and Down in the shell's prompt editor walk the shell's history in the editor
# and never reach the grid behind it; in an agent's rich input they never reach the agent. Bash
# with a history file of its own, oldest first `git status`, `ls -la`, `echo hello world`,
# `ls -la`: newest first and each once, Up shows `ls -la`, `echo hello world`, `git status`.
# Then the shell becomes #481's stand-in Claude Code, which reads its lines in cooked mode, so an
# Up that reached it would put `^[[A` in the line it got.
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

# Whether the stand-in got exactly the line named, read from where its reply starts (#481).
stand_in_got() {
  mcp_agent terminal-read stand_in >"$E2E_WORK/stand-in.txt" || return 1
  cat "$E2E_WORK/stand-in.txt"
  sed -n 's/.*claude got: /claude got: /p' "$E2E_WORK/stand-in.txt" | grep -qxF "claude got: $1"
}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
stand_in() {
  exec -a claude bash -c 'printf "ready\n"; while IFS= read -r line; do printf "claude got: %s\n" "$line"; done'
}
RC
  printf '%s\n' 'git status' 'ls -la' 'echo hello world' 'ls -la' >"$home/.bash_history"
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

  echo "== Down with no walk"
  press "" Down
  settle 1
  shot 783-06-down-alone

  echo "== Up from the start of a draft, through the history"
  type_text "draft words"
  press "" Home
  press "" Up
  settle 1
  shot 783-01-up
  press "" Up
  settle 1
  shot 783-02-older
  press "" Up
  settle 1
  shot 783-03-oldest
  press "" Up
  settle 1
  shot 783-04-stays

  echo "== Down back to the draft"
  press "" Down
  press "" Down
  press "" Down
  settle 1
  shot 783-05-draft

  echo "== a recalled command runs"
  press CTRL c
  press "" Up
  press "" Up
  settle 1
  press "" Return
  settle 2
  shot 783-07-ran
  expect "the recalled command ran" ran "echo hello world"

  echo "== two lines: Up moves the cursor first"
  type_text "first"
  press SHIFT Return
  type_text "second"
  press "" Up
  settle 1
  shot 783-08-line-up
  press "" Up
  settle 1
  shot 783-08-then-walks
  press "" Down
  settle 1
  shot 783-08-draft-back

  echo "== an edit ends the walk"
  press CTRL c
  press "" Up
  type_text " again"
  press "" Up
  settle 1
  shot 783-09-walk-again
  press "" Down
  settle 1
  shot 783-10-edited-back

  echo "== the agent's rich input"
  press CTRL c
  type_text "stand_in"
  press "" Return
  settle 4
  press CTRL g
  settle 1
  type_text "after up"
  press "" Up
  press "" Up
  press "" Down
  press "" Down
  settle 1
  shot 783-11-agent
  press "" Return
  settle 2
  shot 783-11-agent-sent
  expect "the stand-in got the line and no arrow key" stand_in_got "after up"
}
