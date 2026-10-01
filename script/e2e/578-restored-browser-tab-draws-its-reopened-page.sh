# shellcheck shell=bash
# #578's e2e test: a restored Browser tab draws the page it reopened. It runs #576's flow, three
# launches with Marley's Chromium stopped after each so the restored tab opens its saved URL in a
# new page, with the hub's stream logged (each start and stop, each frame, and the watchdog's
# restarts), then reads the third launch's shot at the page's point: page A's light background,
# each channel above 200, or the run fails (`576-03-a-again`, REQ-002). Run it ten times; a line
# "drew nothing" in the log is the watchdog at work (REQ-001).
# shellcheck source=script/e2e/576-browser-tabs-table-item-id-not-unique.sh
. script/e2e/576-browser-tabs-table-item-id-not-unique.sh

# The page area of the restored tab, in the split's right pane.
PAGE_X=${PAGE_X:-1100}
PAGE_Y=${PAGE_Y:-500}

eval "flow_$(declare -f setup)"
eval "flow_$(declare -f steps)"

setup() {
  export ZED_LOG="info,marley_workbench=debug"
  flow_setup
}

steps() {
  flow_steps
  page_drawn 576-03-a-again
}

# Fails the run unless shot `$1` shows the page's light background at the page's point.
page_drawn() {
  local pixel
  pixel=$(magick "$(shot_file "$1.png")" -format "%[pixel:p{$PAGE_X,$PAGE_Y}]" info:)
  echo "the page's point in $1: $pixel"
  python3 - "$pixel" <<'PY'
import re, sys
channels = [int(value) for value in re.findall(r"\d+", sys.argv[1])[:3]]
sys.exit(0 if len(channels) == 3 and min(channels) > 200 else 1)
PY
}
