# shellcheck shell=bash
# #619's e2e test: a block's path link opens against the block's own folder. `repo/a/src/main.rs`
# and `repo/b/src/main.rs` each say which they are. The terminal keeps 100 lines of history, and
# `seq 1 300` in `b` fills it, past which Zed's own guess at a line's folder gives way to the
# shell's folder now. In `a`, `printf` prints `src/main.rs:2:5`; then the shell goes back to `b`.
# Ctrl held over the link names `a`'s file (`hover`, REQ-002); Ctrl+click opens it at line 2
# (`opened`, REQ-001).
compositor sway

# The printed link's row: the prompt on the last row, `cd ../b` above it, the link above that.
LINK_X=${LINK_X:-320}
LINK_Y=${LINK_Y:-916}

set_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
*parents, last = sys.argv[2].split(".")
node = settings
for key in parents:
    node = node.setdefault(key, {})
node[last] = json.loads(sys.argv[3])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

setup() {
  local home=$E2E_WORK/home name
  mkdir -p "$home"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  # The terminal block holds both, since `terminal_env` wants the block to itself.
  set_setting terminal.env "{\"HOME\": \"$home\"}"
  for name in a b; do
    mkdir -p "$E2E_WORK/repo/$name/src"
    printf '// The main.rs in %s.\nfn main() {\n    println!("%s");\n}\n' "$name" "$name" \
      >"$E2E_WORK/repo/$name/src/main.rs"
  done
  set_setting terminal.max_scroll_history_lines 100
  open_path "$E2E_WORK/repo"
}

# Runs `$1` at the terminal's prompt.
run() {
  type_text "$1"
  press "" Return
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 1
  run "cd b"
  run "seq 1 300"
  run "cd ../a"
  run "printf 'src/main.rs:2:5\\n'"
  run "cd ../b"
  settle 1
  pointer_to 700 400
  settle 1
  shot printed

  echo "== Ctrl held over the link"
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -M ctrl -s 3000 -m ctrl &
  local holder=$!
  sleep 0.4
  pointer_to "$LINK_X" "$LINK_Y"
  settle 1
  # The tooltip comes with a move over the link once Zed has found it.
  pointer_to "$((LINK_X + 4))" "$LINK_Y"
  settle 1.2
  shot hover
  wait "$holder"

  echo "== Ctrl+click on the link"
  click_with CTRL "$LINK_X" "$LINK_Y"
  settle 3
  shot opened
}
