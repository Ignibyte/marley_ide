# shellcheck shell=bash
# #768's visual check: Claude Code's and Codex's MCP servers added, changed and removed from MCP
# Servers. #720's fixtures: Claude Code in the run's own folder (`CLAUDE_CONFIG_DIR`; the runner
# points `CODEX_HOME` at one), `.claude.json` with `notes` (env `API_TOKEN`, `NOTES_DIR`) and the
# repository's local `tracker`, the repository's `.mcp.json` with `docs`, Codex's `config.toml`
# with a comment, `search` and `old-wiki`. Each write's file is printed into the run's log.
# Shots: Add Server under Codex filled (REQ-001), the tab after it (REQ-001, REQ-006), Edit on
# `notes` with its `--api-key` argument hidden and its env names (REQ-003), the tab after a
# variable was added with the rest left as shown (REQ-002, REQ-003), Remove on `docs` asking and
# done (REQ-004), and a save refused after `search` changed behind the form (REQ-005).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
ADD_CODEX_X=${ADD_CODEX_X:-366}
ADD_CODEX_Y=${ADD_CODEX_Y:-695}
FORM_COMMAND_X=${FORM_COMMAND_X:-800}
FORM_COMMAND_Y=${FORM_COMMAND_Y:-263}
FORM_ARGUMENTS_X=${FORM_ARGUMENTS_X:-800}
FORM_ARGUMENTS_Y=${FORM_ARGUMENTS_Y:-325}
FORM_ADD_PAIR_X=${FORM_ADD_PAIR_X:-537}
FORM_ADD_PAIR_Y=${FORM_ADD_PAIR_Y:-385}
FORM_PAIR_VALUE_X=${FORM_PAIR_VALUE_X:-850}
FORM_PAIR_VALUE_Y=${FORM_PAIR_VALUE_Y:-387}
EDIT_NOTES_X=${EDIT_NOTES_X:-396}
EDIT_NOTES_Y=${EDIT_NOTES_Y:-387}
EDIT_ADD_PAIR_X=${EDIT_ADD_PAIR_X:-537}
EDIT_ADD_PAIR_Y=${EDIT_ADD_PAIR_Y:-389}
EDIT_PAIR_VALUE_X=${EDIT_PAIR_VALUE_X:-850}
EDIT_PAIR_VALUE_Y=${EDIT_PAIR_VALUE_Y:-391}
REMOVE_DOCS_X=${REMOVE_DOCS_X:-517}
REMOVE_DOCS_Y=${REMOVE_DOCS_Y:-541}
PROMPT_REMOVE_X=${PROMPT_REMOVE_X:-799}
PROMPT_REMOVE_Y=${PROMPT_REMOVE_Y:-545}
EDIT_SEARCH_X=${EDIT_SEARCH_X:-403}
EDIT_SEARCH_Y=${EDIT_SEARCH_Y:-670}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

# Prints `$1` into the run's log under a heading.
show() {
  echo "-- $1"
  cat "$1" 2>/dev/null || echo "(no file)"
  echo
}

# The value of `$2` (a Python expression over `config`) in the JSON file `$1`.
json_value() {
  python3 - "$1" "$2" <<'PY'
import json, sys
with open(sys.argv[1], encoding="utf-8") as file:
    config = json.load(file)
print(json.dumps(eval(sys.argv[2], {"config": config})))
PY
}

