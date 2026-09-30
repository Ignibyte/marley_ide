# shellcheck shell=bash
# #587's e2e test: Claude Code's trust question in a new worktree, brought to the user. A scratch
# repository on `main`, trusted in Zed, and a fake `claude` that, unless its own record names the
# repository's main checkout (Claude Code keys its trust there), draws the question in its
# 2.1.263 form with the focus on "No, exit" and a warning line, and logs each key it reads. New
# Agent in Worktree's Claude Code shows a notification naming the worktree and the folder, with
# the warning, Trust Folder and Show Terminal, over the main checkout's workspace (REQ-001,
# REQ-002); Show Terminal shows the agent's terminal with the question (REQ-004); Trust Folder
# picks "Yes, I trust this folder" from "No, exit" (REQ-003); a question answered elsewhere takes
# the notification with it and Marley sends nothing (REQ-005); `follow_zed` answers it without a
# notification, with a toast (REQ-006); a repository Claude Code already trusts gets no question
# and nothing (REQ-007).
compositor sway

# The project's +, New Agent in Worktree's Claude Code and the main checkout's terminal row, as
# #510's scenario has them; the notification's buttons, from the first run's shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
WORKTREE_STEPS=${WORKTREE_STEPS:-3}
SUBMENU_CLAUDE_X=${SUBMENU_CLAUDE_X:-325}
SUBMENU_CLAUDE_Y=${SUBMENU_CLAUDE_Y:-197}
MAIN_ROW_X=${MAIN_ROW_X:-120}
MAIN_ROW_Y=${MAIN_ROW_Y:-136}
TRUST_X=${TRUST_X:-1190}
TRUST_Y=${TRUST_Y:-932}
SHOW_X=${SHOW_X:-1273}
SHOW_Y=${SHOW_Y:-932}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/src"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1

  git init -q -b main "$repo"
  printf 'hello\n' >"$repo/src/lib.txt"
  repo_git add -A
  repo_git commit -q -m "Start"

  write_fake_claude "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  open_path "$repo"
}

# A fake `claude`: unless its record names the repository's main checkout, the trust question in
# Claude Code 2.1.263's form, read with raw keys; each key and the choice go to a log, with the
# process's id. On the trust option it records the main checkout and prints its arguments; on
# "No, exit" it exits. The flag file `answer-yes` answers yes, as a user at the terminal would.
write_fake_claude() {
  sed -e "s|@LOG@|$E2E_WORK/claude.log|g" -e "s|@RECORD@|$E2E_WORK/claude-trust.txt|g" \
    -e "s|@FLAG@|$E2E_WORK/answer-yes|g" >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import select
import signal
import subprocess
import sys
import termios
import tty

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
ARGS = sys.argv[1:]
PID = os.getpid()


def log(**entry):
    with open("@LOG@", "a", encoding="utf-8") as out:
        out.write(json.dumps({"pid": PID, **entry}) + "\n")


def on_signal(number, _frame):
    log(signal=number)
    sys.exit(128 + number)


for signal_number in (signal.SIGHUP, signal.SIGTERM, signal.SIGINT):
    signal.signal(signal_number, on_signal)
sys.excepthook = lambda kind, value, _trace: log(error=f"{kind.__name__}: {value}")


cwd = os.getcwd()
common = subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"],
                        capture_output=True, text=True, check=False).stdout.strip()
root = os.path.dirname(common) if common.endswith("/.git") else cwd
record = open("@RECORD@", encoding="utf-8").read().split("\n") if os.path.exists("@RECORD@") else []
log(launch=ARGS, cwd=cwd, root=root, record=record)
if root not in record:
    options = ["No, exit", "Yes, I trust this folder"]
    focus = 0

    def draw():
        lines = ["Accessing workspace:", "", cwd, "",
                 "Quick safety check: Is this a project you created or one you trust?",
                 "Claude Code'll be able to read, edit, and execute files here.", "",
                 "⚠ This folder pre-approves 2 tool permissions", ""]
        lines += [("❯ " if index == focus else "  ") + option for index, option in enumerate(options)]
        lines += ["", "Enter to confirm · Esc to cancel"]
        sys.stdout.write("\x1b[2J\x1b[H" + "\r\n".join(lines))
        sys.stdout.flush()

    fd = sys.stdin.fileno()
    saved = termios.tcgetattr(fd)
    tty.setraw(fd)
    choice = None
    try:
        draw()
        while choice is None:
            if os.path.exists("@FLAG@"):
                os.remove("@FLAG@")
                log(flag="answer-yes")
                choice = "Yes, I trust this folder"
                break
            ready, _, _ = select.select([fd], [], [], 0.2)
            if not ready:
                continue
            data = os.read(fd, 64)
            at = 0
            while at < len(data) and choice is None:
                if data[at:at + 3] in (b"\x1b[A", b"\x1bOA"):
                    key, at = "up", at + 3
                    focus = max(0, focus - 1)
                elif data[at:at + 3] in (b"\x1b[B", b"\x1bOB"):
                    key, at = "down", at + 3
                    focus = min(len(options) - 1, focus + 1)
                elif data[at:at + 1] in (b"\r", b"\n"):
                    key, at = "enter", at + 1
                    choice = options[focus]
                elif data[at:at + 1] == b"\x1b":
                    key, at = "escape", at + 1
                    choice = "No, exit"
                else:
                    key, at = "other:" + repr(data[at:at + 1]), at + 1
                log(key=key)
            draw()
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, saved)
    log(choice="yes" if choice.startswith("Yes") else "no")
    if not choice.startswith("Yes"):
        sys.exit(1)
    with open("@RECORD@", "a", encoding="utf-8") as out:
        out.write(root + "\n")
    log(recorded=root)
    sys.stdout.write("\x1b[2J\x1b[H")
    sys.stdout.flush()
