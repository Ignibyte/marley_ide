# shellcheck shell=bash
# #754's visual check: the Claude Code and Codex sessions in the CLIs' own files, listed in Threads
# and reopened. `CLAUDE_CONFIG_DIR` points at the run's own folder (the runner already points
# `CODEX_HOME` at one), holding two `cli` Claude Code sessions (one in the repository with an
# `ai-title` and the prompt "fix the flaky parser test in tokenizer.rs", one in `sub` with a
# `custom-title`), an `sdk-cli` one, a Codex TUI rollout in the repository named in
# `session_index.jsonl`, and a Codex subagent's rollout. Stand-in `claude` and `codex` first on the
# terminals' PATH log their arguments and folder to `launches.log`; the `claude` one prints #519's
# `SessionStart` for the session it resumes, so its terminal runs that session. Shots: the page
# (REQ-001, REQ-002), a search for a word only a prompt holds (REQ-003), the Codex chip (REQ-004), Reopen of each CLI
# (REQ-005, REQ-006), Reopen of a running one (REQ-007), the rail's Threads view (REQ-008), Reopen
# Thread… (REQ-009) and the switch off (REQ-010).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
CODEX_CHIP_X=${CODEX_CHIP_X:-655}
CODEX_CHIP_Y=${CODEX_CHIP_Y:-177}
ALL_CHIP_X=${ALL_CHIP_X:-288}
ALL_CHIP_Y=${ALL_CHIP_Y:-177}
CLAUDE_ROW_X=${CLAUDE_ROW_X:-386}
CLAUDE_ROW_Y=${CLAUDE_ROW_Y:-242}
CODEX_ROW_X=${CODEX_ROW_X:-344}
CODEX_ROW_Y=${CODEX_ROW_Y:-295}
THREADS_SWITCH_X=${THREADS_SWITCH_X:-88}
THREADS_SWITCH_Y=${THREADS_SWITCH_Y:-17}
PROJECTS_SWITCH_X=${PROJECTS_SWITCH_X:-32}
PROJECTS_SWITCH_Y=${PROJECTS_SWITCH_Y:-17}
REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}
STOP_AT=${STOP_AT:-}

CLAUDE_ONE=11111111-7540-4754-8754-111111111111
CLAUDE_TWO=22222222-7540-4754-8754-222222222222
CLAUDE_SDK=33333333-7540-4754-8754-333333333333
TUI_ROLLOUT=44444444-7540-4754-8754-444444444444
SUBAGENT_ROLLOUT=55555555-7540-4754-8754-555555555555

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

launches() {
  cat "$E2E_WORK/launches.log" 2>/dev/null || true
}

launched() {
  launches
  grep -qxF -- "$1" "$E2E_WORK/launches.log"
}

claude_launches() {
  grep -c '^claude:' "$E2E_WORK/launches.log" || true
}

