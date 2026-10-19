# shellcheck shell=bash
# #816's visual check: a command sent from the shell's prompt editor runs alone, whatever is typed
# right after it. One burst of keys, 20 ms apart, types `cd sub`, Return, `here`, Return; `here`
# logs the folder it runs in.
# - In bash: two blocks, `cd sub` and `here`, and the log names `sub` (`816-01-bash`, REQ-001).
# - In a zsh terminal, the same with `sub2` (`816-02-zsh`, REQ-002).
compositor sway

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo/sub" "$E2E_WORK/repo/sub2"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
here() {
  pwd >>"$E2E_WORK/where.log"
}
RC
  cat >"$home/.zshrc" <<RC
PROMPT='\$ '
here() {
  pwd >>"$E2E_WORK/where.log"
}
RC
  terminal_env HOME "$home"
  : >"$E2E_WORK/where.log"
  git init -q -b enter "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Types `$1`, Return, `$2`, Return in one burst, as a quick typist does.
burst() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -k Return "$2" -k Return
}

# Whether the newest `here` line is the repository's folder `$1`.
ran_in() {
  tail -1 "$E2E_WORK/where.log" | grep -q "/repo/$1\$"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== bash"
  burst "cd sub" "here"
  settle 3
  shot 816-01-bash
  cat "$E2E_WORK/where.log"
  expect "bash ran cd sub, then here, each alone" ran_in sub

  echo "== zsh"
  profile_setting terminal.shell '{"program": "zsh"}'
  settle 3
  palette "workspace: new terminal"
  settle 5
  burst "cd sub2" "here"
  settle 3
  shot 816-02-zsh
  cat "$E2E_WORK/where.log"
  expect "zsh ran cd sub2, then here, each alone" ran_in sub2
}
