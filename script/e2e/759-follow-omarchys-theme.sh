# shellcheck shell=bash
# #759's visual check: Marley follows Omarchy's theme. The runner's copy turns
# `marley.omarchy_theme` off and points a run's Omarchy at `$E2E_WORK/omarchy-home`, empty; this
# scenario sets `theme` to Gruvbox Dark, then fakes Omarchy there: two stock palettes, a `switch`
# that does what a theme switch does (stage the palette, render with Omarchy's own
# `omarchy-theme-set-templates` under the fake HOME, swap the folder in, run the `theme-set.d`
# hooks with bash), and a refresh that re-runs it for the current palette and logs each call. It
# never runs `omarchy-theme-set`, whose app hooks reload the user's Hyprland and terminals.
# Shots: the toggle, off (REQ-001); on without Omarchy, still Gruvbox Dark (REQ-002); on with
# Omarchy on tokyo-night (REQ-003, REQ-004); a switch to catppuccin-latte (REQ-005); off, Gruvbox
# Dark again (REQ-007, REQ-008). A render with a placeholder left keeps the copy (REQ-006), and the
# user's own Omarchy folders are the same at the end (REQ-011).
compositor sway

# The names and times in the user's own Omarchy folders this touches.
user_omarchy_listing() {
  ls -la --time-style=full-iso "$HOME/.config/omarchy/themed" "$HOME/.config/omarchy/hooks/theme-set.d" 2>&1
}

