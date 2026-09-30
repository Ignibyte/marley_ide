# shellcheck shell=bash
# #607's e2e test: the Fleet panel on the pseudo provider. The run's settings name
# `{ "kind": "pseudo" }` under `marley.fleet.providers`; `marley: toggle fleet` opens the panel in
# the right dock, listing three agents under two hosts with their chips, work item keys, phases and
# attention marks (`list`, REQ-001). A minute on, the working agent's run has moved to its next phase
# and the quiet agent reads stale (`moved`, REQ-002). With the provider taken out of the settings,
# the panel says the fleet is not set up (`not-set-up`, REQ-003). With it back, a round trip
# through Zed's layout leaves the panel in the right dock, still reading: the working agent has
# moved to its last phase (`layout`, REQ-005). The settings are edited before the round trip
# because an edit to settings.json after Marley has written the file does not reload (#612).
compositor sway

SETTINGS=""

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  SETTINGS=$E2E_PROFILE/config/settings.json
  # The pseudo provider joins the run's `marley` block, or a new one.
  python3 - "$SETTINGS" <<'PY'
import re, sys
path = sys.argv[1]
entry = '"fleet": { "providers": [ { "kind": "pseudo" } ] },'
text = open(path).read()
match = re.search(r'"marley"\s*:\s*\{', text)
if match:
    text = text[:match.end()] + "\n    " + entry + text[match.end():]
else:
    at = text.index("{") + 1
    text = text[:at] + '\n  "marley": {' + entry + '},' + text[at:]
open(path, "w").write(text)
PY
  open_path "$E2E_WORK/repo"
}

# Sets the run's `marley.fleet.providers` to `$1`.
edit_providers() {
  python3 - "$SETTINGS" "$1" <<'PY'
import re, sys
path, providers = sys.argv[1], sys.argv[2]
text = open(path).read()
text, count = re.subn(r'"providers"\s*:\s*\[[^\]]*\]', f'"providers": {providers}', text)
open(path, "w").write(text)
print(f"providers set: {count}")
PY
  grep -n '"fleet"' "$SETTINGS"
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the Fleet panel on the pseudo provider"
  palette "marley: toggle fleet"
  settle 4
  shot list

  echo "== a minute on"
  settle 62
  shot moved

  echo "== the provider taken out of the settings"
  edit_providers '[ ]'
  settle 6
  shot not-set-up

  echo "== the provider back, and a round trip through Zed's layout"
  edit_providers '[ { "kind": "pseudo" } ]'
  settle 6
  palette "marley: use zed layout"
  settle 3
  palette "marley: use marley layout"
  settle 3
  palette "marley: toggle fleet"
  # Past two minutes from the first reading, the working agent is in its last phase.
  settle 35
  shot layout
}
