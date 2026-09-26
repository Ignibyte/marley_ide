# shellcheck shell=bash
# #546's e2e test: block reads while a full-screen program shows. `seq 1 3` runs, then `less`
# opens a 200-line file on the alternate screen and pages down. The stand-in agent reads the
# `seq 1 3` block before `less`, while it shows, and after it quits: the three reads must be the
# same (REQ-001), and while `less` shows the block's output must be listed as kept (REQ-002).
# `LESS=` clears any options from the environment, so `less` takes the alternate screen.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  echo "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b altscreen "$E2E_WORK/repo"
  seq -f 'notes line %03g' 1 200 >"$E2E_WORK/repo/notes.txt"
  open_path "$E2E_WORK/repo"
}

# Reads `seq 1 3`'s block into $E2E_WORK/<label>.txt, through the stand-in agent.
read_seq() {
  mcp_agent terminal-read "seq 1 3" >"$E2E_WORK/$1.txt"
  cat "$E2E_WORK/$1.txt"
}

# Whether two reads are the same text.
same() {
  cmp -s "$E2E_WORK/$1.txt" "$E2E_WORK/$2.txt" || {
    diff "$E2E_WORK/$1.txt" "$E2E_WORK/$2.txt" || true
    return 1
  }
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "seq 1 3"
  press "" Return
  settle 1
  read_seq before
  echo "== less holds the alternate screen"
  type_text "LESS= less notes.txt"
  press "" Return
  settle 2
  # A page down, so the alternate screen scrolls under less.
  press "" space
  settle 1
  shot 546-01-less-open
  read_seq during
  mcp_agent blocks | tee "$E2E_WORK/blocks.txt"
  expect "the block reads the same while less shows" same before during
  expect "its output is kept while less shows" \
    holds "$E2E_WORK/blocks.txt" "'seq 1 3', exit 0, running False, kept True"
  press "" q
  settle 2
  shot 546-02-after-less
  read_seq after
  expect "the block reads the same after less" same before after
}
