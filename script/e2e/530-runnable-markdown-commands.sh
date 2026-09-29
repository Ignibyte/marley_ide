# shellcheck shell=bash
# #530's visual check: a shell code block in the Markdown preview puts its command at the prompt
# of the terminal focused last, unrun. RUNBOOK.md in a scratch repository holds a `bash` block, a
# bare fence, a two-line `bash` block, a `rust` block and an indented block. A shell block shows
# Insert in Terminal beside Copy (REQ-001) and the others do not (REQ-002); a click clears the
# prompt's line, puts the command there and brings the terminal forward with the focus (REQ-003),
# and Enter runs it (REQ-004); two lines go in as one bracketed paste (REQ-005); a running program
# (REQ-006), bracketed paste turned off (REQ-007) and a closed terminal (REQ-008) each type
# nothing and say why in a toast.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The terminal's pane on the left after the split, and in the preview on the right each block's
# middle and its Insert in Terminal button (first in the hover row, before Copy), from the shots.
TERMINAL_X=${TERMINAL_X:-500}
TERMINAL_Y=${TERMINAL_Y:-500}
BLOCK_X=${BLOCK_X:-1000}
BUTTON_X=${BUTTON_X:-1304}
BASH_Y=${BASH_Y:-169}
BASH_BUTTON_Y=${BASH_BUTTON_Y:-159}
BARE_Y=${BARE_Y:-237}
TWO_LINES_Y=${TWO_LINES_Y:-318}
TWO_LINES_BUTTON_Y=${TWO_LINES_BUTTON_Y:-296}
RUST_Y=${RUST_Y:-399}
INDENTED_Y=${INDENTED_Y:-467}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  cat >"$repo/RUNBOOK.md" <<'MD'
# Runbook

```bash
echo hello from the runbook
```

```
ls
```

```bash
echo first line
echo second line
```

```rust
fn main() {}
```

    echo indented block
MD
  git init -q -b main "$repo"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Hovers the block whose middle is at `$1` and clicks its Insert in Terminal button at `$2`.
insert_from() {
  pointer_to "$BLOCK_X" "$1"
  settle 1
  click "$BUTTON_X" "$2"
  settle 2
}

screen() {
  mcp_agent terminal-screen repo | tee "$E2E_WORK/$1.txt"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the runbook in the preview beside the terminal"
  press CTRL p
  settle 1
  type_text "RUNBOOK"
  settle 2
  press "" Return
  settle 2
  palette "markdown: open preview to the side"
  settle 3
  # The runbook took the terminal's place in the left pane; the terminal's tab comes back.
  press ALT 1
  settle 1
  shot 530-00-preview

  echo "== a shell block shows Insert in Terminal; other blocks do not"
  pointer_to "$BLOCK_X" "$BASH_Y"
  settle 1
  shot 530-01-button
  pointer_to "$BLOCK_X" "$BARE_Y"
  settle 1
  shot 530-01b-bare-fence
  pointer_to "$BLOCK_X" "$RUST_Y"
  settle 1
  shot 530-02-no-button
  pointer_to "$BLOCK_X" "$INDENTED_Y"
  settle 1
  shot 530-02b-indented

  echo "== a click puts the command at the prompt, unrun, and focuses the terminal"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "abc"
  settle 1
  insert_from "$BASH_Y" "$BASH_BUTTON_Y"
  shot 530-03-inserted
  screen inserted
  expect "the prompt holds the command alone" \
    bash -c "grep -q '\\\$ echo hello from the runbook *\$' '$E2E_WORK/inserted.txt' && ! grep -q 'abc' '$E2E_WORK/inserted.txt'"
  press "" Return
  settle 1
  shot 530-04-ran

  echo "== two lines go in as one paste"
  insert_from "$TWO_LINES_Y" "$TWO_LINES_BUTTON_Y"
  shot 530-05-two-lines
  press CTRL c
  settle 1

  echo "== a running program: nothing typed, a toast"
  type_text "sleep 30"
  press "" Return
  settle 1
  insert_from "$BASH_Y" "$BASH_BUTTON_Y"
  shot 530-06-busy
  click "$TERMINAL_X" "$TERMINAL_Y"
  press CTRL c
  settle 1

  echo "== bracketed paste off: two lines typed nowhere, a toast"
  type_text "bind 'set enable-bracketed-paste off'"
  press "" Return
  settle 1
  insert_from "$TWO_LINES_Y" "$TWO_LINES_BUTTON_Y"
  shot 530-07-no-bracketed-paste
  screen no-paste
  expect "the prompt line holds nothing" \
    bash -c "tail -1 '$E2E_WORK/no-paste.txt' | grep -qE '^  \| \\\$ *\$'"

  echo "== the terminal closed: a toast"
  click "$TERMINAL_X" "$TERMINAL_Y"
  type_text "exit"
  press "" Return
  settle 2
  insert_from "$BASH_Y" "$BASH_BUTTON_Y"
  shot 530-08-no-terminal
}
