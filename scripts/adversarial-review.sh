#!/usr/bin/env bash
# Adversarial review loop between Claude Code and kimi-cli.
#
#   adversarial-review.sh plan <task-spec.md> --author=claude|kimi
#   adversarial-review.sh code <changes.diff> --author=claude|kimi
#
# One agent authors, the other reviews (whoever is not --author). Exactly one
# rebuttal round. All outputs land in review/ (gitignored transients):
#
#   plan mode: plan.md -> review-1.md -> rebuttal-1.md [+ plan-v2.md if revised]
#   code mode: draft.diff (copy of input) -> review-1.md -> rebuttal-1.md
#
# Non-interactive invocation (verified against installed CLIs, Jul 2026):
#   claude: prompt piped on stdin to `claude -p`, response on stdout.
#   kimi:   prompt piped on stdin to `kimi --quiet` (= --print --output-format
#           text --final-message-only), final message on stdout. NOTE: kimi's
#           --print mode auto-approves tool calls, so we pin --work-dir to
#           review/ to scope any tool use; the prompts also instruct text-only.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
REVIEW_DIR="$REPO_ROOT/review"
PROMPTS_DIR="$REPO_ROOT/prompts"
REVISED_MARKER='=== REVISED PLAN ==='

usage() {
    cat >&2 <<EOF
Usage: $(basename "$0") <plan|code> <input-path> --author=claude|kimi

  plan mode: <input-path> is a task spec; the author drafts a plan first.
  code mode: <input-path> is a diff; the diff itself is the draft.
  --author   who authors/rebuts; the other agent reviews.
EOF
    exit 1
}

# --- argument parsing --------------------------------------------------------

MODE="${1:-}"; INPUT="${2:-}"; AUTHOR=""
shift 2 2>/dev/null || usage
for arg in "$@"; do
    case "$arg" in
        --author=claude|--author=kimi) AUTHOR="${arg#--author=}" ;;
        *) echo "error: unknown argument '$arg'" >&2; usage ;;
    esac
done

[[ "$MODE" == "plan" || "$MODE" == "code" ]] || usage
[[ -n "$AUTHOR" ]] || usage
[[ -f "$INPUT" ]] || { echo "error: input file not found: $INPUT" >&2; exit 1; }

if [[ "$AUTHOR" == "claude" ]]; then REVIEWER="kimi"; else REVIEWER="claude"; fi

command -v claude >/dev/null || { echo "error: 'claude' not on PATH" >&2; exit 1; }
command -v kimi   >/dev/null || { echo "error: 'kimi' not on PATH" >&2; exit 1; }
mkdir -p "$REVIEW_DIR"

if compgen -G "$REVIEW_DIR/*.md" >/dev/null; then
    echo "note: overwriting previous run's files in review/" >&2
fi
rm -f "$REVIEW_DIR"/plan.md "$REVIEW_DIR"/plan-v*.md "$REVIEW_DIR"/review-*.md \
      "$REVIEW_DIR"/rebuttal-*.md "$REVIEW_DIR"/draft.diff

# --- agent invocation --------------------------------------------------------

# invoke <claude|kimi>: prompt on stdin, response text on stdout.
invoke() {
    case "$1" in
        claude) claude -p ;;
        kimi)   kimi --quiet --work-dir "$REVIEW_DIR" ;;
    esac
}

step() { echo "==> [$1] $2" >&2; }

# --- run ---------------------------------------------------------------------

if [[ "$MODE" == "plan" ]]; then
    PROMPT_FILE="$PROMPTS_DIR/adversarial-plan-review.md"
    DRAFT="$REVIEW_DIR/plan.md"
    DRAFT_LABEL="PLAN"

    step "1/3" "author ($AUTHOR) drafting plan from $INPUT"
    {
        cat <<'EOF'
Write a concrete implementation plan for the task below. Be specific about
steps, files, data flow, failure handling, and how the result is verified.
This plan will be adversarially reviewed, so make your assumptions explicit.
Respond with the plan in markdown only — do not modify any files.

=== TASK ===
EOF
        cat "$INPUT"
    } | invoke "$AUTHOR" > "$DRAFT"
else
    PROMPT_FILE="$PROMPTS_DIR/adversarial-code-review.md"
    DRAFT="$REVIEW_DIR/draft.diff"
    DRAFT_LABEL="DIFF"

    step "1/3" "code mode: diff is the draft, copying $INPUT -> review/draft.diff"
    cp "$INPUT" "$DRAFT"
fi

step "2/3" "reviewer ($REVIEWER) critiquing"
{
    cat "$PROMPT_FILE"
    printf '\n=== %s UNDER REVIEW ===\n' "$DRAFT_LABEL"
    cat "$DRAFT"
} | invoke "$REVIEWER" > "$REVIEW_DIR/review-1.md"

step "3/3" "author ($AUTHOR) rebutting"
{
    if [[ "$MODE" == "plan" ]]; then
        cat <<EOF
You wrote the plan below, which then received the adversarial review that
follows it. Address every numbered finding (F1, F2, ...) explicitly — for each
one, state either ACCEPT (and exactly how the plan changes) or REBUT (and why
the failure scenario does not hold). Do not skip or merge findings.

If you accept any finding, output the complete revised plan after a line
containing exactly:
$REVISED_MARKER
If you rebut everything, omit that marker and the revised plan.
Respond with text only — do not modify any files.
EOF
    else
        cat <<'EOF'
You authored the diff below, which then received the adversarial review that
follows it. Address every numbered finding (F1, F2, ...) explicitly — for each
one, state either ACCEPT (acknowledging the issue and what the fix would be) or
REBUT (and why the failure scenario does not hold). Do not skip or merge
findings. Do not produce a revised diff and do not modify any files — respond
with the rebuttal text only.
EOF
    fi
    printf '\n=== YOUR %s ===\n' "$DRAFT_LABEL"
    cat "$DRAFT"
    printf '\n=== REVIEW ===\n'
    cat "$REVIEW_DIR/review-1.md"
} | invoke "$AUTHOR" > "$REVIEW_DIR/rebuttal-1.raw"

# Split rebuttal from revised plan on the marker (plan mode only; marker absent
# means the author rebutted everything and there is no v2).
if [[ "$MODE" == "plan" ]] && grep -qF "$REVISED_MARKER" "$REVIEW_DIR/rebuttal-1.raw"; then
    awk -v m="$REVISED_MARKER" 'index($0, m) { found=1; next } !found' \
        "$REVIEW_DIR/rebuttal-1.raw" > "$REVIEW_DIR/rebuttal-1.md"
    awk -v m="$REVISED_MARKER" 'found; index($0, m) { found=1 }' \
        "$REVIEW_DIR/rebuttal-1.raw" > "$REVIEW_DIR/plan-v2.md"
else
    mv "$REVIEW_DIR/rebuttal-1.raw" "$REVIEW_DIR/rebuttal-1.md"
fi
rm -f "$REVIEW_DIR/rebuttal-1.raw"

echo >&2
echo "Done. Author: $AUTHOR, reviewer: $REVIEWER. Outputs:" >&2
ls -1 "$REVIEW_DIR" | grep -v '^\.gitkeep$' | sed 's|^|  review/|' >&2
