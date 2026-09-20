#!/bin/sh
# Reports whether the base branch has moved ahead of the branch being worked on.
#
# Several sessions are often writing on this repository at the same time, so main moves while
# a branch is being written. This script is the check for that and nothing else: it fetches,
# compares, prints what it found, and exits. It never merges and never touches the working
# tree. The merge itself is /sync-main.
#
# Usage:
#   sh .claude/hooks/check-main.sh              print a report and exit 0
#   sh .claude/hooks/check-main.sh SessionStart print the report as hook context
#   sh .claude/hooks/check-main.sh PreToolUse   the same, at most once every 15 minutes,
#                                               and refuse a push while main is unmerged
#
# Environment:
#   CLAUDE_MAIN_BRANCH           base branch to compare against (default: main)
#   CLAUDE_MAIN_CHECK_INTERVAL   seconds between PreToolUse checks (default: 900)

set -u

EVENT="${1:-}"
BASE_BRANCH="${CLAUDE_MAIN_BRANCH:-main}"
INTERVAL="${CLAUDE_MAIN_CHECK_INTERVAL:-900}"

# Anything unexpected leaves the session alone rather than interrupting it.
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || exit 0
GIT_DIR=$(git rev-parse --git-dir 2>/dev/null) || exit 0
STATE="$GIT_DIR/claude-main-check"

BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null) || exit 0
[ "$BRANCH" = "$BASE_BRANCH" ] && exit 0
[ "$BRANCH" = "HEAD" ] && exit 0

# The hook is handed its input as JSON on standard input. jq is not on every machine
# this runs on, so fall back to reading the one field that is needed.
extract_command() {
    if command -v jq >/dev/null 2>&1; then
        OUT=$(printf '%s' "$1" | jq -r '.tool_input.command // empty' 2>/dev/null)
        if [ -n "$OUT" ]; then
            printf '%s' "$OUT"
            return
        fi
    fi
    printf '%s' "$1" | sed -n 's/.*"command"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p'
}

# Only a command that runs git push counts. A command that merely mentions the words -
# a search for them, a message about them - is not a push, and refusing it stops work
# that was never the risk. Each command in a chain is looked at on its own.
is_push() {
    printf '%s' "$1" \
        | awk '{ gsub(/[;|&]/, "\n"); print }' \
        | grep -Eq '^[[:space:]]*git[[:space:]]+push([^a-zA-Z0-9_-]|$)'
}

# A push is the point after which another session sees the branch, so it is checked every
# time rather than on the interval.
PUSHING=no
if [ "$EVENT" = "PreToolUse" ]; then
    INPUT=$(cat)
    COMMAND=$(extract_command "$INPUT")
    if is_push "$COMMAND"; then
        PUSHING=yes
    fi
    # Deleting a branch, pushing a tag and a dry run all carry none of this branch's
    # work to the remote, so none of them is the push this check is for.
    if [ "$PUSHING" = "yes" ]; then
        case "$COMMAND" in
            *--delete* | *--tags* | *--dry-run* | *" -d "* | *" :"*) PUSHING=no ;;
        esac
    fi
fi

NOW=$(date +%s 2>/dev/null) || exit 0
if [ "$EVENT" = "PreToolUse" ] && [ "$PUSHING" = "no" ] && [ -f "$STATE" ]; then
    LAST=$(cat "$STATE" 2>/dev/null)
    case "$LAST" in
        '' | *[!0-9]*) LAST=0 ;;
    esac
    [ $((NOW - LAST)) -lt "$INTERVAL" ] && exit 0
fi
[ -n "$EVENT" ] && printf '%s\n' "$NOW" >"$STATE" 2>/dev/null

# A machine with no network is not a reason to stop work.
git fetch --quiet origin "$BASE_BRANCH" 2>/dev/null || exit 0
BEHIND=$(git rev-list --count HEAD..FETCH_HEAD 2>/dev/null) || exit 0
[ "$BEHIND" = "0" ] && exit 0

MERGE_BASE=$(git merge-base HEAD FETCH_HEAD 2>/dev/null) || exit 0

TMP=$(mktemp -d 2>/dev/null) || exit 0
trap 'rm -rf "$TMP"' EXIT INT TERM

git diff --name-only "$MERGE_BASE" FETCH_HEAD 2>/dev/null | sort -u >"$TMP/theirs"
{
    git diff --name-only "$MERGE_BASE" HEAD 2>/dev/null
    git diff --name-only HEAD 2>/dev/null
    git ls-files --others --exclude-standard 2>/dev/null
} | sort -u >"$TMP/mine"
comm -12 "$TMP/theirs" "$TMP/mine" >"$TMP/overlap"

COMMITS=$(git log --oneline --no-decorate HEAD..FETCH_HEAD 2>/dev/null | sed 's/^/  /')

# printf rather than echo throughout: a commit subject can contain a backslash, and echo
# expands it in some shells.
{
    if [ "$PUSHING" = "yes" ]; then
        printf 'This push is refused: %s has moved %s commit(s) ahead of %s and has not been merged in.\n' \
            "$BASE_BRANCH" "$BEHIND" "$BRANCH"
    else
        printf '%s has moved: %s commit(s) on origin/%s are not on %s.\n' \
            "$BASE_BRANCH" "$BEHIND" "$BASE_BRANCH" "$BRANCH"
    fi
    printf '\n%s\n\n' "$COMMITS"
    if [ -s "$TMP/overlap" ]; then
        printf 'Files they changed that this branch also touches:\n'
        sed 's/^/  /' "$TMP/overlap"
    else
        printf 'None of them touch a file this branch has changed.\n'
    fi
    printf '\nMerge it in with /sync-main before going further.\n'
} >"$TMP/message"

json_escape() {
    sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' "$1" | awk '{ printf "%s\\n", $0 }'
}

case "$EVENT" in
    '')
        cat "$TMP/message"
        ;;
    PreToolUse)
        if [ "$PUSHING" = "yes" ]; then
            cat "$TMP/message" >&2
            exit 2
        fi
        printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"%s"}}\n' "$(json_escape "$TMP/message")"
        ;;
    *)
        printf '{"hookSpecificOutput":{"hookEventName":"%s","additionalContext":"%s"}}\n' "$EVENT" "$(json_escape "$TMP/message")"
        ;;
esac

exit 0
