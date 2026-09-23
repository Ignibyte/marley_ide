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

# The gate on the change: the static gates on its scope, coverage on its Marley crates.
gate-diff: idle
    script/gates.sh --diff

# The static gates alone, for a change with no Rust.
gate-fast: idle
    script/gates.sh --fast

# The full audit over every Marley crate.
gate-full: idle
    script/gates.sh --full

# The debug `marley` binary, for a live drive.
build: idle
    cargo build -p zed --bin marley

# The tests of the named crates.
test +crates: idle
    cargo nextest run {{ prepend("-p ", crates) }}

# Clippy on the named crates: every target, warnings as errors.
clippy +crates: idle
    cargo clippy {{ prepend("-p ", crates) }} --all-targets -- -D warnings

# Format the named crates.
fmt +crates: idle
    cargo fmt {{ prepend("-p ", crates) }}

# The live drive's capture, no input sent: Marley on a profile copy, hidden; `seed` edits the copy,
# and OPEN in the environment names a path to open.
shot name seed="":
    SEED="{{ seed }}" script/live-shot.sh "{{ name }}"
