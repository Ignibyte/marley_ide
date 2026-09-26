# shellcheck shell=bash
# #515's e2e test: the Settings window's Marley page. `marley: open settings` from the palette
# opens the window on the Marley page, first in its list, with the Layout dropdown at Marley and
# the two telemetry toggles off. Setting the dropdown to Zed switches the main window to Zed's
# layout (the profile is a copy, so nothing needs putting back).
compositor sway

# The Layout dropdown on the Marley page, and its Zed entry once open, measured from the first runs.
DROPDOWN_X=${DROPDOWN_X:-1528}
DROPDOWN_Y=${DROPDOWN_Y:-216}
ZED_ENTRY_X=${ZED_ENTRY_X:-1412}
ZED_ENTRY_Y=${ZED_ENTRY_Y:-250}

setup() {
  git init -q -b settings "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== marley: open settings"
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: open settings"
  settle 1
  press "" Return
  settle 4
  shot 515-01-marley-page
  if [[ $DROPDOWN_X -eq 0 ]]; then
    echo "no dropdown coordinates yet: measure them on 515-01-marley-page"
    return 0
  fi
  echo "== the Layout dropdown set to Zed"
  click "$DROPDOWN_X" "$DROPDOWN_Y"
  settle 1
  shot 515-02a-dropdown-open
  if [[ $ZED_ENTRY_X -eq 0 ]]; then
    echo "no Zed entry coordinates yet: measure them on 515-02a-dropdown-open"
    return 0
  fi
  click "$ZED_ENTRY_X" "$ZED_ENTRY_Y"
  settle 3
  shot 515-02-zed-layout
  expect "the dropdown wrote the Zed layout" \
    grep -qE '"layout": *"zed"' "$E2E_PROFILE/config/settings.json"
}
