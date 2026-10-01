# shellcheck shell=bash
# #628's visual check: a block's prompt rows drawn as Marley's header. bash with a two-line PS1,
# `[the prompt]` over `$ `, and `marley.block_headers` on; the terminal copies a selection as it is
# made, so its text can be pasted into the prompt editor.
#
# After `echo hi` a drag over `hi` selects it alone (`selected`) and its paste into the editor
# reads `hi` (`pasted`, REQ-003). After `ls` each block shows its command and pill where its
# prompt was, with no `[the prompt]` text, and the live prompt as bash drew it (`headers`,
# REQ-001, REQ-002). With the setting off the blocks draw as before (`off`, REQ-004).
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

# The rows of `hi`, `echo hi`'s output, once the block is drawn over the docked editor.
HI_Y=${HI_Y:-882}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  cat >"$home/.bashrc" <<'RC'
PS1='[the prompt]\n$ '
RC
  set_setting terminal "{\"env\": {\"HOME\": \"$home\"}, \"copy_on_select\": true}"
  set_setting marley.block_headers true
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  printf 'notes\n' >"$E2E_WORK/repo/notes.txt"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 2

  echo "== a block's output selected"
  type_text "echo hi"
  press "" Return
  settle 2
  pointer_to 268 "$HI_Y"
  pointer_down
  pointer_to 280 "$HI_Y"
  pointer_to 289 "$HI_Y"
  pointer_up
  settle 1
  shot selected
  press CTRL v
  settle 1
  shot pasted
  press CTRL c
  settle 0.5

  echo "== headers"
  type_text "ls"
  press "" Return
  settle 2
  pointer_to 700 300
  settle 1
  shot headers

  echo "== the setting off"
  set_setting marley.block_headers false
  settle 2
  shot off
}
