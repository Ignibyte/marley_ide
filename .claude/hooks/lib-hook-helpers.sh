#!/usr/bin/env bash
# =============================================================================
# lib-hook-helpers.sh — Shared helpers for the Marley enforcement hooks
# =============================================================================
#
# Source from other hooks: source "$(dirname "$0")/lib-hook-helpers.sh"
#
# Shared transcript-reader + phase-detection helpers. Bash 3.2 compatible
# (macOS /bin/bash) — no mapfile/readarray/declare -A/${x,,}. Transcript
# readers use the canonical `jq -s '[.[] | (.message // .) | ...]'` slurp
# pattern — Claude Code writes JSONL where each line is
# `{ "type": ..., "message": { "role", "content" } }`; `(.message // .)`
# unwraps that and falls back to legacy top-level shape. Never use bare
# `jq '.[]'` (silently errors per-line and the gate goes dormant).
# =============================================================================

# Git root so hooks work regardless of CWD; fall back to pwd.
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

# Strip the project-root prefix (and any worktree prefix) for pattern matching.
normalize_path() {
    echo "$1" \
        | sed "s|^${PROJECT_ROOT}/||" \
        | sed 's|^\.claude/worktrees/[A-Za-z0-9._-]\{1,\}/||'
}

# --- Gate receipt fingerprint ------------------------------------------------
# Content fingerprint of all gated + gate-DEFINING files: HEAD + the worktree
# content of every tracked-or-untracked `crates/**/*.rs` (CONSTITUTION §3
# application code, Zed's crates included), every file under `crates/marley_*`,
# AND the files that define the bar itself — `script/gates.sh`,
# `.claude/hooks/*.sh`, `clippy.toml`, `rustfmt.toml`, `deny.toml`,
# `.gitleaks.toml`, `.semgrep.yml`, `.config/typos.toml`, `.cargo/audit.toml`,
# `.cargo/config.toml`, the Cargo manifests + lockfile, the toolchain pin, the
# nextest config, `tooling/lints` (gate:21's library and its nightly pin), and
# `vendor/` (the upstream crates the build takes through `[patch]`) — so a
# post-green *weakening of the gate*
# invalidates the receipt just as a code edit does. script/gates.sh writes this
# to .git/ignibyte-gate-receipt on a FULL/DIFF green; enforce-commit-gate.sh
# recomputes it at `git commit` and allows the commit only if they match, so a
# green binds to the EXACT worktree it ran on. Hashing tracked+untracked file
# CONTENT (not `git diff`) makes it invariant to `git add` staging. The
# per-file hashes come from one `git hash-object --stdin-paths` (thousands of
# files in a Zed tree; one process, not one per file). Paths are listed
# NUL-separated (no quoting) and joined by newlines for hash-object, which has
# no -z switch in git 2.55; a newline inside a file name is the one case that
# would break it, and git refuses such names in this tree anyway.
gate_state_hash() {
    local root="${PROJECT_ROOT:-$(pwd)}" paths hashes n_paths n_hashes
    paths=$({
        git -C "$root" ls-files -z -- crates script/gates.sh .claude/hooks clippy.toml rustfmt.toml deny.toml .gitleaks.toml .semgrep.yml .config/typos.toml .cargo/audit.toml .cargo/config.toml Cargo.toml Cargo.lock rust-toolchain.toml .config/nextest.toml tooling/lints vendor 2>/dev/null
        git -C "$root" ls-files -z --others --exclude-standard -- crates script/gates.sh .claude/hooks clippy.toml rustfmt.toml deny.toml .gitleaks.toml .semgrep.yml .config/typos.toml .cargo/audit.toml .cargo/config.toml Cargo.toml Cargo.lock rust-toolchain.toml .config/nextest.toml tooling/lints vendor 2>/dev/null
    } | LC_ALL=C sort -z -u | grep -zE '^crates/marley_|\.(rs|sh|toml|lock)$|^\.semgrep\.yml$' | tr '\0' '\n')
    # One git process hashes every file; a path list and a hash list of different
    # lengths means git failed on something, and a fingerprint that covers less
    # than the tree is worse than none — so the failure returns a sentinel that
    # can never match a receipt (the hook then blocks, the gate then re-runs).
    hashes=$(printf '%s\n' "$paths" | (cd "$root" && git hash-object --stdin-paths))
    n_paths=$(printf '%s\n' "$paths" | grep -c .)
    n_hashes=$(printf '%s\n' "$hashes" | grep -c .)
    if [ "$n_paths" -eq 0 ] || [ "$n_paths" -ne "$n_hashes" ]; then
        echo "fingerprint-failed-${n_paths}-${n_hashes}-$(date +%s%N)"
        return 1
    fi
    {
        git -C "$root" rev-parse HEAD 2>/dev/null || printf 'no-head\n'
        printf '%s\n' "$hashes"
    } | if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi | awk '{print $1}'
}