setup() {
  local repo=$E2E_WORK/repo
  mkdir -p "$repo" "$E2E_WORK/claude" "$CODEX_HOME"
  export CLAUDE_CONFIG_DIR=$E2E_WORK/claude
  python3 - "$CLAUDE_CONFIG_DIR/.claude.json" "$repo" <<'PY'
import json, sys
path, repo = sys.argv[1:]
config = {
    "numStartups": 3,
    "tipsHistory": {"new-user-warmup": 1.5},
    "mcpServers": {
        "notes": {"type": "stdio", "command": "notes-mcp",
                  "args": ["--stdio", "--api-key=not-a-real-key"],
                  "env": {"API_TOKEN": "not-a-real-token", "NOTES_DIR": "/tmp/notes"}},
    },
    "projects": {
        repo: {
            "allowedTools": [],
            "mcpServers": {
                "tracker": {"type": "http", "url": "https://tracker.example.com/mcp",
                            "headers": {"Authorization": "Bearer not-a-real-token"}},
            },
            "enabledMcpjsonServers": ["docs"],
            "disabledMcpjsonServers": [],
        },
    },
}
with open(path, "w", encoding="utf-8") as file:
    json.dump(config, file, indent=2)
PY
  printf '{\n  "mcpServers": {\n    "docs": {"command": "docs-mcp", "args": ["serve"]},\n    "wiki": {"command": "wiki-mcp"}\n  }\n}\n' >"$repo/.mcp.json"
  cat >"$CODEX_HOME/config.toml" <<'TOML'
# The run's Codex config: a comment that stays.
model = "gpt-5"

[mcp_servers.search]
command = "search-mcp"
args = ["--stdio"]
env = { SEARCH_API_KEY = "not-a-real-key" }

[mcp_servers.old-wiki]
url = "https://wiki.example.com/mcp"
enabled = false
TOML
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  local repo=$E2E_WORK/repo claude_json=$E2E_WORK/claude/.claude.json
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  palette "marley: open mcp servers" 4
  take 768-00-tab || return 0

  echo "== Add Server under Codex"
  click "$ADD_CODEX_X" "$ADD_CODEX_Y"
  settle 2
  take 768-01a-form-open || return 0
  type_text "lint"
  click "$FORM_COMMAND_X" "$FORM_COMMAND_Y"
  type_text "lint-mcp"
  click "$FORM_ARGUMENTS_X" "$FORM_ARGUMENTS_Y"
  type_text "--stdio"
  click "$FORM_ADD_PAIR_X" "$FORM_ADD_PAIR_Y"
  settle 1
  type_text "LINT_TOKEN"
  click "$FORM_PAIR_VALUE_X" "$FORM_PAIR_VALUE_Y"
  type_text "not-a-real-token"
  settle 1
  take 768-01-add-form || return 0
  press "" Return
  settle 3
  take 768-02-added || return 0
  show "$CODEX_HOME/config.toml"
  ls -a "$CODEX_HOME"
  expect "Codex's config has the lint table" grep -q '^\[mcp_servers.lint\]$' "$CODEX_HOME/config.toml"
  expect "with its env value" grep -q 'LINT_TOKEN = "not-a-real-token"' "$CODEX_HOME/config.toml"
  expect "the comment stayed" grep -q "a comment that stays" "$CODEX_HOME/config.toml"
  expect "search stayed" grep -q 'SEARCH_API_KEY = "not-a-real-key"' "$CODEX_HOME/config.toml"
  expect "a backup beside it" test -s "$CODEX_HOME/config.toml.marley-backup"

  echo "== Edit notes"
  click "$EDIT_NOTES_X" "$EDIT_NOTES_Y"
  settle 2
  take 768-03-edit-form || return 0
  # The arguments stay as shown, their key hidden; one variable is added.
  click "$EDIT_ADD_PAIR_X" "$EDIT_ADD_PAIR_Y"
  settle 1
  type_text "NOTES_MODE"
  click "$EDIT_PAIR_VALUE_X" "$EDIT_PAIR_VALUE_Y"
  type_text "fast"
  press "" Return
  settle 3
  take 768-04-edited || return 0
  json_value "$claude_json" 'config["mcpServers"]["notes"]["args"]'
  expect "the hidden argument kept its value" test "$(json_value "$claude_json" 'config["mcpServers"]["notes"]["args"]')" = '["--stdio", "--api-key=not-a-real-key"]'
  expect "API_TOKEN kept its value" test "$(json_value "$claude_json" 'config["mcpServers"]["notes"]["env"]["API_TOKEN"]')" = '"not-a-real-token"'
  expect "NOTES_MODE was added" test "$(json_value "$claude_json" 'config["mcpServers"]["notes"]["env"]["NOTES_MODE"]')" = '"fast"'
  expect "the other keys stayed" test "$(json_value "$claude_json" '[config["numStartups"], config["tipsHistory"]]')" = '[3, {"new-user-warmup": 1.5}]'
  expect "the local server stayed" test "$(json_value "$claude_json" 'sorted(config["projects"]["'"$repo"'"]["mcpServers"])')" = '["tracker"]'

  echo "== Remove docs"
  click "$REMOVE_DOCS_X" "$REMOVE_DOCS_Y"
  settle 2
  take 768-05-remove-ask || return 0
  click "$PROMPT_REMOVE_X" "$PROMPT_REMOVE_Y"
  settle 3
  take 768-06-removed || return 0
  show "$repo/.mcp.json"
  expect "docs is gone" test "$(json_value "$repo/.mcp.json" 'sorted(config["mcpServers"])')" = '["wiki"]'

  echo "== a save refused: search changed behind the form"
  click "$EDIT_SEARCH_X" "$EDIT_SEARCH_Y"
  settle 2
  sed -i 's/^args = \["--stdio"\]$/args = ["--stdio", "--changed"]/' "$CODEX_HOME/config.toml"
  press "" Return
  settle 3
  take 768-07-refused || return 0
  expect "the outside change stayed" grep -q -- '"--changed"' "$CODEX_HOME/config.toml"
}
