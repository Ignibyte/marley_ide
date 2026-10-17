# shellcheck shell=bash
# #804's visual check: a block of delimited text can show its table in the terminal, over its own
# output rows. In a scratch repository with `data.csv` (5 columns, 60 rows, a quoted field holding
# a comma), one record on each line:
# - `marley.block_tables` at `offer`: `cat data.csv` shows its text, with the Table chip and the
#   Inline toggle on its last row, its first row being off screen (`804-01-offer`, REQ-004).
# - Inline clicked: the table over the block's rows, the prompt under it where it was
#   (`804-02-toggle`, REQ-001); scrolled up to the block's start: the table moved with the rows,
#   its header row on the block's first output row (`804-03-scrolled`, REQ-002).
# - The block menu's Copy Output, pasted at the prompt: the CSV text (`804-04-copy`, REQ-003).
# - Text clicked: the text back (`804-05-text`, REQ-001).
# - `in_place` set, `cat data.csv` again: the table by itself (`804-06-in-place`, REQ-004); a
#   terminal search for `user50`: that row marked in the table (`804-07-find`, REQ-003).
compositor sway

# Where things are, as the first run's shots found them: the toggle beside the Table chip on the
# block's last row, a point on the table for the block's menu, and the menu's Copy Output.
INLINE_X=${INLINE_X:-1565}
INLINE_Y=${INLINE_Y:-891}
MENU_AT_X=${MENU_AT_X:-700}
MENU_AT_Y=${MENU_AT_Y:-300}
COPY_OUTPUT_X=${COPY_OUTPUT_X:-750}
COPY_OUTPUT_Y=${COPY_OUTPUT_Y:-738}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  python3 - "$repo" <<'PY'
import pathlib, sys
repo = pathlib.Path(sys.argv[1])
teams = ["editor", "terminal", "browser", "agents"]
lines = ["Name,Team,Commits,Lines,Notes"]
for n in range(1, 61):
    notes = '"fixed the parser, again"' if n == 7 else "steady"
    lines.append(f"user{n:02d},{teams[n % 4]},{(n * 37) % 90 + 3},{n * 120},{notes}")
(repo / "data.csv").write_text("\n".join(lines) + "\n")
(repo / "README.md").write_text("# repo\n")
PY
  profile_setting marley.block_tables '"offer"'
  open_path "$repo"
}

# Types a command at the terminal's prompt and runs it, in one `wtype`.
run_in_terminal() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 15 "$1" -s 300 -k Return
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== offer: the text, with the chip and the toggle"
  run_in_terminal "clear"
  settle 1
  run_in_terminal "cat data.csv"
  settle 3
  pointer_to "$INLINE_X" "$INLINE_Y"
  settle 1
  shot 804-01-offer

  echo "== Inline: the table over the rows, and scrolled"
  click "$INLINE_X" "$INLINE_Y"
  settle 2
  pointer_to 900 500
  settle 1
  shot 804-02-toggle
  scroll -8
  settle 1
  shot 804-03-scrolled
  scroll 8
  settle 1

  echo "== Copy Output from the block menu, pasted at the prompt"
  click "$MENU_AT_X" "$MENU_AT_Y" right
  settle 2
  shot 804-04a-menu
  click "$COPY_OUTPUT_X" "$COPY_OUTPUT_Y"
  settle 1
  press CTRL v
  settle 2
  shot 804-04-copy
  press CTRL a
  press "" BackSpace
  settle 1

  echo "== Text: the text back"
  pointer_to "$INLINE_X" "$INLINE_Y"
  settle 1
  click "$INLINE_X" "$INLINE_Y"
  settle 2
  pointer_to 900 500
  settle 1
  shot 804-05-text

  echo "== in_place: the table by itself, and a search"
  profile_setting marley.block_tables '"in_place"'
  settle 3
  run_in_terminal "clear"
  settle 1
  run_in_terminal "cat data.csv"
  settle 3
  pointer_to 900 500
  settle 1
  shot 804-06-in-place
  # The keys go to the shell's grid until the next prompt.
  press "" Escape
  settle 1
  press "CTRL SHIFT" f
  settle 1
  type_text "user50"
  settle 2
  pointer_to 900 500
  settle 1
  shot 804-07-find
  press "" Escape
}
