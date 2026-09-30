# shellcheck shell=bash
# #608's e2e test: the selected agent's snapshot under the Fleet panel's list, on the pseudo
# provider. A click on build-1 selects it and shows its snapshot below the list: the header with
# how long it has worked, RB-142's title, the phase strip with its active phase, CPU and memory
# bars and tokens today (`working`, REQ-001 and REQ-002). A click on review-1 shows its question
# and its options (`question`, REQ-003); one on docs-1 shows its failed phase in the strip
# (`failed`, REQ-004). With review-1 selected again, Down selects docs-1 (`keys`, REQ-005). The
# split's handle dragged up gives the snapshot more of the panel (`split`, the resizable split).
compositor sway

# The agents' rows in the panel, as #607's shots place them on the 1600 x 1000 output.
ROW_X=1400
BUILD_Y=141
REVIEW_Y=186
DOCS_Y=258
# The split's handle at half the panel's body, and where the drag takes it.
SPLIT_Y=516
SPLIT_TO_Y=400

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
  settle 4

  echo "== build-1 selected"
  click "$ROW_X" "$BUILD_Y"
  settle 3
  shot working

  echo "== review-1 and its question"
  click "$ROW_X" "$REVIEW_Y"
  settle 3
  shot question

  echo "== docs-1 and its failed phase"
  click "$ROW_X" "$DOCS_Y"
  settle 3
  shot failed

  echo "== Down from review-1"
  click "$ROW_X" "$REVIEW_Y"
  settle 2
  press "" Down
  settle 3
  shot keys

  echo "== the split dragged up"
  pointer_to "$ROW_X" "$SPLIT_Y"
  settle 0.3
  pointer_down
  settle 0.3
  for step in 1 2 3 4 5 6; do
    pointer_to "$ROW_X" "$((SPLIT_Y + (SPLIT_TO_Y - SPLIT_Y) * step / 6))"
    sleep 0.1
  done
  pointer_up
  settle 2
  shot split
}
