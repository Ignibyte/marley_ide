// Marley's Omarchy theme (marley.omarchy_theme). Marley rewrites this file while this line is here; delete the line to keep your edits.
{
  "$schema": "https://zed.dev/schema/themes/v0.2.0.json",
  "name": "Omarchy",
  "author": "Omarchy, through Marley",
  "themes": [
    {
      "name": "Omarchy",
      "appearance": "{{ mode }}",
      "style": {
        "background": "{{ background }}",
        "surface.background": "{{ mix background foreground 4% }}",
        "elevated_surface.background": "{{ mix background foreground 7% }}",
        "panel.background": "{{ mix background foreground 3% }}",
        "status_bar.background": "{{ mix background foreground 3% }}",
        "title_bar.background": "{{ mix background foreground 3% }}",
        "title_bar.inactive_background": "{{ background }}",
        "toolbar.background": "{{ background }}",
        "tab_bar.background": "{{ mix background foreground 3% }}",
        "tab.inactive_background": "{{ mix background foreground 3% }}",
        "tab.active_background": "{{ background }}",
        "border": "{{ mix background foreground 15% }}",
        "border.variant": "{{ mix background foreground 9% }}",
        "border.focused": "{{ accent }}",
        "border.selected": "{{ accent }}",
        "border.transparent": "#00000000",
        "border.disabled": "{{ mix background foreground 8% }}",
        "element.background": "{{ mix background foreground 8% }}",
        "element.hover": "{{ mix background foreground 12% }}",
        "element.active": "{{ mix background foreground 16% }}",
        "element.selected": "{{ mix background accent 25% }}",
        "element.disabled": "{{ mix background foreground 5% }}",
        "drop_target.background": "{{ accent }}40",
        "ghost_element.background": "#00000000",
        "ghost_element.hover": "{{ mix background foreground 10% }}",
        "ghost_element.active": "{{ mix background foreground 14% }}",
        "ghost_element.selected": "{{ mix background accent 20% }}",
        "ghost_element.disabled": "#00000000",
        "text": "{{ foreground }}",
        "text.muted": "{{ mix foreground background 35% }}",
        "text.placeholder": "{{ dark_foreground }}",
        "text.disabled": "{{ mix muted background 40% }}",
        "text.accent": "{{ accent }}",
        "icon": "{{ foreground }}",
        "icon.muted": "{{ mix foreground background 35% }}",
        "icon.disabled": "{{ mix muted background 40% }}",
        "icon.placeholder": "{{ dark_foreground }}",
        "icon.accent": "{{ accent }}",
        "search.match_background": "{{ accent }}40",
        "search.active_match_background": "{{ accent }}70",
        "panel.focused_border": "{{ accent }}",
        "pane.focused_border": "{{ accent }}",
        "scrollbar.thumb.background": "{{ muted }}40",
        "scrollbar.thumb.hover_background": "{{ muted }}70",
        "scrollbar.thumb.border": "#00000000",
        "scrollbar.track.background": "#00000000",
        "scrollbar.track.border": "{{ mix background foreground 6% }}",
        "editor.foreground": "{{ foreground }}",
        "editor.background": "{{ background }}",
        "editor.gutter.background": "{{ background }}",
        "editor.subheader.background": "{{ mix background foreground 7% }}",
        "editor.active_line.background": "{{ mix background foreground 6% }}",
        "editor.highlighted_line.background": "{{ mix background foreground 6% }}",
        "editor.line_number": "{{ muted }}",
        "editor.active_line_number": "{{ foreground }}",
        "editor.hover_line_number": "{{ foreground }}",
        "editor.invisible": "{{ mix background muted 50% }}",
        "editor.wrap_guide": "{{ mix background foreground 8% }}",
        "editor.active_wrap_guide": "{{ mix background foreground 15% }}",
        "editor.document_highlight.read_background": "{{ accent }}25",
        "editor.document_highlight.write_background": "{{ accent }}35",
        "terminal.background": "{{ background }}",
        "terminal.foreground": "{{ foreground }}",
        "terminal.bright_foreground": "{{ bright_foreground }}",
        "terminal.dim_foreground": "{{ muted }}",
        "terminal.ansi.black": "{{ color0 }}",
        "terminal.ansi.bright_black": "{{ color8 }}",
        "terminal.ansi.dim_black": "{{ mix color0 background 30% }}",
        "terminal.ansi.red": "{{ color1 }}",
        "terminal.ansi.bright_red": "{{ color9 }}",
        "terminal.ansi.dim_red": "{{ mix color1 background 30% }}",
        "terminal.ansi.green": "{{ color2 }}",
        "terminal.ansi.bright_green": "{{ color10 }}",
        "terminal.ansi.dim_green": "{{ mix color2 background 30% }}",
        "terminal.ansi.yellow": "{{ color3 }}",
        "terminal.ansi.bright_yellow": "{{ color11 }}",
        "terminal.ansi.dim_yellow": "{{ mix color3 background 30% }}",
        "terminal.ansi.blue": "{{ color4 }}",
        "terminal.ansi.bright_blue": "{{ color12 }}",
        "terminal.ansi.dim_blue": "{{ mix color4 background 30% }}",
        "terminal.ansi.magenta": "{{ color5 }}",
        "terminal.ansi.bright_magenta": "{{ color13 }}",
        "terminal.ansi.dim_magenta": "{{ mix color5 background 30% }}",
        "terminal.ansi.cyan": "{{ color6 }}",
        "terminal.ansi.bright_cyan": "{{ color14 }}",
        "terminal.ansi.dim_cyan": "{{ mix color6 background 30% }}",
        "terminal.ansi.white": "{{ color7 }}",
        "terminal.ansi.bright_white": "{{ color15 }}",
        "terminal.ansi.dim_white": "{{ mix color7 background 30% }}",
        "link_text.hover": "{{ accent }}",
        "version_control.added": "{{ green }}",
        "version_control.modified": "{{ yellow }}",
        "version_control.deleted": "{{ red }}",
        "version_control.word_added": "{{ green }}40",
        "version_control.word_deleted": "{{ red }}40",
        "version_control.conflict_marker.ours": "{{ green }}30",
        "version_control.conflict_marker.theirs": "{{ blue }}30",
        "conflict": "{{ orange }}",
        "conflict.background": "{{ orange }}20",
        "conflict.border": "{{ orange }}60",
        "created": "{{ green }}",
        "created.background": "{{ green }}20",
        "created.border": "{{ green }}60",
        "deleted": "{{ red }}",
        "deleted.background": "{{ red }}20",
        "deleted.border": "{{ red }}60",
        "error": "{{ red }}",
        "error.background": "{{ red }}20",
        "error.border": "{{ red }}60",
        "hidden": "{{ dark_foreground }}",
        "hidden.background": "{{ muted }}20",
        "hidden.border": "{{ muted }}60",
        "hint": "{{ dark_foreground }}",
        "hint.background": "{{ muted }}20",
        "hint.border": "{{ muted }}60",
        "ignored": "{{ dark_foreground }}",
        "ignored.background": "{{ muted }}20",
        "ignored.border": "{{ muted }}60",
        "info": "{{ blue }}",
        "info.background": "{{ blue }}20",
        "info.border": "{{ blue }}60",
        "modified": "{{ yellow }}",
        "modified.background": "{{ yellow }}20",
        "modified.border": "{{ yellow }}60",
        "predictive": "{{ dark_foreground }}",
        "predictive.background": "{{ muted }}20",
        "predictive.border": "{{ muted }}60",
        "renamed": "{{ blue }}",
        "renamed.background": "{{ blue }}20",
        "renamed.border": "{{ blue }}60",
        "success": "{{ green }}",
        "success.background": "{{ green }}20",
        "success.border": "{{ green }}60",
        "unreachable": "{{ dark_foreground }}",
        "unreachable.background": "{{ muted }}20",
        "unreachable.border": "{{ muted }}60",
        "warning": "{{ yellow }}",
        "warning.background": "{{ yellow }}20",
        "warning.border": "{{ yellow }}60",
        "players": [
          {
            "cursor": "{{ accent }}",
            "background": "{{ accent }}",
            "selection": "{{ accent }}40"
          },
          {
            "cursor": "{{ blue }}",
            "background": "{{ blue }}",
            "selection": "{{ blue }}40"
          },
          {
            "cursor": "{{ green }}",
            "background": "{{ green }}",
            "selection": "{{ green }}40"
          },
          {
            "cursor": "{{ yellow }}",
            "background": "{{ yellow }}",
            "selection": "{{ yellow }}40"
          },
          {
            "cursor": "{{ magenta }}",
            "background": "{{ magenta }}",
            "selection": "{{ magenta }}40"
          },
          {
            "cursor": "{{ cyan }}",
            "background": "{{ cyan }}",
            "selection": "{{ cyan }}40"
          },
          {
            "cursor": "{{ orange }}",
            "background": "{{ orange }}",
            "selection": "{{ orange }}40"
          },
          {
            "cursor": "{{ red }}",
            "background": "{{ red }}",
            "selection": "{{ red }}40"
          }
        ],
        "syntax": {
          "attribute": {
            "color": "{{ yellow }}",
            "font_style": null,
            "font_weight": null
          },
          "boolean": {
            "color": "{{ orange }}",
            "font_style": null,
            "font_weight": null
          },
          "comment": {
            "color": "{{ dark_foreground }}",
            "font_style": "italic",
            "font_weight": null
          },
          "comment.doc": {
            "color": "{{ dark_foreground }}",
            "font_style": "italic",
            "font_weight": null
          },
          "constant": {
            "color": "{{ orange }}",
            "font_style": null,
            "font_weight": null
          },
          "constructor": {
            "color": "{{ yellow }}",
            "font_style": null,
            "font_weight": null
          },
          "embedded": {
            "color": "{{ foreground }}",
            "font_style": null,
            "font_weight": null
          },
          "emphasis": {
            "color": "{{ accent }}",
            "font_style": "italic",
            "font_weight": null
          },
          "emphasis.strong": {
            "color": "{{ orange }}",
            "font_style": null,
            "font_weight": 700
          },
          "enum": {
            "color": "{{ yellow }}",
            "font_style": null,
            "font_weight": null
          },
          "function": {
            "color": "{{ blue }}",
            "font_style": null,
            "font_weight": null
          },
          "hint": {
            "color": "{{ dark_foreground }}",
            "font_style": null,
            "font_weight": null
          },
          "keyword": {
            "color": "{{ bright_magenta }}",
            "font_style": null,
            "font_weight": null
          },
          "label": {
            "color": "{{ blue }}",
            "font_style": null,
            "font_weight": null
          },
          "link_text": {
            "color": "{{ blue }}",
            "font_style": "italic",
            "font_weight": null
          },
          "link_uri": {
            "color": "{{ cyan }}",
            "font_style": null,
            "font_weight": null
          },
          "namespace": {
            "color": "{{ yellow }}",
            "font_style": null,
            "font_weight": null
          },
          "number": {
            "color": "{{ orange }}",
            "font_style": null,
            "font_weight": null
          },
          "operator": {
            "color": "{{ cyan }}",
            "font_style": null,
            "font_weight": null
          },
          "predictive": {
            "color": "{{ dark_foreground }}",
            "font_style": "italic",
            "font_weight": null
          },
          "preproc": {
            "color": "{{ magenta }}",
            "font_style": null,
            "font_weight": null
          },
          "primary": {
            "color": "{{ foreground }}",
            "font_style": null,
            "font_weight": null
          },
          "property": {
            "color": "{{ cyan }}",
            "font_style": null,
            "font_weight": null
          },
          "punctuation": {
            "color": "{{ foreground }}",
            "font_style": null,
            "font_weight": null
          },
          "punctuation.bracket": {
            "color": "{{ mix foreground background 35% }}",
            "font_style": null,
            "font_weight": null
          },
          "punctuation.delimiter": {
            "color": "{{ mix foreground background 35% }}",
            "font_style": null,
            "font_weight": null
          },
          "punctuation.list_marker": {
            "color": "{{ accent }}",
            "font_style": null,
            "font_weight": null
          },
          "punctuation.markup": {
            "color": "{{ accent }}",
            "font_style": null,
            "font_weight": null
          },
          "punctuation.special": {
            "color": "{{ magenta }}",
            "font_style": null,
            "font_weight": null
          },
          "selector": {
            "color": "{{ magenta }}",
            "font_style": null,
            "font_weight": null
          },
          "selector.pseudo": {
            "color": "{{ cyan }}",
            "font_style": null,
            "font_weight": null
          },
          "string": {
            "color": "{{ green }}",
            "font_style": null,
            "font_weight": null
          },
          "string.escape": {
            "color": "{{ cyan }}",
            "font_style": null,
            "font_weight": null
          },
          "string.regex": {
            "color": "{{ orange }}",
            "font_style": null,
            "font_weight": null
          },
          "string.special": {
            "color": "{{ orange }}",
            "font_style": null,
            "font_weight": null
          },
          "string.special.symbol": {
            "color": "{{ orange }}",
            "font_style": null,
            "font_weight": null
          },
          "tag": {
            "color": "{{ red }}",
            "font_style": null,
            "font_weight": null
          },
          "text.literal": {
            "color": "{{ green }}",
            "font_style": null,
            "font_weight": null
          },
          "title": {
            "color": "{{ blue }}",
            "font_style": null,
            "font_weight": 700
          },
          "type": {
            "color": "{{ yellow }}",
            "font_style": null,
            "font_weight": null
          },
          "variable": {
            "color": "{{ foreground }}",
            "font_style": null,
            "font_weight": null
          },
          "variable.parameter": {
            "color": "{{ foreground }}",
            "font_style": null,
            "font_weight": null
          },
          "variable.special": {
            "color": "{{ red }}",
            "font_style": null,
            "font_weight": null
          },
          "variant": {
            "color": "{{ blue }}",
            "font_style": null,
            "font_weight": null
          },
          "diff.plus": {
            "color": "{{ green }}",
            "font_style": null,
            "font_weight": null
          },
          "diff.minus": {
            "color": "{{ red }}",
            "font_style": null,
            "font_weight": null
          }
        }
      }
    }
  ]
}
