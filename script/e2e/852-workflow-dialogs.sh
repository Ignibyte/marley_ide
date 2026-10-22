# shellcheck shell=bash
# #852's visual check: Save as Workflow and Run as the Workflows board draws them. A scratch
# repository with `web/`, and a fake `serve` on the terminal's PATH that prints a line and exits.
# - `serve --port 4180 web` run, its block selected and saved: the name, the command, the chips
#   line with `port` and `path`, "2 parts became parameters", the rows and the footer
#   (`852-01-save`, REQ-001).
# - Saved, then run from Zed's task picker with a port typed: the fields and the command filled in
#   (`852-02-run`, REQ-002).
# - Marley Day: the run prompt (`852-03-day`, REQ-002), Escape there running nothing, then the
#   block saved again (`852-04-save-day`, REQ-001), Escape there writing nothing.
compositor sway

STOP_AT=${STOP_AT:-0}
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/web"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # shellcheck disable=SC2016 # The fake's own arguments.
  printf '#!/bin/sh\necho "serving $3 on port $2"\n' >"$bin/serve"
  chmod +x "$bin/serve"
  printf '<h1>web</h1>\n' >"$repo/web/index.html"
  git init -q -b main "$repo"
  open_path "$repo"
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

  echo "== a block, saved as a workflow"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "serve --port 4180 web"
  press "" Return
  settle 3
  press CTRL Up
  settle 1
  palette "marley: save as workflow" 3
  pointer_to 1500 950
  settle 1
  shot 852-01-save
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== saved, then run from the task picker"
  press "" Return
  settle 3
  press "ALT SHIFT" t
  settle 3
  press "" Return
  settle 3
  press CTRL a
  type_text "5000"
  settle 1
  pointer_to 1500 950
  settle 1
  shot 852-02-run

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 852-03-day
  press "" Escape
  settle 2
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  press CTRL Up
  settle 1
  palette "marley: save as workflow" 3
  pointer_to 1500 950
  settle 1
  shot 852-04-save-day
  press "" Escape
  settle 1
}
