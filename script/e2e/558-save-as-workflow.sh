# shellcheck shell=bash
# #558's visual check: a block's command saved as a workflow, a Zed task with `{{name}}`
# parameters. A scratch repository on `feature/one` with `web/index.html` and a `.zed/tasks.json`
# holding a comment and one task. A web server's block shows Save as Workflow on hover (REQ-001);
# the editor guesses `{{port}}` and `{{path}}` with their tokens as defaults (REQ-002); Save
# appends the task and keeps the file's comment and task (REQ-003); Zed's picker lists it
# (REQ-004); running it asks for the values (REQ-005) and serves on the port given (REQ-006); a
# rerun asks again with the last values, and Escape there runs nothing (REQ-008, REQ-009); a
# command with no guesses saves and runs at once (REQ-007); Escape in the editor writes nothing
# (REQ-009).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# The server block's first row and its Save as Workflow button, first in the hover actions, from
# the shots.
SERVER_ROW_Y=${SERVER_ROW_Y:-877}
WORKFLOW_BUTTON_X=${WORKFLOW_BUTTON_X:-1240}

# Two ports free on this machine, picked at run time: a fixed one can be taken by a service the
# box runs, and the server's bind then fails.
free_port() {
  python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}
SAVED_PORT=$(free_port)
GIVEN_PORT=$(free_port)
while [ "$GIVEN_PORT" = "$SAVED_PORT" ]; do
  GIVEN_PORT=$(free_port)
done

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/web" "$repo/.zed"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  printf '<!doctype html><title>served</title><h1>served</h1>\n' >"$repo/web/index.html"
  cat >"$repo/.zed/tasks.json" <<'JSON'
// The scenario's own tasks: Save as Workflow keeps this comment.
[
  {
    "label": "hello",
    "command": "echo hello"
  }
]
JSON
  git init -q -b feature/one "$repo"
  git -C "$repo" add -A
  git -C "$repo" -c user.name=Scenario -c user.email=scenario@example.invalid commit -q -m Start
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Whether a listener is on 127.0.0.1:`$1`.
listening() {
  ss -ltn | grep -q "127.0.0.1:$1 "
}

tasks_file() {
  cat "$E2E_WORK/repo/.zed/tasks.json"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== a server's block: Save as Workflow on hover, the editor with its guesses"
  type_text "python3 -m http.server $SAVED_PORT --bind 127.0.0.1 --directory web"
  press "" Return
  settle 2
  press CTRL c
  settle 2
  pointer_to "$WORKFLOW_BUTTON_X" "$SERVER_ROW_Y"
  settle 1
  shot 558-01-button
  click "$WORKFLOW_BUTTON_X" "$SERVER_ROW_Y"
  settle 2
  shot 558-02-editor

  echo "== Save appends the task and keeps the file"
  press CTRL a
  type_text "serve"
  press "" Return
  settle 2
  shot 558-03-saved
  tasks_file | tee "$E2E_WORK/tasks-after-serve.json"
  expect "the file keeps its comment and its task" \
    holds "$E2E_WORK/tasks-after-serve.json" "Save as Workflow keeps this comment" '"hello"'
  expect "and holds the workflow with its parameters" \
    holds "$E2E_WORK/tasks-after-serve.json" '"serve"' '{{port}}' '{{path}}' "\"default\": \"$SAVED_PORT\""

  echo "== Zed's picker lists it; running it asks for the values"
  press "ALT SHIFT" t
  settle 2
  type_text "serve"
  settle 1
  shot 558-04-picker
  press "" Return
  settle 2
  shot 558-05-parameters
  press CTRL a
  type_text "$GIVEN_PORT"
  press "" Return
  settle 4
  shot 558-06-ran
  expect "the workflow serves on the port given" listening "$GIVEN_PORT"

  echo "== a rerun asks again with the last values; Escape runs nothing"
  ss -ltnpH "sport = :$GIVEN_PORT" >"$E2E_WORK/server-before.txt"
  press "CTRL ALT" r
  settle 2
  shot 558-08-rerun
  press "" Escape
  settle 2
  ss -ltnpH "sport = :$GIVEN_PORT" >"$E2E_WORK/server-after.txt"
  cat "$E2E_WORK/server-after.txt"
  expect "the cancelled rerun left the same server running" \
    cmp -s "$E2E_WORK/server-before.txt" "$E2E_WORK/server-after.txt"

  echo "== a command with nothing to guess saves and runs at once"
  press ALT 1
  settle 1
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "git status --short --branch"
  press "" Return
  settle 2
  press CTRL Up
  settle 1
  palette "marley: save as workflow"
  settle 2
  shot 558-07a-editor
  press "" Return
  settle 2
  tasks_file | tee "$E2E_WORK/tasks-after-status.json"
  expect "the second workflow is saved with no parameters" \
    holds "$E2E_WORK/tasks-after-status.json" '"git status"'
  press "ALT SHIFT" t
  settle 2
  type_text "git status"
  settle 1
  press "" Return
  settle 3
  shot 558-07-no-parameters

  echo "== Escape in the editor writes nothing"
  press ALT 1
  settle 1
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  press CTRL Up
  settle 1
  palette "marley: save as workflow"
  settle 2
  press "" Escape
  settle 1
  expect "the file is as it was" cmp -s "$E2E_WORK/tasks-after-status.json" "$E2E_WORK/repo/.zed/tasks.json"
}
