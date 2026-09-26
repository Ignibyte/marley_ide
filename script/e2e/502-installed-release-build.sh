# shellcheck shell=bash
# #502's e2e test: the installed release build. Setup installs Marley into a scratch prefix with
# script/install-marley (the release build is current, so cargo only checks it) and checks the
# desktop entry with desktop-file-validate. The run starts the installed binary itself, not its
# launcher, whose stderr log would land in Chad's data directory: the binary opens the scratch
# repository in the Marley layout, the install runs again while it runs, and the window still
# answers. Then Marley quits, and the entry starts the way Omarchy's menu starts one,
# `uwsm-app -- gtk-launch marley.desktop`, with this run's sway as its display and scratch XDG
# directories, so it finds the scratch entry, a fresh profile and a scratch log rather than
# Chad's. Last, the launcher run with a flag Marley refuses shows its stderr landing in the log.
compositor sway

PREFIX=
MENU_PID=

setup() {
  local entry installed
  PREFIX=$E2E_WORK/prefix
  entry=$PREFIX/share/applications/marley.desktop
  echo "== install into $PREFIX"
  script/install-marley --prefix "$PREFIX"
  echo "== the desktop entry"
  cat "$entry"
  desktop-file-validate "$entry"
  echo "desktop-file-validate: no complaints"
  installed=$(sed -n 's/^Exec="\(.*\)" %U$/\1/p' "$entry")
  echo "the entry runs $installed, which runs: $(grep '^exec ' "$installed")"
  echo "the binary is $(du -h "$PREFIX/lib/marley/marley" | cut -f1); HEAD is $(git rev-parse --short HEAD)"
  binary "$PREFIX/lib/marley/marley"
  git init -q -b installed "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -n $MENU_PID ]]; then
    # Its log is in the scratch prefix, which goes with the run: the lines that matter, here.
    grep -E "starting zed version|ERROR|panic" "$PREFIX/share/marley/logs/Marley.log" || true
    kill -TERM "$MENU_PID" 2>/dev/null || true
    sleep 1
  fi
}

steps() {
  local pid
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  shot 502-01-installed
  echo "== the install again while the installed Marley runs"
  pid=$(marley_pid)
  script/install-marley --prefix "$PREFIX"
  settle 2
  if ! kill -0 "$pid" 2>/dev/null; then
    echo "the installed Marley ($pid) stopped during the reinstall" >&2
    return 1
  fi
  echo "the installed Marley ($pid) still runs"
  # The palette shows that the window still answers.
  press "CTRL SHIFT" p
  settle 2
  shot 502-02-reinstalled
  press "" Escape
  settle 1
  echo "== the entry started the way Omarchy's menu starts one"
  quit_marley
  mkdir -p "$E2E_WORK/menu-config/marley"
  cp "$E2E_PROFILE/config/settings.json" "$E2E_WORK/menu-config/marley/"
  # A desktop's seat has a keyboard when an app starts; the run's last `wtype` has exited, so
  # the seat gets the held one back first.
  hold_keyboard
  env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" SWAYSOCK="$SWAY_SOCK" \
    XDG_DATA_HOME="$PREFIX/share" XDG_CONFIG_HOME="$E2E_WORK/menu-config" \
    uwsm-app -- gtk-launch marley.desktop "$E2E_WORK/repo"
  for _ in $(seq 90); do
    MENU_PID=$(sway_marley_pid)
    [[ -n $MENU_PID ]] && break
    sleep 1
  done
  if [[ -z $MENU_PID ]]; then
    echo "no Marley window from the menu's launch within 90 seconds" >&2
    return 1
  fi
  echo "the menu's Marley is $MENU_PID, in $(cut -d: -f3 "/proc/$MENU_PID/cgroup")"
  settle 10
  if ! kill -0 "$MENU_PID" 2>/dev/null; then
    echo "the menu's Marley ($MENU_PID) stopped within ten seconds" >&2
    return 1
  fi
  echo "the menu's Marley ($MENU_PID) still runs after ten seconds"
  # Trusts the scratch repository in the fresh profile.
  press "" Return
  settle 3
  shot 502-03-from-the-menu
  echo "== the launcher's stderr log"
  local status=0
  XDG_DATA_HOME="$PREFIX/share" "$PREFIX/bin/marley" --no-such-flag || status=$?
  echo "the launcher with a refused flag exited $status"
  cat "$PREFIX/share/marley/logs/stderr.log"
  if ! grep -q "unexpected argument '--no-such-flag'" "$PREFIX/share/marley/logs/stderr.log"; then
    echo "the refused flag's error is not in the launcher's log" >&2
    return 1
  fi
}
