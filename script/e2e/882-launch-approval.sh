# shellcheck shell=bash
# #882's visual check: a launch config's approval as the Workflows board draws it. #527's scratch
# repository, whose `.zed/marley.json` names `Dev`: a terminal titled `dev server` whose command ends
# in a shell comment holding Markdown (#592), a stand-in Claude Code split right, and a Browser tab
# split down. The project's + and its last entry, Dev: the approval card with three rows, the
# server's command whole and as written, Cancel and Run (`882-01-approve`, REQ-001); Marley Day
# (`882-02-day`, REQ-001); Enter runs it, and the three items open (`882-03-ran`, D1).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}

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
  # The Browser tab Run opens reaches no host but this machine.
  offline_chromium
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

teardown() {
  browser_teardown
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== Dev, from the project's +"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  press "" End
  settle 1
  press "" Return
  settle 2
  pointer_to 1500 950
  settle 1
  shot 882-01-approve

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 882-02-day

  echo "== Enter runs it: the three items open"
  press "" Return
  settle 8
  shot 882-03-ran
}
