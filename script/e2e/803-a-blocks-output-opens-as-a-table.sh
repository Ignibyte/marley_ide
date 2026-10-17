# shellcheck shell=bash
# #803's visual check: a finished block whose output reads as a table gets a Table chip, which
# opens the table in a tab beside the terminal. In a scratch repository:
# - `cat data.csv` (5 columns, 40 rows, a quoted field holding a comma): the chip
#   (`803-01-chip`, REQ-001); the chip clicked: a tab with the table, the quoted field in one cell
#   (`803-02-tab`, REQ-002); a column's sort clicked (`803-03-sorted`, REQ-002). Through a pipe,
#   `cat data.csv | cat`, the block's own text parsed by Marley (`803-02b-parsed`, REQ-002).
# - `python3 -c` printing CSV: the chip, and the block menu's Open Output as Table opens it
#   (`803-04a-menu`, `803-04-pipe`, REQ-001, REQ-003).
# - `ls -l`, `git log --oneline` and `ls --color=always`: no chip (`803-05-none`, REQ-004).
# - `head -n 5000 big.csv` (3 MB): the chip opens Zed's preview of the whole file, sorted by id
#   Z-A to show its last rows (`803-06-big`, REQ-005).
# - `cat data.jsonl` and `cat notes.md` (a markdown table among prose): each chip opens its table
#   (`803-07-json`, `803-08-markdown`, REQ-001).
compositor sway

# Where things are, as the first run's shots found them. After a `clear` the blocks stack from
# the bottom: the prompt's row is at y 920, rows are 19.5 px, and a block's chip sits on its first
# row at x 1538 (`chip_y` places it from the output's line count), or at its last row's right end,
# x 1565, y 891, when its first row is off screen. Then the terminal's tab, the Commits and id
# columns' sort buttons in a table tab (shown on hover), and a point on the pipe block's output
# for its menu with the menu's Open Output as Table.
CHIP_X=${CHIP_X:-1538}
LAST_ROW_CHIP_X=${LAST_ROW_CHIP_X:-1565}
LAST_ROW_CHIP_Y=${LAST_ROW_CHIP_Y:-891}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-400}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}
SORT_X=${SORT_X:-762}
SORT_Y=${SORT_Y:-86}
ID_SORT_X=${ID_SORT_X:-467}
BLOCK_X=${BLOCK_X:-700}
BLOCK_Y=${BLOCK_Y:-853}
MENU_ITEM_X=${MENU_ITEM_X:-778}
MENU_ITEM_Y=${MENU_ITEM_Y:-653}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  python3 - "$repo" <<'PY'
import json, pathlib, sys
repo = pathlib.Path(sys.argv[1])
teams = ["editor", "terminal", "browser", "agents"]
lines = ["Name,Team,Commits,Lines,Notes"]
for n in range(1, 41):
    notes = '"fixed the parser, again"' if n == 7 else "steady"
    lines.append(f"user{n:02d},{teams[n % 4]},{(n * 37) % 90 + 3},{n * 120},{notes}")
(repo / "data.csv").write_text("\n".join(lines) + "\n")
rows = ["id,host,region,latency_ms,status"]
for n in range(60000):
    rows.append(f"{n},host-{n % 500:03d}.example,region-{n % 7},{(n * 13) % 400},ok")
(repo / "big.csv").write_text("\n".join(rows) + "\n")
services = [("api", "up", 42), ("worker", "slow", 310), ("cron", "up", 8), ("search", "down", None)]
(repo / "data.jsonl").write_text("".join(
    json.dumps({"service": name, "state": state, "p95_ms": p95}) + "\n" for name, state, p95 in services))
(repo / "notes.md").write_text("""# Release notes

The table below lists what shipped this week.

| Area | Change | Owner |
|------|:-------|------:|
| Terminal | Tables from block output | Chad |
| Browser | Agent pages | Chad |
| Agents | Components in threads | Chad |

Thanks for reading.
""")
(repo / "README.md").write_text("# repo\n")
PY
  git -C "$repo" init -q -b main
  git -C "$repo" add README.md data.csv
  git -C "$repo" -c user.name=e2e -c user.email=e2e@example.invalid commit -q -m "Add the readme and the data"
  git -C "$repo" add notes.md
  git -C "$repo" -c user.name=e2e -c user.email=e2e@example.invalid commit -q -m "Add the release notes"
  open_path "$repo"
}