write_fakes() {
  mkdir -p "$E2E_WORK/bin"
  cat >"$E2E_WORK/bin/switch" <<SWITCH
#!/usr/bin/env bash
# A theme switch in the fake Omarchy home, the steps of omarchy-theme-set that concern Marley.
set -e
export HOME=$(printf '%q' "$FAKE_HOME")
export OMARCHY_PATH=$(printf '%q' "$OMARCHY_PATH")
state=\$HOME/.local/state/omarchy/current
rm -rf "\$state/next-theme"
mkdir -p "\$state/next-theme"
cp "\$OMARCHY_PATH/themes/\$1/colors.toml" "\$state/next-theme/"
omarchy-theme-set-templates
rm -rf "\$state/theme"
mv "\$state/next-theme" "\$state/theme"
echo "\$1" >"\$state/theme.name"
for hook in "\$HOME/.config/omarchy/hooks/theme-set.d"/*; do
  [[ -f \$hook ]] || continue
  bash "\$hook" "\$1"
done
SWITCH
  cat >"$E2E_WORK/bin/omarchy-theme-refresh" <<REFRESH
#!/usr/bin/env bash
# omarchy-theme-refresh in the fake home: the switch again, to the current palette, logged.
set -e
echo refresh >>$(printf '%q' "$FAKE_REFRESH_LOG")
exec $(printf '%q' "$E2E_WORK/bin/switch") "\$(cat $(printf '%q' "$FAKE_HOME/.local/state/omarchy/current/theme.name"))"
REFRESH
  chmod +x "$E2E_WORK/bin/switch" "$E2E_WORK/bin/omarchy-theme-refresh"
  : >"$FAKE_REFRESH_LOG"
}

setup() {
  if [[ -z ${OMARCHY_PATH:-} || ! -d $OMARCHY_PATH/themes/tokyo-night ]]; then
    echo "759 needs Omarchy installed: no \$OMARCHY_PATH/themes/tokyo-night" >&2
    return 1
  fi
  # The scenario is sourced before the run's folders exist.
  FAKE_HOME=$E2E_WORK/omarchy-home
  FAKE_REFRESH_LOG=$E2E_WORK/refresh.log
  THEMES=$E2E_PROFILE/config/themes
  SETTINGS=$E2E_PROFILE/config/settings.json
  user_omarchy_listing >"$E2E_WORK/user-omarchy-before.txt"
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  cat >"$E2E_WORK/repo/main.rs" <<'RUST'
// The palette, as the editor draws it.
fn main() {
    let name = "Omarchy";
    let count: u32 = 16;
    println!("{name} has {count} colours");
}
RUST
  cat >"$E2E_WORK/repo/colors.sh" <<'COLORS'
for row in 0 8; do
  for index in $(seq "$row" $((row + 7))); do printf '\e[48;5;%sm %2s \e[0m' "$index" "$index"; done
  echo
done
for index in $(seq 0 15); do printf '\e[38;5;%sm%2s \e[0m' "$index" "$index"; done
echo
COLORS
  write_fakes
  export MARLEY_OMARCHY_REFRESH=$E2E_WORK/bin/omarchy-theme-refresh
  profile_setting theme '"Gruvbox Dark"'
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# What `theme` says in the run's settings, comments and trailing commas out.
theme_setting() {
  python3 - "$SETTINGS" <<'PY'
import json, re, sys
text = open(sys.argv[1]).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
print(json.dumps(json.loads(text).get("theme")))
PY
}

theme_is() { [[ $(theme_setting) == "$1" ]]; }
fake_home_empty() { [[ -z $(find "$FAKE_HOME" -mindepth 1 2>/dev/null) ]]; }
refreshes() { [[ $(wc -l <"$FAKE_REFRESH_LOG") -eq $1 ]]; }
installed() {
  [[ -f $FAKE_HOME/.config/omarchy/themed/marley.json.tpl && -x $FAKE_HOME/.config/omarchy/hooks/theme-set.d/marley-theme && -f $THEMES/omarchy.json ]]
}
removed() {
  [[ ! -e $FAKE_HOME/.config/omarchy/themed/marley.json.tpl && ! -e $FAKE_HOME/.config/omarchy/hooks/theme-set.d/marley-theme && ! -e $THEMES/omarchy.json ]]
}
# The copy's appearance; its first line is the template's marker comment.
appearance_is() { [[ $(sed 1d "$THEMES/omarchy.json" | python3 -c 'import json,sys; print(json.load(sys.stdin)["themes"][0]["appearance"])') == "$1" ]]; }
user_omarchy_unchanged() {
  user_omarchy_listing >"$E2E_WORK/user-omarchy-after.txt"
  cmp -s "$E2E_WORK/user-omarchy-before.txt" "$E2E_WORK/user-omarchy-after.txt"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== the toggle, off"
  palette "marley: open settings"
  settle 4
  shot 759-01-setting
  press "CTRL" w
  settle 2

  echo "== the editor and a terminal printing the 16 colours"
  press "CTRL" p
  settle 1
  type_text "main.rs"
  settle 1
  press "" Return
  settle 2
  palette "pane: split right"
  settle 2
  palette "workspace: new terminal"
  settle 3
  type_text "bash colors.sh"
  press "" Return
  settle 2

  echo "== on, without Omarchy"
  profile_setting marley.omarchy_theme true
  settle 4
  shot 759-02-absent
  expect "the fake Omarchy home is still empty" fake_home_empty
  expect "nothing ran the refresh" refreshes 0
  expect "theme is still Gruvbox Dark" theme_is '"Gruvbox Dark"'

  echo "== on, with Omarchy on tokyo-night"
  profile_setting marley.omarchy_theme false
  settle 3
  mkdir -p "$FAKE_HOME/.config/omarchy/hooks/theme-set.d" "$FAKE_HOME/.config/omarchy/themed"
  "$E2E_WORK/bin/switch" tokyo-night
  profile_setting marley.omarchy_theme true
  settle 6
  shot 759-03-on
  cat "$FAKE_REFRESH_LOG"
  expect "theme is Omarchy" theme_is '"Omarchy"'
  expect "the refresh ran once" refreshes 1
  expect "the template, the hook and the copy are in place" installed
  expect "the copy is tokyo-night's, dark" appearance_is dark

  echo "== Omarchy switches to catppuccin-latte"
  "$E2E_WORK/bin/switch" catppuccin-latte
  settle 4
  shot 759-04-switch
  expect "the copy is latte's, light" appearance_is light
  expect "the switch ran no refresh" refreshes 1

  echo "== a render with a placeholder left"
  cp "$THEMES/omarchy.json" "$E2E_WORK/copy-before.json"
  sed '0,/#[0-9a-fA-F]\{6\}/s//{{ accent }}/' \
    "$FAKE_HOME/.local/state/omarchy/current/theme/marley.json" >"$E2E_WORK/broken.json"
  cp "$E2E_WORK/broken.json" "$FAKE_HOME/.local/state/omarchy/current/theme/marley.json"
  grep -c '{{' "$FAKE_HOME/.local/state/omarchy/current/theme/marley.json"
  bash "$FAKE_HOME/.config/omarchy/hooks/theme-set.d/marley-theme"
  expect "the hook kept the last good copy" cmp -s "$E2E_WORK/copy-before.json" "$THEMES/omarchy.json"

  echo "== off"
  profile_setting marley.omarchy_theme false
  settle 4
  shot 759-05-off
  expect "theme is Gruvbox Dark again" theme_is '"Gruvbox Dark"'
  expect "the template, the hook and the copy are gone" removed
  expect "the user's own Omarchy folders are unchanged" user_omarchy_unchanged
}
