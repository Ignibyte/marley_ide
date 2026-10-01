# `marley_system_one`

The System One layer's pure core (#565): typed questions about a state Marley builds, the
`/v1/systemone` request and its answer, the reading a use may act on, the policy that decides
whether a call goes out, and the files that keep each call. It has no gpui, no HTTP and no clock:
the workbench's adapter (`marley_workbench::system_one`) sends the request, holds the key, keeps
the time and names the folder. MIT OR Apache-2.0, with rustal's lint table; its dependencies are
`serde`, `serde_json` and `sha2`.

## What it holds

- **The root.** `QuestionSet { id, model, questions }` is compiled in, named and versioned
  (`check/1`): a change to a question makes a new version, so a logged call always names what it
  asked. `Question` is a `Noul` (its key, what is asked, what yes and no mean), a `Choice` (at most
  255 options, one of them `cannot_tell` wherever the state may not settle it) or a `Score` (2 to
  10 levels). `UseSpec { name, set, deadline }` is a use. `DEFAULT_MODEL` is `jev-1.13.0`, and
  `CHECK_SET` and `CHECK` are the check's: one noul, `command_failed`, with a 2 s deadline.
- **The stop kind's sets** (#566). A caller never supplies its own questions, so a use whose
  questions depend on its input has a set for each shape. `STOP_KIND_SETS` holds seven,
  `stop_kind_0/1` to `stop_kind_6/1`, by the number of parts of the prompt (`MAX_PARTS` is six):
  the `kind` choice (`done_checked`, `done_claimed`, `asks_you`, `blocked`, `still_going`,
  `cannot_tell`) and a noul per part, `part_1_done` to `part_N_done`, about the part of the
  request the state labels `part N`. The parts are the state's text, so they are masked and cut
  like any text, and a metadata-only project, whose state has no text, asks `stop_kind_0/1`.
  `STOP_KIND` holds the seven `UseSpec`s, all named `stop_kind` with a 2 s deadline, as statics,
  so the one `stop_kind(parts)` picks at run time is `&'static`.
