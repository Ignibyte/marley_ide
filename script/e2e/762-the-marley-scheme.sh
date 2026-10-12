# shellcheck shell=bash
# #762's visual check: the marley:// scheme. Fakes of `xdg-open`, `gio` and two browsers come first
# on Marley's PATH and log any call (#579's), so a link that leaves Marley is caught. The desktop's
# half runs the real `/usr/bin/xdg-open` with a scratch `XDG_DATA_HOME` and `XDG_CONFIG_HOME`
# holding a desktop entry like the install's, `MimeType=x-scheme-handler/marley;`, whose `Exec` is
# the run's build on the run's profile, so the user's own entries and defaults play no part.
# Shots: `xdg-open marley://settings` opens Marley's Settings window (REQ-001); Open URL and a
# Ctrl-click on `zed://settings` in a buffer open it, and nothing reached the fakes (REQ-002); a
# click on a `marley://settings` link in the markdown preview opens it, nothing reached the fakes
# (REQ-003).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
URL_X=${URL_X:-420}
URL_Y=${URL_Y:-125}
LINK_X=${LINK_X:-609}
LINK_Y=${LINK_Y:-136}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local bin=$E2E_WORK/bin name log applications=$E2E_WORK/xdg-data/applications
  mkdir -p "$bin" "$applications" "$E2E_WORK/xdg-config" "$E2E_WORK/repo"
  for name in xdg-open gio google-chrome firefox; do
    log=$E2E_WORK/leak.log
    [[ $name == xdg-open ]] && log=$E2E_WORK/xdg-open.log
    cat >"$bin/$name" <<SH
#!/bin/sh
printf '%s\n' "\$*" >>"$log" || true
exit 0
SH
    chmod +x "$bin/$name"
  done
  : >"$E2E_WORK/xdg-open.log"
  : >"$E2E_WORK/leak.log"
  export PATH="$bin:$PATH"
  # The install's entry, its Exec on this run's build and profile.
  # Unquoted, as the install writes it when the path holds no space: xdg-open's generic mode reads
  # the first word as it stands.
  cat >"$applications/marley.desktop" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Marley
Exec=$(marley_binary) --user-data-dir $E2E_PROFILE %U
Terminal=false
MimeType=x-scheme-handler/marley;
EOF
  update-desktop-database "$applications"
  cat >"$E2E_WORK/repo/links.md" <<'MD'
zed://settings

[Open the settings](marley://settings)
MD
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The real xdg-open, in a desktop of the run's own: its entries and no defaults of the user's. When
# it finds no handler it falls back to a web browser: BROWSER names the logging fake, so that shows
# as a leak and starts no browser.
desktop_open() {
  env -u XDG_CURRENT_DESKTOP -u DESKTOP_SESSION -u KDE_FULL_SESSION -u GNOME_DESKTOP_SESSION_ID \
    XDG_DATA_HOME="$E2E_WORK/xdg-data" XDG_CONFIG_HOME="$E2E_WORK/xdg-config" \
    XDG_DATA_DIRS=/usr/share BROWSER="$E2E_WORK/bin/firefox" PATH=/usr/bin:/bin \
    timeout 20 /usr/bin/xdg-open "$1"
}

nothing_left_marley() { [[ ! -s $E2E_WORK/xdg-open.log && ! -s $E2E_WORK/leak.log ]]; }

# Whether a window titled as Marley's Settings window is open in the sway.
settings_open() {
  sway_msg -t get_tree | jq -e '[.. | objects | select(.name? // "" | test("Settings"))] | length > 0' >/dev/null
}

# Closes the Settings window, which has the focus, so the next check sees a window opened anew.
close_settings() {
  press "CTRL" w
  settle 2
  expect "the Settings window closed" settings_closed
}

settings_closed() { ! settings_open; }

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== xdg-open marley://settings"
  desktop_open marley://settings
  settle 5
  take 762-01-desktop || return 0
  expect "the desktop's link opened the Settings window" settings_open
  expect "no browser was asked" nothing_left_marley
  close_settings

  echo "== Open URL on zed://settings in a buffer"
  press "CTRL" p
  settle 1
  type_text "links.md"
  settle 1
  press "" Return
  settle 3
  press "CTRL" Home
  palette "editor: open url"
  settle 4
  take 762-02a-open-url || return 0
  expect "Open URL opened the Settings window" settings_open
  expect "nothing went to xdg-open" nothing_left_marley
  close_settings
  if ((URL_Y == 0)); then
    echo "the buffer's URL is not placed yet: 762-02a is the last shot"
    return 0
  fi

  echo "== Ctrl-click on zed://settings"
  click_with "CTRL" "$URL_X" "$URL_Y"
  settle 4
  take 762-02-editor || return 0
  expect "the Ctrl-click opened the Settings window" settings_open
  expect "nothing went to xdg-open" nothing_left_marley
  close_settings

  echo "== the markdown preview's marley:// link"
  palette "markdown: open preview"
  settle 4
  take 762-03a-preview || return 0
  if ((LINK_Y == 0)); then
    echo "the preview's link is not placed yet"
    return 0
  fi
  click "$LINK_X" "$LINK_Y"
  settle 4
  take 762-03-preview || return 0
  expect "the preview's link opened the Settings window" settings_open
  expect "nothing went to xdg-open" nothing_left_marley
}
