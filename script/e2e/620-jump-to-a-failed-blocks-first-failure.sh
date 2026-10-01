# shellcheck shell=bash
# #620's e2e test: Jump to First Failure. `repo/fail.sh` prints forty lines, a rustc error whose
# ` --> ` names `src/main.rs:2:13`, eighty more, and exits 101. Its block's last row carries the Jump
# chip (`chip`, REQ-003) and its menu Jump to First Failure (`menu`, REQ-003). The chip's click
# opens `src/main.rs` at 2:13 (`opened`, REQ-002); back on the terminal, the error's line is the top
# row and the block is selected (`jumped`, REQ-001).
compositor sway

# As the first run's shots found them: the chip at the right of the block's last row, a row of its
# output for the menu, and the terminal's tab.
CHIP_X=${CHIP_X:-1290}
CHIP_Y=${CHIP_Y:-936}
OUTPUT_X=${OUTPUT_X:-500}
OUTPUT_Y=${OUTPUT_Y:-600}
TAB_X=${TAB_X:-395}
TAB_Y=${TAB_Y:-51}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf 'fn main() {\n    let y = x + 1;\n}\n' >"$E2E_WORK/repo/src/main.rs"
  cat >"$E2E_WORK/repo/fail.sh" <<'SH'
for i in $(seq 1 40); do echo "   Compiling step $i"; done
echo 'error[E0425]: cannot find value `x` in this scope'
echo ' --> src/main.rs:2:13'
echo '  |'
echo '2 |     let y = x + 1;'
echo '  |             ^ not found in this scope'
for i in $(seq 1 80); do echo "note: line $i after the error"; done
exit 101
SH
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click 700 500
  settle 1
  type_text "bash fail.sh"
  press "" Return
  settle 3
  pointer_to 700 300
  settle 1
  shot chip

  echo "== the block's menu"
  click "$OUTPUT_X" "$OUTPUT_Y" right
  settle 1.5
  shot menu
  press "" Escape
  settle 1

  echo "== the chip's click"
  click "$CHIP_X" "$CHIP_Y"
  settle 4
  shot opened

  echo "== back on the terminal"
  click "$TAB_X" "$TAB_Y"
  settle 2
  pointer_to 700 300
  settle 1
  shot jumped
}
