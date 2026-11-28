#!/bin/sh
# Structural checks for the Teacher module.
# Works from any directory: sh teacher/check.sh
# Exits 0 when every check passes, 1 otherwise. Writes nothing.
# Uses only POSIX sh, grep, sed, sort, uniq, wc.
#
# The checks enforce the design in README.md: the role holds no project state,
# the state is one small dated file, and nothing stacks. They cannot tell
# whether the state is true; re-checking its anchors is the Teacher's job.

dir=$(cd "$(dirname "$0")" && pwd)
fail=0

note() {
    printf 'FAIL: %s\n' "$1"
    fail=1
}

# 1. Required files.
required="SKILL.md README.md check.sh
core/evidence.md core/decisions.md core/missions.md
core/state-and-handoff.md core/failure-modes.md core/teaching.md
templates/mission-brief.md templates/decision-record.md
templates/report-review.md templates/owner-update.md
playbooks/README.md exam/README.md exam/QUESTIONS.md exam/ANSWERS.md
project/PROFILE.md project/STATE.md project/DECISIONS.md"
for f in $required; do
    [ -f "$dir/$f" ] || note "missing file: $f"
done

# 2. Size limits, in bytes. A file over its limit must be cut, not excused.
#    DECISIONS.md is an append-only log and has no limit.
limit() {
    [ -f "$dir/$1" ] || return 0
    n=$(wc -c < "$dir/$1" | tr -d ' ')
    [ "$n" -le "$2" ] || note "$1 is $n bytes; limit is $2"
}
limit SKILL.md 10000
limit project/STATE.md 8000
limit project/PROFILE.md 12000
for f in "$dir"/core/*.md; do
    [ -f "$f" ] && limit "core/$(basename "$f")" 9000
done
for f in "$dir"/templates/*.md; do
    [ -f "$f" ] && limit "templates/$(basename "$f")" 4000
done
for f in "$dir"/playbooks/*.md; do
    [ -f "$f" ] && limit "playbooks/$(basename "$f")" 7000
done
limit exam/README.md 4000
limit exam/QUESTIONS.md 14000
limit exam/ANSWERS.md 16000
limit project/RUNBOOK.md 16000

# 3. The role is project-independent: no commit hashes, no pull request
#    numbers, and none of the names listed in project/check-terms.txt may
#    appear in SKILL.md, core/, templates/, playbooks/ or exam/. Commands and
#    names for one system live in project/, for example project/RUNBOOK.md.
terms="$dir/project/check-terms.txt"
generic="$dir/SKILL.md"
for f in "$dir"/core/*.md "$dir"/templates/*.md "$dir"/playbooks/*.md "$dir"/exam/*.md; do
    [ -f "$f" ] && generic="$generic $f"
done
for f in $generic; do
    [ -f "$f" ] || continue
    rel=${f#"$dir"/}
    if grep -E '[0-9a-f]{40}' "$f" >/dev/null; then
        note "$rel contains a full commit hash; project facts belong in project/"
    fi
    if grep -i -E '(pull request|PR) ?#?[0-9]+' "$f" >/dev/null; then
        note "$rel names a specific pull request; project facts belong in project/"
    fi
    if [ -f "$terms" ]; then
        hit=$(grep -v -E '^(#|$)' "$terms" | grep -i -o -E -f - "$f" | sort -u | tr '\n' ' ')
        [ -z "$hit" ] || note "$rel names project-specific things ( $hit); keep the role project-independent"
    fi
done

# 4. Nothing stacks: no override or supersede headings anywhere in the module.
for f in "$dir"/*.md "$dir"/core/*.md "$dir"/templates/*.md "$dir"/playbooks/*.md "$dir"/exam/*.md "$dir"/project/*.md; do
    [ -f "$f" ] || continue
    rel=${f#"$dir"/}
    if grep -i -E '^#+ .*(override|supersed)' "$f" >/dev/null; then
        note "$rel has an override-style heading; rewrite the section it replaces"
    fi
done

# 5. The state carries exactly one dated "As of" line and its required sections.
state="$dir/project/STATE.md"
if [ -f "$state" ]; then
    n=$(grep -c -E '^As of: [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}Z' "$state")
    [ "$n" -eq 1 ] || note "project/STATE.md needs exactly one 'As of: YYYY-MM-DDTHH:MMZ' line; found $n"
    for h in '## Anchors' '## Authorizations in force' '## Open decisions for the owner' '## Next action'; do
        grep -q -F -x "$h" "$state" || note "project/STATE.md is missing section: $h"
    done
fi

# 6. Decision ids are unique, and every open decision is listed in the state.
dec="$dir/project/DECISIONS.md"
if [ -f "$dec" ]; then
    dup=$(grep -o -E '^## D-[0-9]+' "$dec" | sort | uniq -d | tr '\n' ' ')
    [ -z "$dup" ] || note "project/DECISIONS.md repeats an id: $dup"
    id=
    while IFS= read -r line; do
        case "$line" in
            '## D-'*) id=$(printf '%s\n' "$line" | grep -o -E 'D-[0-9]+') ;;
            '- Status: open'*)
                if [ -n "$id" ] && [ -f "$state" ] && ! grep -q -E "(^|[^0-9A-Za-z-])$id([^0-9]|\$)" "$state"; then
                    note "decision $id is open but not listed in project/STATE.md"
                fi ;;
        esac
    done < "$dec"
fi

# 7. Relative links in the module resolve.
for f in "$dir"/*.md "$dir"/core/*.md "$dir"/templates/*.md "$dir"/playbooks/*.md "$dir"/exam/*.md "$dir"/project/*.md "$dir"/project/missions/*.md; do
    [ -f "$f" ] || continue
    rel=${f#"$dir"/}
    base=$(dirname "$f")
    for target in $(grep -o -E '\]\([^)# ]+' "$f" | sed 's/^](//'); do
        case "$target" in
            http://*|https://*|mailto:*) continue ;;
        esac
        [ -e "$base/$target" ] || note "$rel links to missing $target"
    done
done

# 8. Nothing shaped like a secret anywhere in the module.
for f in $(find "$dir" -type f \( -name '*.md' -o -name '*.txt' -o -name '*.sh' \)); do
    rel=${f#"$dir"/}
    [ "$rel" = "check.sh" ] && continue
    if grep -E '(ghp|gho|ghs)_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|hf_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}|BEGIN [A-Z ]*PRIVATE KEY' "$f" >/dev/null; then
        note "$rel contains something shaped like a secret"
    fi
done

# 9. SKILL.md declares a name and a semantic version.
if [ -f "$dir/SKILL.md" ]; then
    grep -q -E '^name: [a-z0-9-]+$' "$dir/SKILL.md" || note "SKILL.md front matter lacks a 'name:' line"
    grep -q -E '^version: [0-9]+\.[0-9]+\.[0-9]+$' "$dir/SKILL.md" || note "SKILL.md front matter lacks 'version: X.Y.Z'"
fi

if [ "$fail" -eq 0 ]; then
    printf 'teacher module: all checks passed\n'
fi
exit "$fail"
