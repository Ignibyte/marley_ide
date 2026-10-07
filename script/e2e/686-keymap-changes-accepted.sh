# shellcheck shell=bash
# #686's visual check: a key binding an agent proposes and the user accepts. A stand-in agent calls
# `keymap_change` through the Claude Code plugin's bridge: `ctrl-alt-m` for
# `workspace::ToggleRightDock` in `Workspace` shows the question (`card`, REQ-001); Apply writes it
# into the run's keymap.json (REQ-002) and Ctrl+Alt+M hides the right dock at once (`key-works`,
# REQ-003); an action the app lacks is refused with no question (`refused`, REQ-004).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The question's Apply, as #682's run found it.
APPLY_X=${APPLY_X:-958}
APPLY_Y=${APPLY_Y:-901}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  export MCP_CLIENT_NAME="Stand-in agent"
  git init -q -b main "$repo"
  printf '# repo\n' >"$repo/README.md"
  open_path "$repo"
}

# The run's keymap file.
keymap_file() {
  printf '%s' "$E2E_PROFILE/config/keymap.json"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== a proposal asks the user"
  mcp_agent tool keymap_change \
    '{"keystrokes": "ctrl-alt-m", "action": "workspace::ToggleRightDock", "context": "Workspace"}' \
    >"$E2E_WORK/apply.txt" 2>&1 &
  local proposal=$!
  settle 3
  shot 686-01-card

  echo "== Apply writes it, and the key works"
  click "$APPLY_X" "$APPLY_Y"
  wait "$proposal" || true
  cat "$E2E_WORK/apply.txt"
  expect "the answer is applied" holds "$E2E_WORK/apply.txt" '"result": "applied"'
  expect "the keymap holds the binding" holds "$(keymap_file)" "ctrl-alt-m" \
    "workspace::ToggleRightDock"
  settle 2
  press "CTRL ALT" m
  settle 2
  shot 686-02-key-works

  echo "== an unknown action asks nothing"
  mcp_agent tool keymap_change '{"keystrokes": "ctrl-alt-n", "action": "workspace::NoSuchAction"}' \
    | tee "$E2E_WORK/unknown.txt"
  expect "an unknown action is no_action" holds "$E2E_WORK/unknown.txt" "no_action"
  settle 1
  shot 686-03-refused
}
