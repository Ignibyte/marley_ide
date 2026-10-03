# TICKET-656: A brain page's outline, and its title, name and properties edited in place

- **Ticket:** LOCAL #656 (feature, Rusty in Marley R2b: the Page tab's outline and inline edits;
  decision R-D4)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/656-brain-page-outline-and-property-edits.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (the slices table's R2b, R-D3's NoteTab row,
  R-D4, R-D10); #645's Out list and #646's D4, which hand this slice both halves; Chad, 2026-10-02
  and 2026-10-03: "Plan it now", "lets make a plan to begin the work and spec out the tickets",
  "lets make sure we use the gpui components we found here"
- **Status:** open

## Summary
#645's Page tab gains what Rusty's app has and #645 left out. In Read, a column beside the body
lists the page's headings from `brain_render`'s `outline`, indented by level, and a click brings a
heading to the top of the body; Zed's own outline panel cannot take a list from a tab that holds no
editor, so in Edit the tab hands Zed its editor instead, as Zed's Markdown preview does, and Zed's
outline panel lists the source's headings. A heading is found by its line, mapped through #645's
wikilink pass, and a small additive request in Zed's `markdown` crate lands it at the top, where
Zed's own request stops at the bottom edge. The page is also edited where it is drawn, through
Rusty's tools only: the title writes the `title` property, the name in the header renames the page
with `brain_rename` (Rusty rewrites every link, and every open Page tab follows, as they do after a
rename in the Brain view), and each property edits by its value's kind (text, number, date,
checkbox, list of text) with `brain_set_property`, with Remove (`brain_remove_property`) and Add
property. The in-place editor is Ely GPUI Components' `InlineEdit`, ported onto Zed's single-line
`Editor`. Writes go one at a time per tab and Rusty's change signal waits for them. Behind
`marley.rusty.enabled` (#643), off by default. After #645.

## Acceptance
A page with headings shows its outline beside the body; a click brings a heading, the second of two
with the same name included, to the top; the Outline button hides and shows it; in Edit, Zed's
outline panel lists the source's headings. A click on the title or a value opens an editor; Enter or
leaving writes the value as its kind (a number as a number, a date only as a date, a checkbox's
other value, a list with an item added or removed), Escape writes nothing, and a malformed number
says so. Remove and Add property work, and a taken key is refused. Renaming the page in the header
renames it through `brain_rename`, says in how many pages Rusty updated links, and every open Page
tab follows it, in what it shows and in its history; Rusty's refusal is shown. Nothing opens for
editing without the connection.
