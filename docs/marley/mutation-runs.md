# Mutation runs

A mutation pass changes Marley's code one small way at a time (a `+` to `-`, a function's body
to a default) and runs the crate's tests against each change. A change no test notices is a
*missed mutant*: a place the tests do not hold. The workflow keeps the pass out of the gate and
runs it at the end (CONSTITUTION §0, §7); #636's run is the last one, and its findings are in
`docs/planning/design-notes/mutation-run-2026-10.md`.

The pass is slow. #636's took 1 h 27 min on the dev box for the nine crates with no gpui, two
workers at a time. `script/mutants` runs it on this machine, or splits it over GitHub's runners,
which a public repository gets free, up to 20 at once.

## On this machine

```sh
just mutants                                       # the nine pure cores, two workers
script/mutants run --packages marley_fleet         # one crate
script/mutants run --shard 0/4 --output ~/mut/q0   # a quarter of the mutants, into ~/mut/q0
script/mutants report ~/mut/q0 ~/mut/q1 ~/mut/q2 ~/mut/q3   # one table over the quarters
```

`run` mutates in copy mode: each worker copies the tree under `~/.cache/marley-mutants` and builds
in a target of its own, so the shared `CARGO_TARGET_DIR` is never touched (#443). Its outcomes go
to `--output`, by default a new `run-<stamp>` folder there, never into the repository. Without
`--packages` it takes the nine crates with no gpui: `marley_agent`, `marley_dcs`, `marley_fleet`,
`marley_mcp`, `marley_rail`, `marley_remote`, `marley_sdk`, `marley_system_one` and
`marley_terminal`. `marley_browser` and `marley_workbench` build most of Zed for every mutant;
name them only on a big machine, and time a shard first.

`report` prints a table by crate (caught, timeouts, missed, unviable, and the killed share of the
viable mutants) and every missed mutant by place. Missed mutants are findings, so a run that
completes exits 0 whatever it found.

## On GitHub's runners

```sh
just mutants-cloud                       # 16 shards of the nine crates
script/mutants cloud --shards 32         # more runners
script/mutants cloud --packages "marley_terminal marley_agent" --output ~/mut/terminal
```

`cloud` pushes HEAD to a `mutants/<stamp>` branch of the origin, with the run's settings in a
commit on top; nothing in the work tree changes, so commit first what you want mutated. The push
starts `.github/workflows/marley_mutants.yml`: a runner per shard (4 vCPUs, two workers each, the
mutants dealt round-robin so every shard gets its share of the big crates), then a job that merges
the shards into one report, shown in the run's summary. `cloud` follows the run, downloads the
report to `~/.cache/marley-mutants/cloud-<stamp>/report.md` (or `--output`), prints its table
and deletes the branch. The run and its logs are public, as the repository is.

### Once, before the first cloud run

GitHub keeps Actions off for a fork until its owner turns them on, and the fork carries Zed's own
47 workflows, which would turn on with them.

1. On github.com, open `Ignibyte/marley_ide`, the **Actions** tab, and choose **I understand my
   workflows, go ahead and enable them**.
2. Right away, run `script/mutants cloud-setup`: it turns off every workflow but Marley's, through
   GitHub's API, so pushes to the fork start nothing of Zed's.

Until then, `cloud` waits two minutes for a run, says Actions may be off, and deletes its branch.
