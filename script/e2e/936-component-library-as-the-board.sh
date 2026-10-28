# shellcheck shell=bash
# #936's visual check: the Component Library as the Components board draws it. A scratch
# repository; the run's config folder `components/` holds `deploy_status.json` and
# `release_note.json` (#801's) and `bad_one.json`, which uses an input its schema does not name.
# - `card` clicked in the left list: its row filled, the page narrowed to it, Try it's header with
#   its hint, and the template strip with two "loaded" chips and one red chip
#   (`936-01-chosen`, REQ-001, REQ-003, REQ-005).
# - The `deploy_status` chip clicked: its example in Try it and its card drawn under the box
#   (`936-02-template`, REQ-002).
# - A badge with a tone Marley does not have pasted into Try it: the error on the header line and
#   no callout (`936-03-error`, REQ-004).
# - `936-01-chosen`'s view in Marley Day (`936-04-day`, REQ-001, REQ-003, REQ-005).
compositor sway

STOP_AT=${STOP_AT:-0}
# The navigation column's find field (#849's measure), the `deploy_status` chip and the
# scratch box, from the first run's shots.
FIND_X=${FIND_X:-370}
FIND_Y=${FIND_Y:-86}
CHIP_X=${CHIP_X:-570}
CHIP_Y=${CHIP_Y:-840}
SCRATCH_X=${SCRATCH_X:-1000}
SCRATCH_Y=${SCRATCH_Y:-260}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo components=$E2E_PROFILE/config/components
  mkdir -p "$home" "$repo" "$components"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  cat >"$components/deploy_status.json" <<'JSON'
{
  "name": "deploy_status",
  "description": "Shows one environment's deploy: its version, its deploys today and its services.",
  "inputs": {
    "type": "object",
    "properties": {
      "environment": {"type": "string"},
      "version": {"type": "string"},
      "deploys": {"type": "integer"},
      "services": {"type": "array"}
    },
    "required": ["environment", "version"]
  },
  "example": {"environment": "staging", "version": "1.4.0", "deploys": 4,
              "services": [["api", "up"], ["worker", "slow"]]},
  "body": {"type": "card", "title": {"$input": "environment"}, "tone": "success", "children": [
    {"type": "row", "gap": "large", "children": [
      {"type": "stat", "label": "Version", "value": {"$input": "version"}},
      {"type": "stat", "label": "Deploys today", "value": {"$input": "deploys"}}]},
    {"type": "table", "columns": ["Service", "State"], "rows": {"$input": "services"}}]}
}
JSON
  cat >"$components/release_note.json" <<'JSON'
{
  "name": "release_note",
  "description": "Shows a release's headline and what changed.",
  "inputs": {"type": "object", "properties": {"headline": {"type": "string"}, "text": {"type": "string"}}},
  "example": {"headline": "1.5.0 is out", "text": "Templates load from files."},
  "body": {"type": "callout", "tone": "success", "title": {"$input": "headline"}, "text": {"$input": "text"}}
}
JSON
  cat >"$components/bad_one.json" <<'JSON'
{
  "name": "bad_one",
  "description": "Uses an input it does not name.",
  "inputs": {"type": "object", "properties": {"title": {"type": "string"}}},
  "example": {"title": "x"},
  "body": {"type": "text", "text": {"$input": "missing"}}
}
JSON
  printf '{"type": "badge", "text": "beta", "tone": "orange"}\n' >"$E2E_WORK/wrong-tone.json"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

# Replaces the scratch box's text with the file `$1`, through the run's clipboard (#799's).
paste_into_scratch() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy <"$1"
  click "$SCRATCH_X" "$SCRATCH_Y"
  settle 1
  press CTRL a
  settle 0.5
  press CTRL v
  settle 2
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the Component Library, card chosen"
  palette "marley: open component library" 5
  click "$FIND_X" "$FIND_Y"
  settle 1
  type_text "card"
  settle 2
  pointer_to 1500 950
  settle 1
  shot 936-01-chosen
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== the deploy_status chip"
  click "$CHIP_X" "$CHIP_Y"
  settle 3
  pointer_to 1500 950
  settle 1
  shot 936-02-template

  echo "== a tone Marley does not have"
  paste_into_scratch "$E2E_WORK/wrong-tone.json"
  pointer_to 1500 950
  settle 1
  shot 936-03-error

  echo "== Marley Day"
  click "$SCRATCH_X" "$SCRATCH_Y"
  press "CTRL" a
  press "" BackSpace
  settle 1
  profile_setting theme '"Marley Day"'
  settle 4
  pointer_to 1500 950
  settle 1
  shot 936-04-day
}
