# shellcheck shell=bash
# The one-shot scenario `just shot` runs: Marley settles, and one shot is taken, with no input.
#
#   NAME    the shot's name (required)
#   SEED    a script run with the profile copy's directory before the launch, to edit the copy
#   SETTLE  seconds to wait after the window maps (12)
#   INSPECT a command run after the shot, while Marley still runs
#   OPEN    a path to open, from the environment; a scratch repository without one

setup() {
  if [[ -n ${SEED:-} ]]; then
    "$SEED" "$E2E_PROFILE"
  fi
  # The profile copy's last session is the user's own projects, whose Agent Panel threads would
  # start their agent (#588).
  if [[ -z $OPEN ]]; then
    git init -q -b main "$E2E_WORK/repo"
    open_path "$E2E_WORK/repo"
  fi
}

steps() {
  settle "${SETTLE:-12}"
  shot "${NAME:?shot.sh needs NAME}"
  if [[ -n ${INSPECT:-} ]]; then
    bash -c "$INSPECT"
  fi
}
