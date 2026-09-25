# shellcheck shell=bash
# #488's e2e test, when there is no Chromium: MARLEY_CHROMIUM names a binary that is not there,
# and the Browser tab says so, naming it, instead of waiting or showing nothing.
compositor sway

setup() {
  export MARLEY_CHROMIUM=$E2E_WORK/no-chromium-here
  git init -q -b browser "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: open browser"
  settle 1
  press "" Return
  settle 3
  shot 488-06-no-chromium
}
