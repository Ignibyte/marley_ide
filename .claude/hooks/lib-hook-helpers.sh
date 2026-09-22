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
        | sed 's|^\.claude/worktrees/[A-Za-z0-9._-]\+/||'
}

# --- Gate receipt fingerprint ------------------------------------------------
# Content fingerprint of all gated + gate-DEFINING files: HEAD + the worktree
# content of every tracked-or-untracked `crates/**/*.rs` (CONSTITUTION §3
# application code, Zed's crates included) AND the files that define the bar
# itself — `script/gates.sh`, `.claude/hooks/*.sh`, `clippy.toml`, `deny.toml`,
# `.gitleaks.toml`, `.cargo/audit.toml`, `.cargo/config.toml`,
# `.cargo/mutants.toml`, the Cargo manifests + lockfile, the toolchain pin, the
# nextest config — so a post-green *weakening of the gate*
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
        git -C "$root" ls-files -z -- crates script/gates.sh .claude/hooks clippy.toml deny.toml .gitleaks.toml .cargo/audit.toml .cargo/config.toml .cargo/mutants.toml Cargo.toml Cargo.lock rust-toolchain.toml .config/nextest.toml 2>/dev/null
        git -C "$root" ls-files -z --others --exclude-standard -- crates script/gates.sh .claude/hooks clippy.toml deny.toml .gitleaks.toml .cargo/audit.toml .cargo/config.toml .cargo/mutants.toml Cargo.toml Cargo.lock rust-toolchain.toml .config/nextest.toml 2>/dev/null
    } | LC_ALL=C sort -z -u | grep -zE '\.(rs|sh|toml|lock)$' | tr '\0' '\n')
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
# Phases: plan design implement inspect validate complete. Plus work/commit.
_PIPE_RE='(pipeline[-:](plan|design|implement|inspect|validate|complete)|commit|work)'

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

# Bare PIPELINE PHASE name (plan|design|implement|inspect|validate|complete),
# or "" when the latest slash command is NOT a pipeline phase (e.g. /commit).
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
    [ -n "$m" ] && echo "$m" | grep -oE 'pipeline[-:](plan|design|implement|inspect|validate|complete)' | tail -1 | sed 's/^pipeline[-:]//' || true
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
        to_entries | map(select(.value | test("(^|\\n)\\s*/?pipeline[-:](plan|design|implement|inspect|validate|complete)\\b"))) |
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
