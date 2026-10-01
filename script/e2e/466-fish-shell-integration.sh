# shellcheck shell=bash
# #466's visual check: Marley's shell integration in fish. The terminal runs fish with a HOME, a
# config folder and a data folder of the scenario's own (this box sets XDG_CONFIG_HOME, so fish
# would read the user's files otherwise). The scenario's `config.fish` prints a marker and whether
# the nonce reached it; fish's history file holds `cargo test --workspace`.
#
# The marker shows the user's file ran, the nonce gone (`marker`, REQ-003); `echo hi` is a finished
# block with its output (`block`, REQ-001); `false` a failed one (`failed`, REQ-002); the nonce is
# not in a command's environment and XDG_DATA_DIRS is what it would have been (`nonce`, REQ-003,
# REQ-004); and the prompt editor's completions list the history file's command (`history`,
# REQ-005).
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
  mkdir -p "$home/.config/fish" "$home/.local/share/fish" "$E2E_WORK/repo"
  cat >"$home/.config/fish/config.fish" <<'FISH'
set -g fish_greeting
# Marley's autosuggestion, not fish's own, is the one to see at the prompt.
set -g fish_autosuggestion_enabled 0
function fish_prompt
    echo -n '$ '
end
echo "config.fish ran; the nonce is" (set -q MARLEY_SHELL_NONCE; and echo set; or echo gone)
FISH
  printf -- '- cmd: cargo test --workspace\n  when: 1700000000\n' >"$home/.local/share/fish/fish_history"
  set_setting terminal "{\"shell\": {\"program\": \"fish\"}, \"env\": {\"HOME\": \"$home\", \"XDG_CONFIG_HOME\": \"$home/.config\", \"XDG_DATA_HOME\": \"$home/.local/share\"}}"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 2
  shot marker

  echo "== a command"
  type_text "echo hi"
  press "" Return
  settle 2
  shot block

  echo "== a failing command"
  type_text "false"
  press "" Return
  settle 2
  shot failed

  echo "== the environment"
  type_text "printenv MARLEY_SHELL_NONCE; echo nonce status \$status"
  press "" Return
  settle 2
  type_text "echo data dirs \$XDG_DATA_DIRS"
  press "" Return
  settle 2
  shot nonce

  echo "== fish's history in the prompt editor"
  type_text "cargo t"
  settle 0.5
  press "" Tab
  settle 1.5
  shot history
}
