#!/usr/bin/env bash
# Marley's theme hook (marley.omarchy_theme, #759). Omarchy runs it after each theme switch: it
# copies the theme Omarchy rendered from Marley's template into Marley's themes folder, where
# Marley reloads it. Marley writes this file, and removes it when the setting is turned off.
render=@RENDER@
copy=@COPY@
[[ -f $render ]] || exit 0
# A render with a placeholder left, a colour Omarchy could not fill, keeps the last good copy.
grep -q '{{' "$render" && exit 0
folder=$(dirname "$copy")
mkdir -p "$folder" || exit 0
temporary=$(mktemp "$folder/.omarchy.json.XXXXXX") || exit 0
if cp "$render" "$temporary"; then
  mv "$temporary" "$copy"
else
  rm -f "$temporary"
fi