- **The find sets** (#567). `find_set(items)` answers one of 254 sets, `find_1/1` to `find_254/1`,
  one per window of items (`FIND_WINDOW` is 254, since a choice holds 255 options and `none` is
  one): a choice `which` over `none` and `1` to `N` ("Which item in the state matches its `query`
  line?") and a noul `present` ("Does any item match the query?"). The options are one table,
  `none` first, so each set's are a prefix of it; the names, their meanings, the questions and
  the sets are `static LazyLock` tables built once, whose strings a static holds, so nothing is
  leaked. `find_label(item)` gives the state's label for an item, the same as its option.
- **The stall kind's set** (#569). `STALL_KIND_SET` (`stall_kind/1`) asks about a working agent
  that has gone quiet: the choice `kind` (`long_task`, `waiting_for_input`, `stuck`, `frozen`,
  `cannot_tell`) and the nouls `repeating` and `progress`. `STALL_KIND` is its use, `stall_kind`,
  with a 2 s deadline. A loop is the use's own rule, logged as a `rules` row with `repeating`
  held.
- **The click consequence's set** (#571). `CLICK_CONSEQUENCE_SET` (`click_consequence/1`) asks
  four nouls about an element an agent is about to click: `pays`, `deletes`, `sends` and
  `changes_account`. `CLICK_CONSEQUENCE` is its use, `click_consequence`, with a 1.5 s deadline,
  inside the pause a click can afford. A rule's pause is logged as a `rules` row with its noul held.
- **The inbox's set** (#568). `INBOX_RISK_SET` (`inbox_risk/1`) asks about an action waiting in
  the rail's inbox that Marley's rules found nothing on: a noul for each chip a tool's action can
  carry (`destroys`, `credentials`, `rewrites_history`, `sends_out`, `installs`,
  `outside_project`, `claims_approval`) and `urgency`, a score of five levels from a routine read
  inside the project to an action that destroys data or cannot be undone, the first set to ask a
  score. `INBOX_RISK` is its use, `inbox`, with a 600 ms deadline. The rules' chips are logged
  as one `rules` row with each chip's noul held.
- **The question route's set** (#570). `QUESTION_ROUTE_SET` (`question_route/1`) asks who should
  answer what an agent waits on: the choice `route` (`owner`, `manager`, `agent_proceeds`,
  `cannot_tell`) and the noul `answerable_from_prompt`. `QUESTION_ROUTE` is its use,
  `question_route`, with a 2 s deadline, since the agent already waits. A rule's mark is logged
  as a `rules` row with the choice it settled.
- **The running error's set** (#572). `RUNNING_ERROR_SET` (`running_error/1`) asks two nouls
  about a line a running command printed that Marley's shapes left open, with the lines before
  and after it: `new_failure` and `recovered`. `RUNNING_ERROR` is its use, `running_error`, with a
  2 s deadline. A shape's flag and recovery are logged as `rules` rows with their noul held.
- **The typed line's set** (#573). `TYPED_LINE_SET` (`typed_line/1`) asks one choice, `kind`,
  about a line at a shell's prompt the rules leave open: `command`, `request`, `comment`,
  `command_then_english` or `cannot_tell`. `TYPED_LINE` is its use, `typed_line`, with a 600 ms
  deadline, since the user is typing; #557's reading is its verdict.
- **`state`.** `StateBuilder::new(detail, mask)` takes facts, kept at every `Detail`, and text,
  left out at `Detail::Facts`, each value through the host's mask. A text value is masked whole
  and then cut to 300 characters (`cut`), since a cut can split a secret the mask would find.
  `build` renders `label: value` lines with their SHA-256, or answers `None` for an empty state,
  which is refused, never asked.
- **`request`.** `build(model, state, set)` renders `{model, state, questions}`: a noul's
  `criteria` as `{true, false}`, a choice's as a map of option to meaning, a score's as an array of
  its levels. `parse(body)` reads the answer as Jev gives it, the shape Chad's own recorded runs
  and working client showed: a noul's `noul`; a choice's `choice`, `confidence` and
  `probabilities`; a score's fractional `score` (the expected level), `confidence`, `probabilities`
  by level and `legend`, kept as JSON; then `model` and `usage.input_tokens`. Unknown fields are
  ignored, and an answer of an unknown kind is listed in `unreadable`. `error_excerpt` cuts an
  error body to 300 characters, since a body can echo the request.
- **`reading`.** `read(set, answers)` reads each question against its threshold. A choice or a
  score counts at a confidence of 0.5 or more, and never for `cannot_tell` or `none`; a noul
  between 0.35 and 0.65, the band widened by 1e-9, is no signal. `Reading` is `Off`, `Rules`,
  `Model`, `Refused` or `Unavailable`, never an error, and `summary` is its line: `command failed:
  yes (0.92)`, `Refused: project not listed`.
- **`policy`.** `may_send(folders, local, projects, metadata_only)` decides what a project may
  send. A remote project sends nothing; a folder on the metadata-only list lets a project send
  facts alone, and one on the projects list facts and text; the metadata-only list wins. `cost`
  and `budget` count billionths of a cent, since a 1,200-token call at Jev's 4.2 cents per million
  costs about 0.005 cents. `Gate::admit` holds a call back past the day's budget, while the breaker
  is open (two minutes after five failures in a row), past 1,000 calls a minute, and for a
  subject's state already answered. `answered` and `failed` tell the gate what came of a call.
- **`files`.** A `CallRow` holds a call's id, time, use, set, model, provider, project, mode and
  verdict, the masked state and its hash, the questions, the answers, the reading, whether it
  failed, the thresholds, the latency, the tokens, the cost and any error. An `OutcomeRow` is a
  later outcome naming a call, and both are `Row` lines. `append_in(dir, day, row)` appends to
  `calls-<day>.jsonl`, the file readable by its owner alone, and `read_day_in` reads a day back.
  `Replay::load_in(dir)` reads `replay.jsonl`, and `answer(set, state)` gives the first row whose
  set, `match` text and `state_hash` fit, once unless the row repeats.

## Why a crate of its own

Plan D8, the house pattern: a pure core the uses (#566 to #571) share, apart from the network, the
keychain and the settings, which the workbench owns.

## Consumers

`marley_workbench::system_one` (the adapter and the check), `marley_workbench::decisions` (the
view), since #566 `marley_workbench::agent_events` (the stop kind), since #567
`marley_workbench::find` (the find tools), since #569 `marley_workbench::stall` (the stall
kind), since #571 `marley_workbench::click_pause` (the click consequence), since #572
`marley_workbench::running_errors` (the running error), and since #573
`marley_workbench::typed_line` (the typed line).

## Tests

None written (§7). `script/e2e/565-system-one-layer.sh` drives the crate through the app against a
fake `/v1/systemone`, `script/e2e/566-stop-kind.sh` the stop kind's sets on the `replay`
provider, `script/e2e/567-find-tools.sh` the find sets on it too, and
`script/e2e/569-stalled-or-looping-agents.sh` the stall kind's set, and
`script/e2e/571-pause-before-a-consequential-click.sh` the click consequence's.
