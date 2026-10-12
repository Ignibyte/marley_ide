# shellcheck shell=bash
# #728's visual check: a plan's questions as cards in #726's Plan tab, with the plugin's part
# played by a stand-in. `marley.plan_review` is on in the run's settings. In the project's terminal
# a stand-in `claude` reports idle through `"$MARLEY_BIN" report`, then runs `"$MARLEY_BIN" approve
# --tool ExitPlanMode` with a plan holding four questions: one choice with context and
# `Recommended:`, several choices, free text, and one choice more. After a deny it sends a revised
# plan with a settled question, an example in a code block, and a block left unclosed. Shots: the
# four cards in their places, Open, Accept recommended on the first, "0 of 4 answered"
# (`728-01-cards`, REQ-001, REQ-002); Accept recommended on the first, two choices ticked on the
# second, an answer typed on the third (with a `c` in it, which the field keeps), Skip on the
# fourth: their states, "3 of 4 answered", Send Answers in Approve's place (`728-02-answered`,
# REQ-002, REQ-003, REQ-004); Send Answers and the deny's reason in the terminal (`728-03-sent`,
# REQ-005); the revised plan, its question Settled, the example as code and the unclosed block as
# text (`728-04-revised`, REQ-006, REQ-007).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
ACCEPT_X=${ACCEPT_X:-1083}
ACCEPT_Y=${ACCEPT_Y:-195}
FILES_X=${FILES_X:-299}
FILES_Y=${FILES_Y:-473}
TEXT_X=${TEXT_X:-299}
TEXT_Y=${TEXT_Y:-529}
WORDS_X=${WORDS_X:-700}
WORDS_Y=${WORDS_Y:-700}
SKIP_X=${SKIP_X:-1170}
SKIP_Y=${SKIP_Y:-769}
SEND_X=${SEND_X:-1270}
SEND_Y=${SEND_Y:-301}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-470}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}
PLAN_TAB_X=${PLAN_TAB_X:-676}
PLAN_TAB_Y=${PLAN_TAB_Y:-51}
AWAY_X=${AWAY_X:-1700}
AWAY_Y=${AWAY_Y:-1000}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  profile_setting marley.plan_review true
  write_fake_claude "$bin/claude"
  : >"$E2E_WORK/answers.log"
  open_path "$repo"
}

# The plugin's part: a report, then an approve with the plan; a revised plan after a deny.
write_fake_claude() {
  sed -e "s|@LOG@|$E2E_WORK/answers.log|" >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

BIN = os.environ.get("MARLEY_BIN", "")
LOG = "@LOG@"
FIRST = """# Cache the search

1. Put a cache in front of the index lookups.

:::question
Where should the cache live?
The search reads it on every keystroke.
- [ ] In memory — fastest, lost on restart
- [ ] On disk - survives a restart
Recommended: In memory
:::

:::question-multi
Which searches should use it?
- [ ] File names
- [ ] Symbols
- [ ] Text
:::

:::question-text
What should the cache be called?
:::

:::question
Should old entries expire?
- [ ] After an hour
- [ ] Never
:::

2. Run the search tests.
"""
REVISED = """# Cache the search, revised

:::question
Where should the cache live?
- [x] In memory — fastest, lost on restart
- [ ] On disk - survives a restart
:::

1. Add `search_cache` in memory for file names and text.
2. Run the search tests.

An example in a code block stays code:

```markdown
:::question
Not a card.
:::
```

:::question
This block has no closing line, so it stays text.
"""

subprocess.run([BIN, "report", "--source", "mod:claude-code", "--seq", "1", "idle"], check=False)


def ask(call, plan):
    done = subprocess.run(
        [BIN, "approve", "--source", "mod:claude-code", "--call", call, "--tool", "ExitPlanMode"],
        input=json.dumps({"plan": plan}), capture_output=True, text=True, check=False)
    line = f"{call}: exit {done.returncode} {done.stdout.strip()} {done.stderr.strip()}"
    print(line, flush=True)
    with open(LOG, "a", encoding="utf-8") as log:
        log.write(line + "\n")
    return done


if '"deny"' in ask("toolu_1", FIRST).stdout:
    ask("toolu_2", REVISED)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the stand-in Claude Code asks with a plan holding four questions"
  type_text "claude"
  press "" Return
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 728-01-cards || return 0
  if ((ACCEPT_X == 0)); then
    echo "the cards' points are not placed yet: 728-01 is the last shot"
    return 0
  fi

  echo "== Accept recommended, two choices, an answer typed, a skip"
  click "$ACCEPT_X" "$ACCEPT_Y"
  settle 1
  click "$FILES_X" "$FILES_Y"
  settle 1
  click "$TEXT_X" "$TEXT_Y"
  settle 1
  click "$WORDS_X" "$WORDS_Y"
  settle 1
  type_text "search_cache"
  settle 1
  click "$SKIP_X" "$SKIP_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 728-02-answered || return 0

  echo "== Send Answers: a deny with the answers first"
  click "$SEND_X" "$SEND_Y"
  settle 4
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 728-03-sent || return 0
  cat "$E2E_WORK/answers.log"
  expect "the plan was denied with the answers in their form" \
    holds "$E2E_WORK/answers.log" 'toolu_1: exit 0 {"decision": "deny"' \
    'The user answered your questions.' 'Answers to your questions' \
    '3 of 4 questions answered.' 'Answer: In memory (your recommendation)' \
    '- File names' '- Text' '> search_cache' 'Should old entries expire?\nSkipped'

  echo "== the revised plan: a settled question, an example as code, an unclosed block as text"
  click "$PLAN_TAB_X" "$PLAN_TAB_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 728-04-revised || return 0
}
