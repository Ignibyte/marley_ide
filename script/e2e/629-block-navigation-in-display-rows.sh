# shellcheck shell=bash
# #629's visual check: moving between blocks, the pinned header, bookmark ticks and search, with
# #628's headers on. bash with a two-line PS1, `[the prompt]` over `$ `; three blocks of 40 lines
# and one that prints `seq`.
#
# Ctrl+Up to the second block puts its header on the top row (`top`, REQ-001); scrolled into its
# output, the pinned header reads its command with no `$ ` (`pinned`, REQ-002); its bookmark's tick
# sits at its height in the scrollback (`ticks`, REQ-003); a search for `seq` highlights the output
# and no header (`search`, REQ-004).
compositor sway

# Sets the settings key path `$1` (dot-separated) to the JSON value `$2` in the run's copy of the
# settings.
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
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  cat >"$home/.bashrc" <<'RC'
PS1='[the prompt]\n$ '
RC
  terminal_env HOME "$home"
  set_setting marley.block_headers true
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
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
  run "seq 1 40"
  run "seq 101 140"
  run "seq 201 240"
  run "echo the seq ends"
  # The keys go to the shell's grid until the next prompt.
  press "" Escape
  settle 1

  echo "== a search, with no block selected"
  press "CTRL SHIFT" f
  settle 1
  type_text "seq"
  settle 1.5
  pointer_to 700 400
  settle 1
  shot search
  press "" Escape
  settle 1

  echo "== to the second block"
  press CTRL Up
  settle 0.5
  press CTRL Up
  settle 0.5
  press CTRL Up
  settle 1
  pointer_to 700 400
  settle 1
  shot top

  echo "== into its output"
  scroll 3
  settle 1
  shot pinned

  echo "== a bookmark"
  press "CTRL SHIFT" b
  settle 0.5
  scroll 60
  settle 1
  shot ticks

}
