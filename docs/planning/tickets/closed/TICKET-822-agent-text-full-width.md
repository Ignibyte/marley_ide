# TICKET-822 — An agent's text runs the full width, with a toggle in its bottom row

- **Ticket:** LOCAL #822 (feature, the agent thread view and #798's width button)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** ../../pipeline/completed/822-agent-text-full-width.spec.md
- **Source ticket:** Chad, 2026-10-20, over a shot of a thread tab whose text ran in an 850 px
  column: "the small strip in the middle the agent response i want to be flexible and default it
  should be 100%. See all that extra space?", then "can we put that setting into the bottom panel
  like you said. If the default zed setting conflicts we overwrite it." He picked "Own toggle":
  an icon button that always shows beside the width button. (His own settings got
  `agent.limit_content_width: false` the same morning, through Marley's settings_change.)
- **Status:** closed

## Summary
Marley's default for `agent.limit_content_width` becomes `false`, overriding Zed's `true`, so a
thread's messages and message box run the full width of its tab or panel. A toggle in the
thread's bottom row, beside #798's width button and shown always, flips the setting between the
full width and the readable column of `agent.max_content_width`, and writes it to the user's
settings.

## Acceptance
A new Marley's thread text runs the full width; the toggle narrows it to the column and back, and
the user's settings file holds the choice.
