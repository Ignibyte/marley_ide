# shellcheck shell=bash
# #527's visual check, after the fact: a project's launch configs. A scratch repository whose
# `.zed/marley.json` names `Dev`: a terminal titled `dev server` running a small web server on a
# loopback port, a stand-in Claude Code split to the right with the focus, and a Browser tab on
# the server's page split down. The `+` lists `Dev` under Launch (REQ-001); choosing it asks with
# the config's text (REQ-002); Run opens the three, the page once the port answers (REQ-003,
# REQ-004); choosing it again opens it with no question (REQ-005); a changed file asks again and
# says so (REQ-006); a file that does not parse shows one disabled entry with its error (REQ-008).
# The server's command ends in a shell comment holding Markdown, which #592's approval shows as
# written: an HTML comment, emphasis and a run of backticks.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}

write_config() {
  cat >"$E2E_WORK/repo/.zed/marley.json" <<JSON
{
  // The scenario's launch configs.
  "launch": {
    "Dev": {
      "items": [
        { "terminal": "python3 -m http.server $1 --bind 127.0.0.1 # <!-- hidden --> *kept* \`\`\`", "title": "dev server" },
        { "agent": "claude", "split": "right", "focus": true },
        { "browser": "http://127.0.0.1:$1/", "split": "down" }
      ]
    }
  }
}
JSON
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/.zed"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$bin/claude" <<'SH'
#!/bin/sh
# The stand-in Claude Code: says where it runs and waits.
echo "stand-in Claude Code in $(pwd)"
exec cat >/dev/null
SH
  chmod +x "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  printf '<!doctype html><title>launched</title><h1>The dev server answers</h1>\n' >"$repo/index.html"
  git init -q -b main "$repo"
  write_config 8766
  open_path "$repo"
}

# Chooses the last entry of the project's `+`: the last launch config.
launch_last() {
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  press "" End
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== the + lists Dev under Launch"
  launch_last
  shot 527-01-menu
  press "" Return
  settle 2
  shot 527-02-approve
  echo "== Run: the three items open"
  press "" Return
  settle 8
  shot 527-03-opened
  echo "== again: no question"
  launch_last
  press "" Return
  settle 3
  shot 527-04-no-prompt-again
  echo "== a changed file asks again"
  write_config 8767
  settle 3
  launch_last
  press "" Return
  settle 2
  shot 527-05-changed
  press "" Escape
  settle 1
  echo "== a file that does not parse"
  printf '{ "launch": { "Dev": { "items": [ { "terminal": "x", "agent": "claude" } ] } } }\n' \
    >"$E2E_WORK/repo/.zed/marley.json"
  settle 3
  launch_last
  shot 527-06-broken
  press "" Escape
}
