# shellcheck shell=bash
# #623's e2e test: a failed block's errors as the project's diagnostics. The task `build` runs
# `build.sh`, which prints two rustc errors and a warning in `src/main.rs` and exits 101, until a
# file `ok` exists, when it prints `compiled` and exits 0. After the failing run the status bar
# counts 2 errors and 1 warning (`status`, REQ-001) and the Diagnostics view lists them under
# `src/main.rs` (`diagnostics`, REQ-001). With `ok` made, the task run again clears them
# (`cleared`, REQ-002).
compositor sway

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo/.zed" "$E2E_WORK/repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf 'fn main() {\n    let y = x + 1;\n    let z = w;\n}\n' >"$E2E_WORK/repo/src/main.rs"
  cat >"$E2E_WORK/repo/build.sh" <<'SH'
if [ -e ok ]; then echo compiled; exit 0; fi
echo 'error[E0425]: cannot find value `x` in this scope'
echo ' --> src/main.rs:2:13'
echo 'error[E0425]: cannot find value `w` in this scope'
echo ' --> src/main.rs:3:13'
echo 'warning: unused variable: `y`'
echo ' --> src/main.rs:2:9'
exit 101
SH
  cat >"$E2E_WORK/repo/.zed/tasks.json" <<'JSON'
[
  { "label": "build", "command": "sh build.sh" }
]
JSON
  open_path "$E2E_WORK/repo"
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Runs the task `build` from `task: spawn`.
build() {
  palette "task: spawn"
  settle 1.5
  type_text "build"
  settle 1
  press "" Return
  settle 4
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the failing build"
  build
  pointer_to 900 400
  settle 1
  shot status
  palette "diagnostics: deploy"
  settle 3
  shot diagnostics

  echo "== the build made to pass, run again"
  touch "$E2E_WORK/repo/ok"
  build
  pointer_to 900 400
  settle 2
  shot cleared
}
