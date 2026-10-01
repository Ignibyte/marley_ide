# Marley's workflow: one recipe per command the pipeline repeats. `just` lists them.
#
# The recipes wrap and do not replace: `script/gates.sh` is the gate and writes the receipt the
# commit hook reads. Every cargo recipe first waits for the box's other cargo runs to end, since
# the target directory is shared by every project on it (CONSTITUTION §0).

set shell := ["bash", "-euo", "pipefail", "-c"]

# List the recipes.
default:
    @just --list

# Wait until no cargo runs on the box.
idle:
    @until ! pgrep -x cargo >/dev/null; do sleep 2; done

# The gate on the change: every gate on its scope, and the receipt the commit needs.
gate-diff: idle
    script/gates.sh --diff

# The same gates without a receipt, for a change with no Rust.
gate-fast: idle
    script/gates.sh --fast

# The debug `marley` binary, which the e2e scenarios run.
build: idle
    cargo build -p zed --bin marley

# The release `marley` installed with its desktop entry, under ~/.local unless `--prefix DIR`;
# `--regress` runs the golden set against it first.
install *args: idle
    script/install-marley {{ args }}

# Clippy on the named crates: every target, warnings as errors.
clippy +crates: idle
    cargo clippy {{ prepend("-p ", crates) }} --all-targets -- -D warnings

# Format the named crates.
fmt +crates: idle
    cargo fmt {{ prepend("-p ", crates) }}

# An e2e scenario, run by hand (§7): it drives the debug Marley, hidden, and shoots it.
e2e scenario:
    script/e2e.sh "{{ scenario }}"

# Marley's regression suite (#517): the golden set, or the scenarios named, each in a headless
# sway and checking itself; a line per scenario and a verdict. It waits for the box's cargo runs,
# which would slow the scenarios' timing.
regress *scenarios: idle
    script/regress {{ scenarios }}

# One shot of the debug Marley, hidden, with no input; `seed` edits the profile copy first.
shot name seed="":
    NAME="{{ name }}" SEED="{{ seed }}" script/e2e.sh script/e2e/shot.sh

# The mutation pass on this machine, or one shard of it; `script/mutants` says the rest (#638).
mutants *args: idle
    script/mutants run {{ args }}

# The mutation pass on GitHub's runners, its report downloaded (`docs/marley/mutation-runs.md`).
mutants-cloud *args:
    script/mutants cloud {{ args }}
