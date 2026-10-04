# shellcheck shell=bash
# #652's visual check: agent reports through `$MARLEY_BIN`. A stand-in `claude`, first on the
# terminal's PATH, prints its `MARLEY_BIN` and `MARLEY_TERMINAL_ID`, logs its arguments and folder
# to `claude.log`, and on each line `n` runs the next step of the script `CLAUDE_SCRIPT` names: a
# hook frame as #519's plugin prints it, a run of `"$MARLEY_BIN" report` or `release` as the shared
# plugin's mod would run it (printing the exit, the first word on stderr and the time taken), the
# same from a child `sh` (another process than the agent), a helper process that reports and
# exits, or a copy of the program whose socket file names a missing socket. The hook frames name
# the sessions `1111…`, the reports `2222…`, so each shot shows which the row follows. Never the
# user's Claude Code.
#
# `652-01-environment`, `652-02-task`, `652-03-reported`, `652-04-waiting`, `652-05-interrupted`,
# `652-06-refused`, `652-07-released`, `652-08-fallback`, `652-09-lapsed`, `652-10-resumed`.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK_A=11111111-1111-4111-8111-11111111111a
REPORT_A=22222222-2222-4222-8222-22222222222a
HOOK_B=11111111-1111-4111-8111-11111111111b
HOOK_C=11111111-1111-4111-8111-11111111111c
REPORT_C=22222222-2222-4222-8222-22222222222c

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# Runs the next step of the stand-in's script.
step() {
  type_text "n"
  press "" Return
  settle "${1:-2}"
}

# Whether the stand-in printed `$1`.
printed() { grep -qF -- "$1" "$E2E_WORK/stand-in.log"; }

write_scripts() {
  python3 - "$E2E_WORK" "$HOOK_A" "$REPORT_A" "$HOOK_B" "$HOOK_C" "$REPORT_C" <<'SCRIPTS'
import json, sys
work, hook_a, report_a, hook_b, hook_c, report_c = sys.argv[1:]
source = ["--source", "mod:claude-code"]
def report(state, *rest, seq="@SEQ"):
    return {"report": source + ["--seq", seq, state, *rest]}
def resume(session):
    return ["--session-id", session, "--resume-arg=claude", "--resume-arg=--resume",
            f"--resume-arg={session}"]
scripts = {
    "a": [
        [{"frame": {"event": "SessionStart", "session_id": hook_a, "source": "startup"}},
         report("idle", *resume(report_a))],
        [{"frame": {"event": "UserPromptSubmit", "session_id": hook_a, "prompt": "Add a README"}}],
        [report("working", *resume(report_a)),
         {"frame": {"event": "PreToolUse", "session_id": hook_a, "tool": "Bash",
                    "preview": "cargo test", "tool_use_id": "t1"}},
         {"frame": {"event": "PermissionRequest", "session_id": hook_a, "tool": "Bash",
                    "preview": "cargo test", "tool_use_id": "t1"}}],
        [{"frame": {"event": "PostToolUse", "session_id": hook_a, "tool": "Bash",
                    "preview": "cargo test", "tool_use_id": "t1"}},
         report("idle", "--activity", "the last turn was interrupted", *resume(report_a))],
        [dict(report("idle", seq="1"), label="stale seq"),
         {"child": source + ["--seq", "@SEQ", "idle"], "label": "child report"},
         {"child_release": source, "label": "child release"},
         dict(report("idle", "--resume-arg=claude", "--resume-arg=it's"), label="apostrophe"),
         dict(report("idle", "--prompt", "Go?", "--option", "yes"), label="question idle"),
         {"lost": source + ["--seq", "@SEQ", "idle"], "label": "missing socket"}],
        [{"release": source, "label": "release"}, {"exit": True}],
    ],
    "b": [
        [{"frame": {"event": "SessionStart", "session_id": hook_b, "source": "startup"}},
         {"frame": {"event": "UserPromptSubmit", "session_id": hook_b, "prompt": "Fix the build"}},
         {"frame": {"event": "PreToolUse", "session_id": hook_b, "tool": "Bash",
                    "preview": "make", "tool_use_id": "b1"}}],
        [{"helper": source + ["--seq", "@SEQ", "idle"], "label": "helper"},
         {"frame": {"event": "UserPromptSubmit", "session_id": hook_b, "prompt": "Second prompt"}},
         {"frame": {"event": "PreToolUse", "session_id": hook_b, "tool": "Read",
                    "preview": "README.md", "tool_use_id": "b2"}}],
    ],
    "c": [
        [{"frame": {"event": "SessionStart", "session_id": hook_c, "source": "startup"}},
         report("idle", *resume(report_c))],
    ],
}
for name, steps in scripts.items():
    with open(f"{work}/script-{name}.json", "w") as file:
        json.dump(steps, file)
SCRIPTS
}