# --- Transcript readers ------------------------------------------------------

extract_bash_commands() {
    local t="$1"; [ -f "$t" ] || return 0
    jq -sr '[.[] | (.message // .) | select(.role=="assistant") |
            (.content // [])[] | select(.type=="tool_use") |
            select(.name=="Bash") | .input.command // empty] | .[]' "$t" 2>/dev/null || true
}

extract_user_texts() {
    local t="$1"; [ -f "$t" ] || return 0
    jq -sr '[.[] | (.message // .) | select(.role=="user") | (.content // []) |
            if type=="array" then (.[] | select(.type=="text") | .text // empty)
            elif type=="string" then . else empty end] | .[]' "$t" 2>/dev/null || true
}

# Every assistant tool_use name (Bash, Edit, Write, TaskCreate, MCP-stripped).
extract_assistant_tool_uses() {
    local t="$1"; [ -f "$t" ] || return 0
    jq -sr '[.[] | (.message // .) | select(.role=="assistant") |
            (.content // [])[] | select(.type=="tool_use") | .name // empty |
            sub("^mcp__.*__";"")] | .[]' "$t" 2>/dev/null || true
}

# The Ignibyte pipeline command alphabet (used by the two detectors below).
# Phases: plan code test complete (CONSTITUTION §3). Plus /spec, the batch planner.
_PIPE_RE='(pipeline[-:](plan|code|test|complete)|spec)'

# Most-recent slash command of ANY kind (returns bare name or "").
latest_pipeline_command() {
    local t="$1"; [ -f "$t" ] || { echo ""; return; }
    local m
    m=$(jq -sr --arg re "^\\s*/$_PIPE_RE\\b" '
        [.[] | (.message // .) |
         if .role=="user" then ((.content // []) |
             if type=="array" then (.[] | select(.type=="text") | .text // empty)
             elif type=="string" then . else empty end)
         elif .role=="assistant" then ((.content // [])[]? |
             select(.type=="tool_use" and .name=="Skill") | "/" + (.input.skill // ""))
         else empty end] | map(select(test($re))) | last // empty' "$t" 2>/dev/null || true)
    [ -n "$m" ] && echo "$m" | grep -oE "/$_PIPE_RE" | tail -1 | sed 's#^/##' || true
}

# Bare PIPELINE PHASE name (plan|code|test|complete), or "" when the latest
# slash command is NOT a pipeline phase (e.g. /spec).
detect_active_command() {
    local t="$1"; [ -f "$t" ] || { echo ""; return; }
    local m
    m=$(jq -sr --arg re "^\\s*/?$_PIPE_RE\\b" '
        [.[] | (.message // .) |
         if .role=="user" then ((.content // []) |
             if type=="array" then (.[] | select(.type=="text") | .text // empty)
             elif type=="string" then . else empty end)
         elif .role=="assistant" then ((.content // [])[]? |
             select(.type=="tool_use" and .name=="Skill") | "/" + (.input.skill // ""))
         else empty end] | map(select(test($re))) | last // empty' "$t" 2>/dev/null || true)
    [ -n "$m" ] && echo "$m" | grep -oE 'pipeline[-:](plan|code|test|complete)' | tail -1 | sed 's/^pipeline[-:]//' || true
}

# 0-based JSONL line index of the last phase-advance (user /pipeline:<phase> or
# assistant Skill pipeline-<phase>). Used to scope per-phase TaskCreate counts.
index_of_latest_phase_advance() {
    local t="$1"; [ -f "$t" ] || { echo ""; return; }
    jq -sr '
        [.[] | (.message // .) | (
            if .role=="user" then ((.content // []) |
                if type=="array" then ([.[]? | select(.type=="text") | .text // empty] | join("\n"))
                elif type=="string" then . else "" end)
            elif .role=="assistant" then
                ([.content // [] | .[]? | select(.type=="tool_use" and .name=="Skill") |
                  "/" + (.input.skill // "")] | join("\n"))
            else "" end)] |
        to_entries | map(select(.value | test("(^|\\n)\\s*/?pipeline[-:](plan|code|test|complete)\\b"))) |
        last // empty | if . == "" then "" else (.key | tostring) end' "$t" 2>/dev/null || true
}

# Count assistant tool_use blocks with NAME on lines strictly after INDEX.
count_tool_uses_after_index() {
    local t="$1" from="${2:--1}" tool="$3"; [ -f "$t" ] || { echo 0; return; }
    [ -n "$from" ] || from=-1
    jq -sr --argjson from "$from" --arg tool "$tool" '
        [.[] | (.message // .) | (if .role=="assistant" then
            ([.content // [] | .[]? | select(.type=="tool_use" and .name==$tool)] | length)
         else 0 end)] | to_entries | map(select((.key|tonumber) > $from) | .value) | add // 0' "$t" 2>/dev/null || echo 0
}

# Count TaskUpdate(status=completed|deleted) on lines strictly after INDEX.
count_terminal_task_updates_after_index() {
    local t="$1" from="${2:--1}"; [ -f "$t" ] || { echo 0; return; }
    [ -n "$from" ] || from=-1
    jq -sr --argjson from "$from" '
        [.[] | (.message // .) | (if .role=="assistant" then
            ([.content // [] | .[]? | select(.type=="tool_use" and .name=="TaskUpdate") |
              select(.input.status=="completed" or .input.status=="deleted")] | length)
         else 0 end)] | to_entries | map(select((.key|tonumber) > $from) | .value) | add // 0' "$t" 2>/dev/null || echo 0
}

# First active pipeline .spec.md (or legacy .md), or "". Always returns 0 so
# `VAR=$(get_active_pipeline_doc)` under `set -e` never aborts on "no doc".
get_active_pipeline_doc() {
    local d="${PROJECT_ROOT}/docs/planning/pipeline/active"
    local f
    for f in "$d"/*.spec.md; do [ -f "$f" ] && { echo "$f"; return 0; }; done
    for f in "$d"/*.md; do
        case "$(basename "$f")" in *.notes.md|.gitkeep) continue;; esac
        [ -f "$f" ] && { echo "$f"; return 0; }
    done
    return 0
}

# Pipeline session = pipeline INTENT is present. Armed by EITHER:
#   - an active /pipeline phase in the transcript (detect_active_command), OR
#   - an active pipeline doc on disk.
# (§19 2026-08-09: the third arm — a sidecar MCP tool call — retired with the
# sidecar; intent now always leaves one of these two footprints.) Per-hook phase
# scoping still keeps the gates inert outside their phase. Normal chat (no
# phase, no active doc) leaves every gate dormant.
is_pipeline_session() {
    local t="$1"; [ -f "$t" ] || return 1
    [ -n "$(detect_active_command "$t")" ] && return 0
    [ -n "$(get_active_pipeline_doc)" ]
}

# --- The Zed touchpoint ledger (CONSTITUTION §14, docs/marley/zed-touchpoints.md) ---
# The ONE definition of the Marley-owned paths and of the ledger check, shared by
# gate:16 (script/gates.sh), enforce-zed-ledger.sh (every Write or Edit) and
# enforce-commit-gate.sh (every git commit). Every other path in the tree is
# upstream Zed's: a change to it needs a row in the ledger's Touchpoints table.
# The ledger's prose list documents marley_owned_path; this file is the authority.
ZED_LEDGER="docs/marley/zed-touchpoints.md"

# The upstream fork point: the merge-base with a fetched `upstream/main`, else
# MARLEY_UPSTREAM_BASE when set (a value that names no commit fails closed), else
# the commit this fork was cut from, moved at each upstream merge (the ledger's
# merge checklist). Prints nothing when none resolves.
UPSTREAM_BASE_FALLBACK="78648aaf7d"
upstream_base() {
    git -C "$PROJECT_ROOT" merge-base upstream/main HEAD 2>/dev/null \
        || git -C "$PROJECT_ROOT" rev-parse --verify "${MARLEY_UPSTREAM_BASE:-$UPSTREAM_BASE_FALLBACK}^{commit}" 2>/dev/null \
        || true
}

marley_owned_path() {
    case "$1" in
        crates/marley_*|docs/marley/*|docs/planning/*|docs/marley_architecture/*|\
        docs/specs/*|docs/warp_architecture/*|docs/zed_architecture/*|\
        docs/decisions/*|docs/tickets/*|.claude/*|script/gates.sh|script/mutation.sh|\
        script/live-shot.sh|justfile|\
        CONSTITUTION.md|CHANGELOG.md|deny.toml|.gitleaks.toml|.semgrep.yml|\
        .cargo/audit.toml|.mcp.json.example|vendor/*)
            return 0 ;;
    esac
    return 1
}

# The paths the Touchpoints table of the ledger under ROOT (default: this
# checkout) names, one per line: the backticked path that opens a row's first
# column. Paths quoted anywhere else in the ledger are not rows.
zed_ledger_rows() {
    local ledger="${1:-$PROJECT_ROOT}/${ZED_LEDGER}" tick
    [ -f "$ledger" ] || return 0
    tick=$(printf '\140')
    awk '/^## Touchpoints[[:space:]]*$/{t=1;next} t&&/^## /{t=0} t' "$ledger" \
        | sed -nE "s/^[|][[:space:]]*${tick}([^${tick}]+)${tick}.*/\\1/p"
}

# 0 when the newline-separated LIST contains ITEM as a whole line. Pure bash: no
# pipe for an early-exiting grep to break under pipefail
# (PR-claude-no-quiet-grep-tail-in-pipefail-hooks-001).
line_in_list() {
    case $'\n'"$2"$'\n' in
        *$'\n'"$1"$'\n'*) return 0 ;;
    esac
    return 1
}

# Every path of the checkout at ROOT that differs from BASE, one per line: the
# work tree and the index against BASE (a commit ships the index, `commit -a` the
# work tree), plus untracked files that are not ignored. NUL-separated from git,
# so no name comes back quoted; a name with a newline in it is the one case this
# cannot carry. Fails when any git command fails, whatever the caller's options.
zed_changed_paths() {
    local base="$1" root="$2"
    (
        set -o pipefail
        {
            git -C "$root" diff --name-only --no-renames -z "$base" -- . \
                && git -C "$root" diff --cached --name-only --no-renames -z "$base" -- . \
                && git -C "$root" ls-files -z --others --exclude-standard
        } | tr '\0' '\n' | LC_ALL=C sort -u
    )
}

# gate:16's check of the checkout at ROOT (default: this one) against the
# upstream commit BASE. Reports every changed path outside the Marley-owned set
# that has no row, every row whose path no longer differs, rows for owned paths,
# duplicate rows, and upstream files the owned set claims (it must stay disjoint
# from upstream, or "owned" would exempt a Zed file); returns 1 if it reported
# anything.
zed_ledger_check() {
    local base="$1" root="${2:-$PROJECT_ROOT}" changed rows upstream p
    local missing="" stale="" owned_rows="" duplicates="" owned_upstream=""
    [ -n "$base" ] || {
        echo "zed-ledger: upstream fork point unknown (fetch the upstream remote or set MARLEY_UPSTREAM_BASE)"
        return 1
    }
    if ! [ -f "$root/$ZED_LEDGER" ] || ! grep -qE '^## Touchpoints[[:space:]]*$' "$root/$ZED_LEDGER"; then
        echo "zed-ledger: $root/$ZED_LEDGER has no '## Touchpoints' section"
        return 1
    fi
    changed=$(zed_changed_paths "$base" "$root") \
        || { echo "zed-ledger: git could not list the changes since $base"; return 1; }
    upstream=$(set -o pipefail; git -C "$root" ls-tree -r --name-only -z "$base" | tr '\0' '\n') \
        || { echo "zed-ledger: git could not list the files of $base"; return 1; }
    rows=$(zed_ledger_rows "$root")
    duplicates=$(printf '%s\n' "$rows" | LC_ALL=C sort | uniq -d)
    while IFS= read -r p; do
        [ -n "$p" ] || continue
        marley_owned_path "$p" && continue
        line_in_list "$p" "$rows" || missing="${missing}  ${p}"$'\n'
    done <<< "$changed"
    while IFS= read -r p; do
        [ -n "$p" ] || continue
        marley_owned_path "$p" && owned_rows="${owned_rows}  ${p}"$'\n'
        line_in_list "$p" "$changed" || stale="${stale}  ${p}"$'\n'
    done <<< "$rows"
    while IFS= read -r p; do
        [ -n "$p" ] || continue
        marley_owned_path "$p" && owned_upstream="${owned_upstream}  ${p}"$'\n'
    done <<< "$upstream"
    if [ -n "$missing$stale$owned_rows$duplicates$owned_upstream" ]; then
        [ -n "$missing" ] && { echo "zed-ledger: changed outside the Marley-owned paths with no row in ${ZED_LEDGER}:"; printf '%s' "$missing"; }
        [ -n "$stale" ] && { echo "zed-ledger: rows in ${ZED_LEDGER} whose path no longer differs from upstream (remove the row):"; printf '%s' "$stale"; }
        [ -n "$owned_rows" ] && { echo "zed-ledger: rows in ${ZED_LEDGER} for Marley-owned paths, which are never listed:"; printf '%s' "$owned_rows"; }
        [ -n "$duplicates" ] && { echo "zed-ledger: paths with more than one row in ${ZED_LEDGER}:"; printf '%s\n' "$duplicates" | sed 's/^/  /'; }
        [ -n "$owned_upstream" ] && { echo "zed-ledger: upstream files that marley_owned_path claims (narrow the owned set):"; printf '%s' "$owned_upstream"; }
        return 1
    fi
    echo "zed-ledger: every touchpoint recorded ($(printf '%s\n' "$rows" | grep -c . || true) rows)"
    return 0
}
