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

# The debug `marley` binary, which the e2e tests run.
build: idle
    cargo build -p zed --bin marley

# The release `marley` installed with its desktop entry, under ~/.local unless `--prefix DIR`.
install *args: idle
    script/install-marley {{ args }}

# Clippy on the named crates: every target, warnings as errors.
clippy +crates: idle
    cargo clippy {{ prepend("-p ", crates) }} --all-targets -- -D warnings

# Format the named crates.
fmt +crates: idle
    cargo fmt {{ prepend("-p ", crates) }}

# A ticket's e2e test (§7): its scenario drives the debug Marley, hidden, and shoots it.
e2e scenario:
    script/e2e.sh "{{ scenario }}"

# One shot of the debug Marley, hidden, with no input; `seed` edits the profile copy first.
shot name seed="":
    NAME="{{ name }}" SEED="{{ seed }}" script/e2e.sh script/e2e/shot.sh
