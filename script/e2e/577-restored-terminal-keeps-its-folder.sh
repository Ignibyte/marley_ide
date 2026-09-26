# shellcheck shell=bash
# #577's e2e test: a restored center terminal opens in the folder it was in. The repository's
# terminal goes into `alpha`, a split of it into `beta`, and each prints where it is (`here`); a
# third terminal opens and closes. After a quit and a launch, each restored terminal prints the
# folder it was in, and Marley's tools list two terminals: the closed one stays closed.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# A point in the left and the right pane of the split.
LEFT_X=500
LEFT_Y=500
RIGHT_X=1100
RIGHT_Y=500

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
here() { echo "\$1 at \$(pwd)" | tee -a "$E2E_WORK/here.log"; }
RC
  terminal_env HOME "$home"
  : >"$E2E_WORK/here.log"
  git init -q -b folders "$repo"
  mkdir -p "$repo/alpha" "$repo/beta"
  open_path "$repo"
}

# The folder on the newest `here` line of label `$1`.
folder_of() {
  grep -E "^$1 at " "$E2E_WORK/here.log" | tail -1 | sed -E 's/^[^ ]* at //' || true
}

# Whether label `$1` printed the folder `$2` under the repository.
at_folder() {
  local found
  found=$(folder_of "$1")
  echo "$1 at $found"
  [[ $found == "$E2E_WORK/repo/$2" ]]
}

# Types a command into the pane at x `$1`, y `$2`.
run_at() {
  click "$1" "$2"
  settle 1
  type_text "$3"
  press "" Return
  settle 2
}

# Whether Marley's MCP server lists `$1` terminals.
terminal_count() {
  mcp_agent terminals | tee "$E2E_WORK/terminals.txt"
  [[ $(grep -c "^  terminal " "$E2E_WORK/terminals.txt" || true) -eq $1 ]]
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "cd alpha && here left-0"
  press "" Return
  settle 2
  palette "pane: split right"
  settle 4
  type_text "cd ../beta && here right-0"
  press "" Return
  settle 2
  echo "== a third terminal, opened and closed"
  palette "workspace: new terminal"
  settle 4
  type_text "here third"
  press "" Return
  settle 2
  press "CTRL SHIFT" w
  settle 2
  shot 577-01-before
  cat "$E2E_WORK/here.log"
  expect "the left terminal is in alpha" at_folder left-0 alpha
  expect "the right terminal is in beta" at_folder right-0 beta
  echo "== a quit and a launch"
  quit_marley
  launch_marley
  settle 12
  run_at "$LEFT_X" "$LEFT_Y" "here left-1"
  run_at "$RIGHT_X" "$RIGHT_Y" "here right-1"
  shot 577-02-restored
  expect "the left terminal came back in alpha" at_folder left-1 alpha
  expect "the right terminal came back in beta" at_folder right-1 beta
  expect "the closed terminal stayed closed" terminal_count 2
}
