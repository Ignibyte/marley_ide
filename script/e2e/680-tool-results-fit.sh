# shellcheck shell=bash
# #680's visual check: tool results an agent reads in pages that fit Claude Code's limit. A
# stand-in agent reaches Marley's MCP server through the Claude Code plugin's bridge, as Claude
# Code in a Marley terminal does. It reads the instructions `initialize` gives (REQ-008), runs a
# script that prints a fake AWS key and `seq 1 8000` with terminal_run, whose answer is the newest
# page (REQ-007), then reads that block page by page from the newest to the first (REQ-001 to
# REQ-003, REQ-005, REQ-006, REQ-013), reads a 40,000-character line (REQ-004), and makes the
# terminal tools refuse with their codes (REQ-009 to REQ-012).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# A click in the terminal, as #556's scenario makes it.
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  export MCP_CLIENT_NAME="Stand-in agent"
  # A command on neither of the user's lists runs at once, the default, whatever the copied
  # profile says.
  profile_setting marley.agent_commands_outside_lists '"run"'
  git init -q -b main "$repo"
  # The fake key is put together here, so no file in the tree holds one.
  python3 - "$repo" <<'PY'
import pathlib, sys

repo = pathlib.Path(sys.argv[1])
aws = "AK" + "IA" + "FAKE" * 4
(repo / "pages.sh").write_text(f"#!/bin/sh\nprintf 'aws: %s\\n' '{aws}'\nseq 1 8000\n")
(repo / "long.sh").write_text("#!/bin/sh\nhead -c 40000 /dev/zero | tr '\\0' x\necho\n")
for name in ("pages.sh", "long.sh"):
    (repo / name).chmod(0o755)
PY
  open_path "$repo"
}

# `numbers <file> <word>`: each number the stand-in printed after the word, one a line.
numbers() {
  grep -oP "$2 \K[0-9]+" "$1"
}

# `all_at_most <max> <file> <word>`: whether every number after the word is at most max, and
# there is one.
all_at_most() {
  local max=$1 file=$2 word=$3 number found=0
  while read -r number; do
    found=1
    ((number <= max)) || return 1
  done < <(numbers "$file" "$word")
  ((found))
}

# Whether the joined pages are the script's output, the key redacted: line 1 the masked key,
# then 1 to 8000, nothing repeated or skipped.
joined_is_the_output() {
  python3 - "$1" <<'PY'
import sys

lines = open(sys.argv[1]).read().split("\n")
if lines and lines[-1] == "":
    lines.pop()
first = lines[0] if lines else ""
masked = first.startswith("aws: [redacted:") and "FAKE" not in first
counted = lines[1:] == [str(number) for number in range(1, 8001)]
print(f"  first line {first!r}; {len(lines)} lines; the numbers in order: {counted}")
sys.exit(0 if masked and counted else 1)
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== initialize's instructions"
  mcp_agent instructions | tee "$E2E_WORK/instructions.txt"
  expect "they are at most 2,048 bytes" all_at_most 2048 "$E2E_WORK/instructions.txt" "instructions:"
  expect "they name only Marley's tools" holds "$E2E_WORK/instructions.txt" \
    "named but not listed: none" "terminal_read" "browser_tabs"

  echo "== terminal_run's answer is the newest page"
  # Run as programs: `bash` and `sh` are on the default denylist, which asks the user.
  mcp_agent terminal-run repo "./pages.sh" | tee "$E2E_WORK/run.txt"
  expect "it ran" holds "$E2E_WORK/run.txt" "'./pages.sh', exit 0" "previous " "of 8001"
  expect "its output fits a page" all_at_most 12000 "$E2E_WORK/run.txt" "output"
  mcp_agent terminal-run repo "./long.sh" | tee "$E2E_WORK/run-long.txt"
  expect "the long line ran" holds "$E2E_WORK/run-long.txt" "'./long.sh', exit 0"
  settle 1
  shot 680-01-blocks

  echo "== the newest page of the block"
  mcp_agent terminal-page pages.sh | tee "$E2E_WORK/newest.txt"
  expect "it ends at the block's last line" holds "$E2E_WORK/newest.txt" "of 8001" \
    "output ends: '8000'"
  expect "it fits a page" all_at_most 12000 "$E2E_WORK/newest.txt" "output"
  expect "its text says how to read the rest" holds "$E2E_WORK/newest.txt" \
    "text starts: '[lines" "read earlier lines with terminal_read before="
  expect "its structured output is the plain page" grep -qP "output starts: '[0-9]+'" \
    "$E2E_WORK/newest.txt"

  echo "== every page, newest to first"
  mcp_agent terminal-pages pages.sh "$E2E_WORK/joined.txt" | tee "$E2E_WORK/pages.txt"
  expect "the pages join into the output" joined_is_the_output "$E2E_WORK/joined.txt"
  expect "the walk ends at line 1 with no previous" grep -qP "page 1-[0-9]+ of 8001: previous None" \
    "$E2E_WORK/pages.txt"
  expect "every page fits" all_at_most 12000 "$E2E_WORK/pages.txt" "output"
  expect "every answer is under 40,000 bytes on the wire" all_at_most 39999 "$E2E_WORK/pages.txt" \
    "result"
  expect "every page counts the one secret" all_at_most 1 "$E2E_WORK/pages.txt" "redacted"
  expect "and none counts none" bash -c "! grep -q 'redacted 0' '$E2E_WORK/pages.txt'"

  echo "== a line longer than a page"
  mcp_agent terminal-page long.sh | tee "$E2E_WORK/long.txt"
  expect "it is one line with its end kept" holds "$E2E_WORK/long.txt" "page 1-1 of 1" \
    "line_cut True" "truncated True"
  expect "and fits a page" all_at_most 12000 "$E2E_WORK/long.txt" "output"

  echo "== refusals"
  mcp_agent refusal terminal_read '{"terminal": 999999, "block": 0}' | tee "$E2E_WORK/refused.txt"
  expect "an unknown terminal is no_terminal" holds "$E2E_WORK/refused.txt" "code no_terminal" \
    "next: terminal_list lists the terminals" "next: the terminals now:"
  mcp_agent read-refusal pages.sh '{"block": 999}' | tee "$E2E_WORK/refused-block.txt"
  expect "an unknown block is no_block" holds "$E2E_WORK/refused-block.txt" "code no_block" \
    "next: terminal_blocks lists terminal"
  mcp_agent read-refusal pages.sh '{"before": 0}' | tee "$E2E_WORK/refused-before.txt"
  expect "a before outside the output is bad_argument" holds "$E2E_WORK/refused-before.txt" \
    "code bad_argument" "next: pass a page's \`previous\` as \`before\`"
  local terminal
  terminal=$(mcp_agent terminals | grep -oP "terminal \K[0-9]+" | head -1)
  mcp_agent refusal terminal_run "{\"terminal\": $terminal, \"command\": \" \"}" \
    | tee "$E2E_WORK/refused-words.txt"
  expect "a refusal in words alone is refused" holds "$E2E_WORK/refused-words.txt" \
    "code refused" "give \`command\`"
  settle 1
  shot 680-02-after
}
