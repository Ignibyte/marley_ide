# shellcheck shell=bash
# #528's visual check: a block's output filtered in a panel over the terminal. A long `seq`, then a
# log of 40 entries in mixed case; Alt+Shift+F opens the filter on the newest block (REQ-001), a
# typed query keeps the lines holding it in any case (REQ-003), case, regex, an invalid regex,
# invert and context each change the list as grep would (REQ-004 to REQ-007); Escape closes it with
# nothing typed at the prompt (REQ-008, REQ-009), and it opens again as it was left (REQ-010). A
# short block's Filter button filters that block (REQ-002), and a running loop's list takes in its
# new lines (REQ-011).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# The panel's controls row and its toggles, and a short block's first row with its Filter button,
# from the shots.
CONTROLS_Y=${CONTROLS_Y:-122}
QUERY_X=${QUERY_X:-600}
CASE_X=${CASE_X:-1171}
REGEX_X=${REGEX_X:-1197}
INVERT_X=${INVERT_X:-1232}
CONTEXT_X=${CONTEXT_X:-1325}
SHORT_ROW_Y=${SHORT_ROW_Y:-877}
FILTER_BUTTON_X=${FILTER_BUTTON_X:-1262}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo number
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  for ((number = 1; number <= 40; number++)); do
    case $((number % 8)) in
      0) echo "ERROR request $number failed" ;;
      3) echo "error: retry $number" ;;
      5) echo "WARN disk 9$((number % 10))%" ;;
      6) echo "warn cache $number cold" ;;
      *) echo "INFO step $number done" ;;
    esac
  done >"$repo/log.txt"
  git init -q -b main "$repo"
  open_path "$repo"
}

# Puts `$1` in the focused field in place of what it held.
retype() {
  press CTRL a
  type_text "$1"
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "seq 1 300"
  press "" Return
  settle 1
  type_text "cat log.txt"
  press "" Return
  settle 2

  echo "== Alt+Shift+F opens the filter on the newest block"
  press "ALT SHIFT" f
  settle 2
  shot 528-01-open
  type_text "error"
  settle 2
  shot 528-02-text
  click "$CASE_X" "$CONTROLS_Y"
  settle 2
  shot 528-03-case
  click "$CASE_X" "$CONTROLS_Y"
  click "$REGEX_X" "$CONTROLS_Y"
  click "$QUERY_X" "$CONTROLS_Y"
  retype '^WARN .* 9[0-9]%$'
  settle 1
  shot 528-04-regex
  retype '('
  shot 528-05-invalid
  retype 'WARN'
  click "$REGEX_X" "$CONTROLS_Y"
  click "$INVERT_X" "$CONTROLS_Y"
  settle 2
  shot 528-06-invert
  click "$INVERT_X" "$CONTROLS_Y"
  click "$CONTEXT_X" "$CONTROLS_Y"
  retype 1
  settle 1
  shot 528-07-context

  echo "== Escape closes it, with nothing typed at the prompt"
  press "" Escape
  settle 2
  shot 528-08-closed
  mcp_agent terminal-screen repo | tee "$E2E_WORK/closed.txt"
  expect "the prompt line holds nothing the panel took" \
    bash -c "tail -1 '$E2E_WORK/closed.txt' | grep -qE '^  \| \\\$ *\$'"
  press "ALT SHIFT" f
  settle 2
  shot 528-09-reopened
  press "" Escape
  settle 1

  echo "== a short block's Filter button"
  type_text "printf 'alpha\nbeta\ngamma\n'"
  press "" Return
  settle 2
  pointer_to "$FILTER_BUTTON_X" "$SHORT_ROW_Y"
  settle 1
  shot 528-10a-hover
  click "$FILTER_BUTTON_X" "$SHORT_ROW_Y"
  settle 2
  shot 528-10-button
  press "" Escape
  settle 1

  echo "== a running block's list follows its output"
  type_text "for i in \$(seq 1 60); do echo line \$i; sleep 0.2; done"
  press "" Return
  settle 1
  press "ALT SHIFT" f
  settle 1
  retype 5
  click "$CONTEXT_X" "$CONTROLS_Y"
  retype 0
  settle 1
  shot 528-11-running-a
  settle 5
  shot 528-11-running-b
  press "" Escape
  settle 1
}
