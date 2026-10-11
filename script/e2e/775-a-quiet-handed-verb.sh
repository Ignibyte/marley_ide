# shellcheck shell=bash
# #775's visual check: a handed verb prints one line. A scratch repo; `$E2E_WORK/bin/marley` is the
# debug binary, first on the terminals' PATH (#749).
# - `marley home`, its output kept, in the repo's terminal: the Home tab opens; back in the
#   terminal, the kept output is one line, "Marley is already running on …; it was asked to run
#   `marley home`." (`quiet`, REQ-001)
# - Marley quit; `marley home` launched: Marley starts with its session and the Home tab (`cold`,
#   REQ-002).
compositor sway

# The repo's terminal row in the rail, as #749's shots found it.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-95}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-176}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  printf '# repo\n' >"$repo/README.md"
  ln -s "$(marley_binary)" "$bin/marley"
  terminal_env HOME "$home"
  terminal_env PATH "$bin:$PATH"
  open_path "$repo"
}

# Types `$1` into the repo's terminal, which the rail's row brings forward, and runs it.
run_in_terminal() {
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  type_text "$1"
  press "" Return
}

# Whether the hand-off printed one line, naming the verb.
one_line() {
  local out=$E2E_WORK/out.txt
  echo "the launch printed: $(cat "$out")"
  [[ $(wc -l <"$out") -eq 1 ]] && grep -q "asked to run .marley home." "$out"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== marley home, its output kept"
  run_in_terminal "clear; marley home > $E2E_WORK/out.txt 2>&1"
  settle 3
  run_in_terminal "cat $E2E_WORK/out.txt"
  settle 2
  shot 775-01-quiet
  expect "the handed verb printed one line" one_line

  echo "== a cold marley home"
  quit_marley
  launch_marley_with home
  settle 15
  shot 775-02-cold
}