# Types a command at the terminal's prompt and runs it, in one `wtype`.
run_in_terminal() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 15 "$1" -s 300 -k Return
}

# Brings the terminal's tab to the front and clears its screen, so the next block starts on the
# top row.
fresh_terminal() {
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 1
  run_in_terminal "clear"
  settle 1
}

# The y of the chip of the newest block after a `clear`, its output `$1` lines long.
chip_y() {
  echo $(((1822 - 39 * ($1 + 2)) / 2))
}

# Clicks the point `$1`, `$2`: a block's Table chip.
open_chip() {
  pointer_to "$1" "$2"
  settle 1
  click "$1" "$2"
  settle 3
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== cat data.csv: the chip, the tab, sorted"
  run_in_terminal "clear"
  settle 1
  run_in_terminal "cat data.csv"
  settle 3
  pointer_to "$CHIP_X" "$(chip_y 41)"
  settle 1
  shot 803-01-chip
  open_chip "$CHIP_X" "$(chip_y 41)"
  shot 803-02-tab
  pointer_to "$SORT_X" "$SORT_Y"
  settle 1
  click "$SORT_X" "$SORT_Y"
  settle 2
  shot 803-03-sorted

  echo "== the same file through a pipe: the block's own text, parsed by Marley"
  fresh_terminal
  run_in_terminal "cat data.csv | cat"
  settle 3
  open_chip "$CHIP_X" "$(chip_y 41)"
  shot 803-02b-parsed

  echo "== a program's CSV on stdout, opened from the block menu"
  fresh_terminal
  run_in_terminal "python3 -c \"import csv, sys; out = csv.writer(sys.stdout); out.writerow(['city', 'temp', 'rain']); out.writerows([('Austin', 31, 0.1), ('Denver', 18, 0.0), ('Seattle', 12, 2.3), ('Boston', 15, 0.8)])\""
  settle 3
  click "$BLOCK_X" "$BLOCK_Y" right
  settle 2
  shot 803-04a-menu
  click "$MENU_ITEM_X" "$MENU_ITEM_Y"
  settle 3
  shot 803-04-pipe

  echo "== ls -l, git log and colored ls: no chip"
  fresh_terminal
  run_in_terminal "ls -l"
  settle 1
  run_in_terminal "git log --oneline"
  settle 1
  run_in_terminal "ls --color=always"
  settle 2
  shot 803-05-none

  echo "== head of a 3 MB file: the whole file's preview"
  fresh_terminal
  run_in_terminal "head -n 5000 big.csv"
  settle 3
  # The block's first row has scrolled away, so its chip is on its last row.
  pointer_to "$LAST_ROW_CHIP_X" "$LAST_ROW_CHIP_Y"
  settle 1
  shot 803-06a-big-chip
  open_chip "$LAST_ROW_CHIP_X" "$LAST_ROW_CHIP_Y"
  settle 3
  shot 803-06b-big-top
  # The id column sorted Z-A (two clicks) puts the file's last rows, far past the 5,000 the block
  # printed, on top.
  pointer_to "$ID_SORT_X" "$SORT_Y"
  settle 1
  click "$ID_SORT_X" "$SORT_Y"
  settle 3
  click "$ID_SORT_X" "$SORT_Y"
  settle 4
  shot 803-06-big

  echo "== JSON lines and a markdown table"
  fresh_terminal
  run_in_terminal "cat data.jsonl"
  settle 3
  open_chip "$CHIP_X" "$(chip_y 4)"
  shot 803-07-json
  fresh_terminal
  run_in_terminal "cat notes.md"
  settle 3
  open_chip "$CHIP_X" "$(chip_y 11)"
  shot 803-08-markdown
}
