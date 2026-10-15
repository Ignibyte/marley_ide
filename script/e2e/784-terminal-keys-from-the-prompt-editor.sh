# shellcheck shell=bash
# #784's e2e test: the keys the terminal sends its program, and Zed's unscoped Ctrl-O and Ctrl-Q,
# stay in the shell's prompt editor and in an agent's rich input. The shell prints `LEAK <key>`
# for each of them it gets (`bind -x`), so a key that reached it shows on the grid. The stand-in
# Claude Code reads raw bytes and prints each one, so a key that reached it shows as a line of
# its own on the screen and in a file the checks read.
compositor sway

# The screen's rows, read through Marley's MCP server (#525).
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

screen() {
  mcp_agent terminal-screen "" >"$E2E_WORK/screen-$1.txt" || return 1
  cat "$E2E_WORK/screen-$1.txt"
}

# Whether the shell's screen shows no key that reached it.
no_leak() {
  screen "$1" || return 1
  ! grep -q 'LEAK' "$E2E_WORK/screen-$1.txt"
}

# Whether the stand-in got exactly the bytes named since it started, one `%q` per line, or none.
stand_in_got() {
  local got=$E2E_WORK/home/got.txt
  [[ -f $got ]] || return 1
  cat "$got"
  printf '%s\n' "$@" | sed '/^$/d' | diff - "$got"
}

# Whether the clipboard holds exactly the text named.
clipboard_is() {
  [[ $(WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste -n) == "$1" ]]
}

# Whether the run's Marley still has its window.
still_running() {
  [[ -n $(sway_marley_pid) ]]
}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
stty -ixon
leak() { printf 'LEAK %s\n' "$1"; }
bind -x '"\C-w": leak ctrl-w'
bind -x '"\C-e": leak ctrl-e'
bind -x '"\C-b": leak ctrl-b'
bind -x '"\C-r": leak ctrl-r'
bind -x '"\C-o": leak ctrl-o'
bind -x '"\C-q": leak ctrl-q'
bind -x '"\C-v": leak ctrl-v'
bind -x '"\eb": leak alt-b'
bind -x '"\ef": leak alt-f'
stand_in() {
  exec -a claude bash -c 'stty -icanon -echo -isig -ixon -iexten; : >"$HOME/got.txt"; printf "ready\n"; while IFS= read -r -n1 -d "" c; do printf "%q\n" "$c" >>"$HOME/got.txt"; printf "claude got: %q\n" "$c"; done'
}
RC
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # A 1x1 PNG, for an image on the clipboard.
  printf '\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0\x1f\x15\xc4\x89\0\0\0\rIDATx\x9cc\xf8\xff\xff?\0\x05\xfe\x02\xfe\xa7\x35\x81\x84\0\0\0\0IEND\xaeB`\x82' >"$E2E_WORK/dot.png"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  # The terminal takes the focus, and the prompt editor docks.
  click 700 500
  settle 1.5

  echo "== readline's moves in the shell's editor"
  type_text "one two"
  press "" Home
  press CTRL e
  type_text "X"
  press CTRL b
  type_text "Y"
  press ALT b
  type_text "Z"
  press ALT f
  type_text "W"
  settle 1
  shot 784-01-moves
  expect "no move reached the shell" no_leak moves

  echo "== Ctrl-W deletes the word before the cursor"
  press CTRL w
  settle 1
  shot 784-02-ctrl-w
  expect "Ctrl-W stayed in the editor" no_leak ctrl-w

  echo "== Ctrl-R, Ctrl-O and Ctrl-Q do nothing"
  press CTRL r
  press CTRL o
  settle 1
  press CTRL q
  settle 2
  expect "Marley still runs after Ctrl-Q" still_running
  shot 784-03-nothing
  expect "none of them reached the shell" no_leak nothing

  echo "== Ctrl-Shift-V pastes into the editor"
  printf 'PASTED' | WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy
  press "CTRL SHIFT" v
  settle 1
  shot 784-04-paste
  expect "the paste stayed in the editor" no_leak paste

  echo "== Ctrl-Shift-C copies the editor's selection"
  printf 'BEFORE' | WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy
  press CTRL a
  press "CTRL SHIFT" c
  settle 1
  expect "the clipboard holds the editor's text" clipboard_is "one PASTED"

  echo "== an image goes nowhere from the shell's editor"
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy --type image/png <"$E2E_WORK/dot.png"
  press "" End
  press "CTRL SHIFT" v
  settle 1
  shot 784-05-shell-image
  expect "the image did not reach the shell" no_leak shell-image

  echo "== the same keys in the agent's rich input"
  press CTRL c
  type_text "stand_in"
  press "" Return
  settle 4
  press CTRL g
  settle 1
  type_text "ab cd"
  press CTRL w
  press CTRL e
  press CTRL b
  press ALT b
  press ALT f
  press CTRL r
  press CTRL o
  settle 1
  printf 'EF' | WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy
  press "CTRL SHIFT" v
  settle 1
  shot 784-06-agent
  expect "the stand-in got nothing" stand_in_got

  echo "== an image goes to the agent as Ctrl-V"
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy --type image/png <"$E2E_WORK/dot.png"
  press "CTRL SHIFT" v
  settle 1
  shot 784-07-image
  expect "the stand-in got Ctrl-V" stand_in_got "\$'\\026'"

  echo "== Enter sends exactly the editor's text"
  press "" Return
  settle 2
  shot 784-07-sent
  # `ab cd`: Ctrl-W leaves `ab `, Ctrl-E, Ctrl-B, Alt-B and Alt-F put the cursor after `ab`, and
  # the paste puts `EF` there. Bash's `read -n` turns the tty's CR-to-NL translation back on, so
  # the return Marley sends arrives as a newline.
  expect "the stand-in got the editor's text and a return" \
    stand_in_got "\$'\\026'" a b E F '\ ' "\$'\\n'"

  echo "== Ctrl-Q in the agent's rich input"
  press CTRL g
  settle 1
  press CTRL q
  settle 2
  expect "Marley still runs after Ctrl-Q in the rich input" still_running
  shot 784-08-still-running
}
