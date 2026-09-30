# shellcheck shell=bash
# #609's e2e test: the Agent tab, on the pseudo provider. With the Fleet panel showing for about a
# minute, a double-click on build-1 opens its tab in the center: its phases in time with their
# gates, its events newest first, CPU, memory and network over the samples kept, its tokens, and
# review-1 under ON THIS HOST (`tab`, REQ-001 to REQ-005). docs-1 selected and Enter opens its tab
# with the failed phase and gate (`failed`, REQ-002). A second double-click on build-1 shows its
# tab again rather than a second one (`again`, REQ-001). In build-1's tab, a click on review-1
# opens review-1's tab (`other`, REQ-006).
compositor sway

# The agents' rows in the panel, as #607's shots place them on the 1600 x 1000 output.
ROW_X=1400
BUILD_Y=141
DOCS_Y=258
# review-1's row under ON THIS HOST in build-1's tab.
NEIGHBOUR_X=330
NEIGHBOUR_Y=844

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # The pseudo provider joins the run's `marley` block, or a new one.
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
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

# Two clicks at `$1`,`$2`, close enough together to count as one double-click.
double_click() {
  pointer_to "$1" "$2"
  sleep 0.05
  pointer_down
  sleep 0.05
  pointer_up
  sleep 0.08
  pointer_down
  sleep 0.05
  pointer_up
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: toggle fleet"
  settle 1
  press "" Return

  echo "== a minute of samples, then build-1's tab"
  settle 60
  double_click "$ROW_X" "$BUILD_Y"
  settle 4
  shot tab

  echo "== docs-1 with Enter"
  click "$ROW_X" "$DOCS_Y"
  settle 2
  press "" Return
  settle 4
  shot failed

  echo "== build-1 again"
  double_click "$ROW_X" "$BUILD_Y"
  settle 4
  shot again

  echo "== review-1 from build-1's tab"
  click "$NEIGHBOUR_X" "$NEIGHBOUR_Y"
  settle 4
  shot other
}