print("Claude Code (stand-in) past the question: "
      + " ".join("[" + argument + "]" for argument in ARGS), flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

# Merges the JSON object `$1` into `marley` in the profile copy's settings.
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, re, sys

path, changes = pathlib.Path(sys.argv[1]), json.loads(sys.argv[2])
text = path.read_text() if path.exists() else "{}"
text = re.sub(r"^\s*//.*$", "", text, flags=re.MULTILINE)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
settings.setdefault("marley", {}).update(changes)
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
  settle 3
}

# New Agent in Worktree's Claude Code from the project's +, with the prompt `$1`.
worktree_agent() {
  click "$MAIN_ROW_X" "$MAIN_ROW_Y"
  settle 1
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  local step
  for ((step = 0; step < WORKTREE_STEPS; step++)); do
    press "" Down
  done
  press "" Return
  settle 1
  click "$SUBMENU_CLAUDE_X" "$SUBMENU_CLAUDE_Y"
  settle 2
  type_text "$1"
  press "" Return
}

# What the `n`th launch of the fake logged, 1 first: its keys, its choice and whether it asked.
launch_report() {
  python3 - "$E2E_WORK/claude.log" "$1" <<'PY'
import json, sys

entries = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
pids = [entry["pid"] for entry in entries if "launch" in entry]
pid = pids[int(sys.argv[2]) - 1]
mine = [entry for entry in entries if entry["pid"] == pid]
keys = [entry["key"] for entry in mine if "key" in entry]
choice = next((entry["choice"] for entry in mine if "choice" in entry), "none")
flag = any("flag" in entry for entry in mine)
print(f"launch {sys.argv[2]}: keys {keys}, choice {choice}, flag {flag}")
PY
}

steps() {
  local report
  settle 12
  # Trusts the scratch repository in Zed.
  press "" Return
  settle 3

  echo "== the terminal's claude is the fake"
  type_text "command -v claude > $E2E_WORK/which.txt"
  press "" Return
  settle 2
  if ! grep -qx "$E2E_WORK/bin/claude" "$E2E_WORK/which.txt"; then
    echo "check the fake comes first on the terminal's PATH: FAIL (stopping before any launch)"
    return 1
  fi

  echo "== a first worktree agent: the question comes to the main checkout's workspace"
  worktree_agent "first"
  settle 8
  shot 587-01-asked

  echo "== Show Terminal: the agent's terminal with the question; answered there"
  click "$SHOW_X" "$SHOW_Y"
  settle 2
  shot 587-02-shown
  press "" Down
  press "" Return
  settle 2
  report=$(launch_report 1)
  echo "$report"
  expect "the first was answered with the scenario's keys in its terminal" test "$report" = "launch 1: keys ['down', 'enter'], choice yes, flag False"

  echo "== a second, and Trust Folder"
  : >"$E2E_WORK/claude-trust.txt"
  worktree_agent "second"
  settle 8
  click "$TRUST_X" "$TRUST_Y"
  settle 3
  shot 587-03-trusted
  report=$(launch_report 2)
  echo "$report"
  expect "Trust Folder moved the focus to the trust option and confirmed it" test "$report" = "launch 2: keys ['down', 'enter'], choice yes, flag False"

  echo "== a third, answered elsewhere: the notification goes, and Marley sends nothing"
  : >"$E2E_WORK/claude-trust.txt"
  worktree_agent "third"
  settle 8
  touch "$E2E_WORK/answer-yes"
  settle 3
  shot 587-04-answered-elsewhere
  report=$(launch_report 3)
  echo "$report"
  expect "the third got no key from Marley" test "$report" = "launch 3: keys [], choice yes, flag True"

  echo "== follow_zed: a fourth, answered by Marley, with a toast"
  marley_setting '{"claude_code_worktree_trust": "follow_zed"}'
  : >"$E2E_WORK/claude-trust.txt"
  worktree_agent "fourth"
  settle 8
  shot 587-05-followed-zed
  report=$(launch_report 4)
  echo "$report"
  expect "follow_zed picked the trust option without a click" test "$report" = "launch 4: keys ['down', 'enter'], choice yes, flag False"

  echo "== a fifth, in a repository the fake trusts already: no question, nothing sent"
  echo "the fake's record:"
  cat "$E2E_WORK/claude-trust.txt"
  worktree_agent "fifth"
  settle 8
  shot 587-06-no-question
  report=$(launch_report 5)
  echo "$report"
  echo "the fake's log:"
  cat "$E2E_WORK/claude.log"
  expect "the fifth was asked nothing and sent nothing" test "$report" = "launch 5: keys [], choice none, flag False"
}
