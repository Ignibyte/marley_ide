# shellcheck shell=bash
# #630's visual check: block headers with the folder and branch, on by default (the scenario sets
# nothing). A git repository on `main` under the shell's HOME, so its folder reads `~/repo`; bash
# starts with a two-line PS1, `[the prompt]` over `$ `.
#
# Over two prompt rows each header shows `~/repo · main` over the command (`two-rows`, REQ-001,
# REQ-003, REQ-004); after `PS1='$ '` a one-row header shows the command, then the folder and
# branch (`one-row`, REQ-001); after a checkout the earlier headers keep `main` and the next one
# reads `other` (`checkout`, REQ-002).
compositor sway

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home/repo"
  cat >"$home/.bashrc" <<'RC'
PS1='[the prompt]\n$ '
RC
  terminal_env HOME "$home"
  printf '# repo\n' >"$home/repo/README.md"
  git -C "$home/repo" init -q -b main
  git -C "$home/repo" add README.md
  git -C "$home/repo" -c user.name=marley -c user.email=marley@example.invalid commit -q -m first
  open_path "$home/repo"
}

# Runs `$1` at the prompt, in the prompt editor.
run() {
  type_text "$1"
  press "" Return
  settle 1.5
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 2

  echo "== two prompt rows"
  run "echo hi"
  run "ls"
  pointer_to 700 300
  settle 1
  shot two-rows

  echo "== one prompt row"
  run "PS1='\$ '"
  run "echo one"
  pointer_to 700 300
  settle 1
  shot one-row

  echo "== a checkout"
  run "git checkout -q -b other"
  settle 2
  run "echo after"
  pointer_to 700 300
  settle 1
  shot checkout
}
