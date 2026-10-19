# shellcheck shell=bash
# #815's visual check: the installed release build draws Chad's icons. A debug build reads
# `assets/` from the checkout and drew them before the fix, so this runs the installed binary,
# with #807's set-up (its `setup`, scripted agent and `palette`, sourced from its scenario).
# - The title bar's Rusty and Marley buttons draw the butler and the cat, and the status bar's Home
#   the house (`815-01-title-bar`, REQ-002).
# - `rusty: open home`: Home's tab and its rail row draw the house (`815-02-home`, REQ-002).
# - The New Agent picker: the Marley and Rusty entries draw the cat and the butler
#   (`815-03-picker`, REQ-002).
. script/e2e/807-marley-rusty-and-home-icons.sh

INSTALLED=${INSTALLED:-$HOME/.local/lib/marley/marley}
binary "$INSTALLED"

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5
  shot 815-01-title-bar

  echo "== Home"
  palette "rusty: open home"
  settle 3
  shot 815-02-home

  echo "== the New Agent picker"
  press "CTRL ALT" n
  settle 2
  shot 815-03-picker
  press "" Escape
  settle 1
}