# A Claude Code transcript: `$1` the id, `$2` the folder, `$3` the entrypoint, `$4` the prompt,
# then `ai-title <text>` or `custom-title <text>`.
claude_session() {
  local id=$1 folder=$2 entry=$3 prompt=$4 kind=$5 title=$6
  local dir=$E2E_WORK/claude/projects/${folder//\//-}
  mkdir -p "$dir"
  python3 - "$dir/$id.jsonl" "$id" "$folder" "$entry" "$prompt" "$kind" "$title" <<'PY'
import json, sys
path, ident, folder, entry, prompt, kind, title = sys.argv[1:]
base = {"sessionId": ident, "cwd": folder, "entrypoint": entry, "isSidechain": False,
        "timestamp": "2026-10-11T10:00:00.000Z", "version": "2.1.296"}
records = [
    {**base, "type": "user", "message": {"role": "user", "content": prompt}},
    {**base, "type": "assistant", "message": {"role": "assistant",
                                               "content": [{"type": "text", "text": "On it."}]}},
    {**base, "type": "user", "isMeta": True, "message": {"role": "user",
                                                          "content": "Caveat: a meta record"}},
]
if kind == "ai-title":
    records.append({"type": "ai-title", "sessionId": ident, "aiTitle": title})
else:
    records.append({"type": "custom-title", "sessionId": ident, "customTitle": title})
with open(path, "w", encoding="utf-8") as file:
    for record in records:
        file.write(json.dumps(record) + "\n")
PY
}

# A Codex rollout: `$1` the id, `$2` the folder, `$3` the thread source, `$4` the prompt.
codex_session() {
  local id=$1 folder=$2 source=$3 prompt=$4
  local dir=$CODEX_HOME/sessions/2026/10/11
  mkdir -p "$dir"
  python3 - "$dir/rollout-2026-10-11T10-00-00-$id.jsonl" "$id" "$folder" "$source" "$prompt" <<'PY'
import json, sys
path, ident, folder, source, prompt = sys.argv[1:]
meta = {"id": ident, "cwd": folder, "originator": "codex-tui", "source": "cli",
        "thread_source": source, "cli_version": "0.162.0", "timestamp": "2026-10-11T10:00:00Z"}
if source == "subagent":
    meta["parent_thread_id"] = "44444444-7540-4754-8754-444444444444"
records = [
    {"timestamp": "2026-10-11T10:00:00Z", "type": "session_meta", "payload": meta},
    {"timestamp": "2026-10-11T10:00:01Z", "type": "event_msg",
     "payload": {"type": "item_completed", "item": {
         "type": "UserMessage", "id": "u1", "client_id": "c1",
         "content": [{"type": "text", "text": prompt, "text_elements": []}]}}},
]
with open(path, "w", encoding="utf-8") as file:
    for record in records:
        file.write(json.dumps(record) + "\n")
PY
}

write_stand_ins() {
  local bin=$1
  cat >"$bin/claude" <<FAKE
#!/usr/bin/env python3
import base64, json, os, sys
with open("$E2E_WORK/launches.log", "a", encoding="utf-8") as log:
    log.write("claude: " + " ".join(sys.argv[1:]) + " @ " + os.getcwd() + "\n")
print("Claude Code (stand-in): " + " ".join(sys.argv[1:]), flush=True)
if "--resume" in sys.argv:
    session = sys.argv[sys.argv.index("--resume") + 1]
    body = {"v": 1, "cwd": os.getcwd(), "event": "SessionStart", "session_id": session,
            "source": "resume"}
    encoded = base64.b64encode(json.dumps(body).encode()).decode()
    sys.stdout.write("\x1b]777;notify;marley-event;" + encoded + "\x07")
    sys.stdout.flush()
for _ in sys.stdin:
    pass
FAKE
  cat >"$bin/codex" <<FAKE
#!/usr/bin/env python3
import os, sys
with open("$E2E_WORK/launches.log", "a", encoding="utf-8") as log:
    log.write("codex: " + " ".join(sys.argv[1:]) + " @ " + os.getcwd() + "\n")
print("Codex (stand-in): " + " ".join(sys.argv[1:]), flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$bin/claude" "$bin/codex"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/sub" "$CODEX_HOME"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  git init -q -b main "$repo"
  printf 'sessions\n' >"$repo/README.md"
  write_stand_ins "$bin"
  export CLAUDE_CONFIG_DIR=$E2E_WORK/claude
  claude_session "$CLAUDE_ONE" "$repo" cli "fix the flaky parser test in tokenizer.rs" \
    ai-title "Fix the flaky tokenizer test"
  claude_session "$CLAUDE_TWO" "$repo/sub" cli "write the release notes" \
    custom-title "Release notes for sub"
  claude_session "$CLAUDE_SDK" "$repo" sdk-cli "a harness task" ai-title "Harness task"
  codex_session "$TUI_ROLLOUT" "$repo" user "add a usage meter"
  codex_session "$SUBAGENT_ROLLOUT" "$repo" subagent "a subagent's task"
  printf '{"id":"%s","thread_name":"Usage meter","updated_at":"2026-10-11T10:00:02Z"}\n' \
    "$TUI_ROLLOUT" >"$CODEX_HOME/session_index.jsonl"
  # Newest first: the repository's Claude Code session, the Codex one, then `sub`'s.
  touch -d '-1 minutes' "$CLAUDE_CONFIG_DIR/projects/"*"/$CLAUDE_ONE.jsonl"
  touch -d '-2 minutes' "$CODEX_HOME/sessions/2026/10/11/rollout-2026-10-11T10-00-00-$TUI_ROLLOUT.jsonl"
  touch -d '-3 minutes' "$CLAUDE_CONFIG_DIR/projects/"*"/$CLAUDE_TWO.jsonl"
  open_path "$repo"
}

steps() {
  local repo=$E2E_WORK/repo
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== the Threads page"
  palette "marley: open threads" 4
  take 754-01-page || return 0

  echo "== a prompt search"
  type_text "parser"
  settle 2
  take 754-02-search || return 0
  press "CTRL" a
  press "" BackSpace
  settle 2

  echo "== the Codex chip"
  click "$CODEX_CHIP_X" "$CODEX_CHIP_Y"
  settle 2
  take 754-03-chip || return 0
  click "$ALL_CHIP_X" "$ALL_CHIP_Y"
  settle 2

  echo "== reopen the repository's Claude Code session"
  click "$CLAUDE_ROW_X" "$CLAUDE_ROW_Y"
  settle 6
  take 754-04-reopen || return 0
  expect "claude --resume in the repository" launched "claude: --resume $CLAUDE_ONE @ $repo"

  echo "== reopen the Codex session"
  palette "marley: open threads" 3
  click "$CODEX_ROW_X" "$CODEX_ROW_Y"
  settle 6
  take 754-05-codex || return 0
  expect "codex resume in the repository" launched "codex: resume $TUI_ROLLOUT @ $repo"

  echo "== reopen the running Claude Code session"
  palette "marley: open threads" 3
  click "$CLAUDE_ROW_X" "$CLAUDE_ROW_Y"
  settle 4
  take 754-06-forward || return 0
  expect "no second claude started" test "$(claude_launches)" -eq 1

  echo "== the rail's Threads view"
  click "$THREADS_SWITCH_X" "$THREADS_SWITCH_Y"
  settle 3
  take 754-07-rail || return 0

  echo "== Reopen Thread…, from the repository's +"
  click "$PROJECTS_SWITCH_X" "$PROJECTS_SWITCH_Y"
  settle 2
  click "$REPO_PLUS_X" "$REPO_PLUS_Y"
  settle 1
  press "" Home
  press "" Down
  press "" Down
  press "" Down
  press "" Return
  settle 2
  type_text "release notes"
  settle 2
  take 754-08-reopen-thread || return 0
  press "" Return
  settle 6
  expect "claude --resume in sub" launched "claude: --resume $CLAUDE_TWO @ $repo/sub"

  echo "== the switch off"
  profile_setting marley.list_cli_sessions false
  settle 3
  palette "marley: open threads" 3
  type_text "x"
  press "" BackSpace
  settle 2
  take 754-09-off || return 0
}