write_stand_in() {
  sed -e "s|@WORK@|$E2E_WORK|g" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: its variables, its hook frames and its mod's reports, step by step.
import base64
import json
import os
import shutil
import subprocess
import sys
import time

WORK = "@WORK@"
args = sys.argv[1:]
folder = os.getcwd()
with open(f"{WORK}/claude.log", "a", encoding="utf-8") as log:
    log.write(f"{' '.join(args)} @ {folder}\n")


def say(line):
    print(line, flush=True)
    with open(f"{WORK}/stand-in.log", "a", encoding="utf-8") as log:
        log.write(line + "\n")


program = os.environ.get("MARLEY_BIN", "")
say(f"MARLEY_BIN={program}")
say(f"MARLEY_TERMINAL_ID={os.environ.get('MARLEY_TERMINAL_ID', '')}")
with open(f"{WORK}/env.txt", "w", encoding="utf-8") as env:
    env.write(os.environ.get("MARLEY_TERMINAL_ID", "") + "\n")
if "--resume" in args:
    say(f"resumed {args[args.index('--resume') + 1]} in {folder}")
seq = [int(time.time() * 1000)]


def fill(arguments):
    filled = []
    for argument in arguments:
        if argument == "@SEQ":
            seq[0] = max(seq[0] + 1, int(time.time() * 1000))
            argument = str(seq[0])
        filled.append(argument)
    return filled


def run(label, argv):
    started = time.monotonic()
    done = subprocess.run(argv, capture_output=True, text=True, check=False)
    took = int((time.monotonic() - started) * 1000)
    first = (done.stderr.split() or ["-"])[0].rstrip(":")
    say(f"{label}: exit {done.returncode} {first} {took} ms")


def frame(summary):
    summary = {"v": 1, "cwd": folder, **summary}
    body = base64.b64encode(json.dumps(summary).encode()).decode()
    sys.stdout.write(f"\x1b]777;notify;marley-event;{body}\x07")
    sys.stdout.flush()


name = os.environ.get("CLAUDE_SCRIPT", "")
steps = json.load(open(f"{WORK}/script-{name}.json")) if name else []
for line in sys.stdin:
    if line.strip() != "n" or not steps:
        continue
    for action in steps.pop(0):
        label = action.get("label", "")
        if "frame" in action:
            frame(action["frame"])
            time.sleep(0.2)
        elif "report" in action:
            arguments = fill(action["report"])
            run(label or f"report {arguments[4]}", [program, "report", *arguments])
        elif "release" in action:
            run(label, [program, "release", *action["release"]])
        elif "child" in action:
            run(label, ["sh", "-c", '"$0" "$@"; exit $?', program, "report", *fill(action["child"])])
        elif "child_release" in action:
            run(label, ["sh", "-c", '"$0" "$@"; exit $?', program, "release", *action["child_release"]])
        elif "helper" in action:
            helper = f"import subprocess, sys; sys.exit(subprocess.run(sys.argv[1:]).returncode)"
            run(label, [sys.executable, "-c", helper, program, "report", *fill(action["helper"])])
        elif "lost" in action:
            lost = f"{WORK}/lost"
            os.makedirs(lost, exist_ok=True)
            shutil.copy(program, f"{lost}/marley-agent")
            with open(f"{lost}/agent-socket", "w", encoding="utf-8") as file:
                file.write(f"{lost}/none.sock\n")
            run(label, [f"{lost}/marley-agent", "report", *fill(action["lost"])])
        elif action.get("exit"):
            say("bye")
            sys.exit(0)
FAKE
  chmod +x "$1"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/sub" "$repo/.zed"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_scripts
  write_stand_in "$bin/claude"
  : >"$E2E_WORK/claude.log"
  cat >"$repo/.zed/tasks.json" <<'JSON'
[{"label": "env", "command": "echo MARLEY_BIN=[$MARLEY_BIN]", "use_new_terminal": true}]
JSON
  profile_setting marley.no_update_after_minutes 0
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== a task's MARLEY_BIN"
  palette "task: spawn"
  type_text "env"
  settle 1
  press "" Return
  settle 3
  shot 652-02-task

  echo "== terminal A: the variables"
  palette "workspace: new terminal"
  type_text "cd sub && CLAUDE_SCRIPT=a claude"
  press "" Return
  settle 3
  shot 652-01-environment
  expect "MARLEY_BIN names the run's own program" printed "MARLEY_BIN=$E2E_PROFILE/mcp/marley-agent"

  echo "== reported idle, then a prompt frame"
  step
  step
  shot 652-03-reported
  expect "the report was taken" printed "report idle: exit 0"
  mcp_agent fleet-report

  echo "== reported working, then a permission frame"
  step
  shot 652-04-waiting

  echo "== the tool's end, then an interrupted idle"
  step 1
  shot 652-05-interrupted
  mcp_agent fleet-report

  echo "== refused"
  step 4
  shot 652-06-refused
  expect "the stale seq was refused" printed "stale seq: exit 1 agent_report_stale"
  expect "the child's report was refused" printed "child report: exit 1 agent_report_authority"
  expect "the child's release was refused" printed "child release: exit 1 agent_release_authority"
  expect "the apostrophe was refused" printed "apostrophe: exit 1 agent_report_argv"
  expect "the question without waiting was refused" printed "question idle: exit 1 agent_report_question"
  expect "the missing socket says so" printed "missing socket: exit 1 marley_not_running"

  echo "== from outside Marley"
  local outside
  outside=$(MARLEY_TERMINAL_ID=$(cat "$E2E_WORK/env.txt") "$E2E_PROFILE/mcp/marley-agent" report idle --source mod:claude-code --seq 9999999999999 2>&1 || true)
  echo "  outside: $outside"
  expect "a report from outside Marley is unknown" test "${outside%%:*}" = agent_unknown

  echo "== released"
  step 2
  shot 652-07-released
  expect "the release was taken" printed "release: exit 0"

  echo "== terminal B: frames only"
  palette "workspace: new terminal"
  type_text "CLAUDE_SCRIPT=b claude"
  press "" Return
  settle 3
  step
  shot 652-08-fallback

  echo "== a helper reports and goes"
  step 3
  shot 652-09-lapsed
  expect "the helper's report was taken" printed "helper: exit 0"

  echo "== terminal C: a reported session, then a relaunch"
  palette "workspace: new terminal"
  type_text "cd sub && CLAUDE_SCRIPT=c claude"
  press "" Return
  settle 3
  step
  # B still works by its frames, which the close guard would ask about at the quit.
  profile_setting marley.ask_before_ending_a_working_agent false
  settle 3
  quit_marley
  launch_marley
  settle 14
  shot 652-10-resumed
  expect "C resumed the reported session in sub" grep -qxF -- "--resume $REPORT_C @ $E2E_WORK/repo/sub" "$E2E_WORK/claude.log"
  expect "nothing resumed the released or the hook sessions of A" test "$(grep -cF -- "--resume $REPORT_A" "$E2E_WORK/claude.log" || true)" = 0
  cat "$E2E_WORK/claude.log"
}
