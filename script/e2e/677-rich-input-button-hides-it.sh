# shellcheck shell=bash
# #677's visual check: the agent bar's Rich Input button hides the agent's editor too. The shell
# becomes #481's stand-in Claude Code (`exec -a claude`, so the terminal's foreground process is
# `claude` by name), which brings the agent bar and its pencil.
#
# `677-01-open`: the pencil clicked, the pointer away and back on it: the editor above the bar, the
# button pressed, its tooltip Hide Rich Input (REQ-001). `677-02-hidden`: a draft typed, the pencil
# clicked: the editor gone, the button unpressed (REQ-002). `677-03-draft`: the pencil clicked
# again: the editor back with the draft (REQ-002).
compositor sway

# In the window's logical pixels, from the first run's shots: the agent bar's pencil, and a point
# clear of the bar.
PENCIL_X=${PENCIL_X:-472}
PENCIL_Y=${PENCIL_Y:-931}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-400}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
stand_in() {
  exec -a claude bash -c 'printf "ready\n"; while IFS= read -r line; do printf "claude got: %s\n" "$line"; done'
}
RC
  terminal_env HOME "$home"
  git init -q -b rich-input "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "stand_in"
  press "" Return
  settle 4

  echo "== the pencil: open"
  click "$PENCIL_X" "$PENCIL_Y"
  settle 1
  # A tooltip keeps what it said while the pointer stays, so the pointer leaves and comes back.
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  pointer_to "$PENCIL_X" "$PENCIL_Y"
  settle 2
  shot 677-01-open

  echo "== a draft, the pencil: hidden"
  type_text "a draft for later"
  settle 1
  click "$PENCIL_X" "$PENCIL_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 677-02-hidden

  echo "== the pencil again: the draft back"
  click "$PENCIL_X" "$PENCIL_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 677-03-draft
}
