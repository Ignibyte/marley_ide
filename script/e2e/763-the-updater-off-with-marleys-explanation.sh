# shellcheck shell=bash
# #763's visual check: Check for Updates on Marley's `dev` channel says Marley updates through its
# install script (`763-01-check`, REQ-001); on that channel it never polls.
compositor sway

setup() {
  mkdir -p "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== Check for Updates"
  press "CTRL SHIFT" p
  settle 1
  type_text "auto update: check"
  settle 1
  press "" Return
  settle 3
  shot 763-01-check
  press "" Return
  settle 1
}
