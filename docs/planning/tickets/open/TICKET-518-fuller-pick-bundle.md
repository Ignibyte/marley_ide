# TICKET-518 — A fuller pick bundle: HTML, styles and the React component

- **Ticket:** LOCAL #518 (feature, prong 3, before #505)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/518-fuller-pick-bundle.spec.md
- **Source ticket:** Chad, 2026-09-25: the Orca survey's items are taken (his decision on the survey, relayed with the night's drafting brief). This is its "fuller pick" (`docs/orca_architecture/README.md`, "What Orca does well that Marley lacks, first", item 4, and "New tickets these imply": "a fuller pick bundle (item 4, before #505)"; report 03 §2.4 and item 1)
- **Status:** open

## Summary
A pick gives the agent the element's locators, listeners, blockers, box and crop, but not its
HTML or its styles, the two things an agent most needs to change how an element looks, and on a
React app the listeners' sources land in react-dom rather than in the component that rendered the
element. The pick bundle gains the element's HTML (scripts removed, cut at 4,096 characters),
sixteen computed styles, the texts beside it and the page's selection, and on a React dev build
the component chain around the element with the file and line it was written at: `_debugSource`
on React 18 and older, and on React 19 the first frame of `_debugStack` outside React, followed
through its script's source map to the file in the project. Field values, URL query strings,
scripts and secret-looking attribute values never enter it. `browser_pick` returns the new
fields; the tray and the pick line stay as they are. #505's check compares against them.

## Acceptance
`browser_pick` on a React 18 and on a React 19 dev page returns the component chain and the
source file and line in the project; on any page it returns the HTML and the computed styles, and
the HTML holds no script, field value, query string or secret-looking value.
