# shellcheck shell=bash
# #881's visual check: plan review's question cards and answers as the Review board draws them.
# #728's stand-in `claude` asks with a plan holding four questions, the first with a recommended
# choice; `marley.plan_review` is on in the run's settings.
# - The cards: the Recommended chip on the first's recommended choice, Accept recommended and Skip
#   at each card's foot; Approve and Send Feedback in the column (`881-01-cards`, REQ-001).
# - Marley Day (`881-02-day`, REQ-001).
compositor sway

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
  pointer_to 1400 700
  settle 1
  shot 881-01-cards

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 881-02-day
}
