# shellcheck shell=bash
# #485's e2e test. A two-line prompt whose second line is wider than the 100 columns Zed's
# terminal starts a PTY at. Readline lays the prompt out for the width it starts with, and a
# resize after that leaves it misdrawing the line (its multi-line redraw restores the old
# layout), so the shell has to start at the terminal's real width. A terminal opened while
# Marley runs does, from the size the last terminal view laid out; the first terminal of a
# launch, which starts before any view has a size, does not (shot 01).

setup() {
  mkdir -p "$E2E_WORK/home"
  # As starship draws it: a blank line, then 124 columns of colored dashes, a branch and "❯ ",
  # with the colors marked invisible; and a `bind` while bash starts, as Omarchy's rc runs one,
  # which sets up readline at the width the PTY has then.
  cat > "$E2E_WORK/home/.bashrc" <<'RC'
bind 'set show-all-if-ambiguous on'
dashes=$(printf '%.0s-' {1..124})
PS1="\n\[\e[1;36m\]$dashes\[\e[0m\] \[\e[3;36m\]probe\[\e[0m\] \[\e[1;36m\]❯\[\e[0m\] "
RC
  terminal_env HOME "$E2E_WORK/home"
  git init -q -b long-prompt "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "echo first"
  settle 1
  shot 485-01-first-terminal
  press "CTRL SHIFT" p
  settle 1
  type_text "new center terminal"
  settle 1
  press "" Return
  settle 4
  type_text "echo second"
  settle 1
  shot 485-02-second-terminal
  press "" Return
  settle 1
  shot 485-03-second-ran
}
