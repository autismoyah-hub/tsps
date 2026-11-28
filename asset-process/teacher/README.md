# Teacher module

A self-contained role definition for the **Teacher**: the agent that directs and mentors the owner in building this project's agentic system. Any capable agent can take up the role by reading this directory. It does not depend on any earlier conversation.

## Start a Teacher session

In any agent harness, with this repository checked out, say:

> Read `teacher/SKILL.md` in the repository and act as the Teacher.

That is the whole handoff. The agent reads [SKILL.md](SKILL.md), then the project binding in [project/](project/), checks the state against the live system, and reports where things stand.

**In this package:** `project/` is not included. [project-docs/](project-docs/) is this asset project's own record (the Producer's role, runbook, handoff and domain register, profile, capabilities, areas and plans), sanitized and kept as a worked example; the prompts point at it. A new project starts from [project-template/](project-template/): copy it to `project/` and fill it in.

Harness conveniences:

- **Claude Code:** the stub at `.claude/skills/teacher/SKILL.md` makes the role available as the `teacher` skill in sessions opened in this repository.
- **CuttleCode:** to list the role among the project's skills, register the same stub once the module is on the default branch. Check `cuttle skills write --help` first, then run `cuttle skills write teacher.md --file .claude/skills/teacher/SKILL.md` from the repository root.
- **Any other agent:** the one-line instruction above is enough.

## Layout

| Path | Purpose | Changes |
| --- | --- | --- |
| [SKILL.md](SKILL.md) | The role: responsibilities, authority, session start, principles, limits | Rarely |
| [core/evidence.md](core/evidence.md) | How claims are graded and described | Rarely |
| [core/decisions.md](core/decisions.md) | Challenging ideas, holding positions, asking the owner, recording | Rarely |
| [core/missions.md](core/missions.md) | Designing, authorizing and reviewing delegated work | Rarely |
| [core/state-and-handoff.md](core/state-and-handoff.md) | Where truth lives and how it is kept current | Rarely |
| [core/failure-modes.md](core/failure-modes.md) | How this role has failed before, and the counters | When a new pattern appears |
| [core/teaching.md](core/teaching.md) | How the owner is taught; the curriculum | Rarely |
| [templates/](templates/) | Mission brief, decision record, report review, owner update | Rarely |
| [playbooks/](playbooks/README.md) | Proven briefs for recurring agent jobs, as fill-in templates, with what went wrong | When a job is done a new way |
| [exam/](exam/README.md) | An examination for a candidate Teacher, built from real decision points, with a marking guide | When the role meets a new kind of decision |
| [teacher/project-docs/PROFILE.md](teacher/project-docs/PROFILE.md) | This project: goal, system map, standing rules, sources of truth | When something stable changes |
| [project/STATE.md](project/STATE.md) | What is true now, open decisions, next action | Whenever something material changes; rewritten in place |
| [project/DECISIONS.md](project/DECISIONS.md) | Log of decisions and authorizations | Appended |
| [teacher/project-docs/RUNBOOK.md](teacher/project-docs/RUNBOOK.md) | The exact commands and pitfalls for this system | When a procedure changes |
| `project/check-terms.txt` | Project names that must not appear in the role | With the project |
| [check.sh](check.sh) | Structural checks | With the rules it enforces |

`SKILL.md`, `core/`, `templates/`, `playbooks/` and `exam/` are independent of any project, and the check script fails if this project's names appear in them. `project/` is the only part that describes this one; the runbook lives there because commands and machine names are project facts.

## Design

Three choices shape the module. Each answers a way the previous version of this role, which ran for 64 versions in a chat product, went wrong.

1. **The role is separate from the state.** The earlier skill kept commit hashes, pull request numbers and quota readings in the role file. It grew to 52 KB and opened with 27 blocks each marked current. Here the role holds no project facts, and the check script fails if any appear.
2. **There is one state file, rewritten in place.** The earlier skill required about nine artifacts to be updated after each event, and they fell behind. Here a material change means editing one file, sometimes adding a few lines to the decision log, and committing.
3. **Principles are consolidated; rules are not accumulated.** The earlier skill added a prohibition after each mistake, and the same mistake returned under another name at least six times. Here the corrections on record are reduced to seven patterns with their causes, and a new lesson is folded into an existing pattern where it fits.

The repository is the only memory. Version control is the history.

## Maintaining

- Run `sh teacher/check.sh` before committing a change under `teacher/`. It works from any directory and writes nothing. It checks structure; it cannot tell whether the state is true.
- The Teacher edits `project/` as part of its work; `teacher/project-docs/PROFILE.md` says which branch that is committed to. Changes to the role go through the repository's review path like any other change.
- Bump `version` in `SKILL.md` when the role's behavior changes.
- Keep the size limits. If a file needs to exceed its limit, that is a signal to cut or to move detail into a project document, not to raise the limit.

## Using this on another project

Copy `teacher/` and the stub, then rewrite `project/`: the profile, the state, an empty decision log, and `check-terms.txt`, the list of project names the check keeps out of the role. `SKILL.md`, `core/` and `templates/` should need no changes. If they do, the part that needed changing was project-specific and belongs in `project/`; fix that here as well.

This repository remains the source of the role. A copy made for another project, or for a second Teacher working for the same owner, names this repository in its profile and is not edited in place: its Teacher proposes changes to the role here, through the review path, as [core/state-and-handoff.md](core/state-and-handoff.md) describes. The two Teachers each direct their own project and answer to the owner; neither directs the other.

## Lineage

Version 3.0.0, written 2026-11-22, replaces `cuttlecode-agent-systems-teacher` version 2.10.5. Kept from it: the evidence vocabulary, the mission and authorization discipline, the review method, the six-field owner update, the curriculum, and the record of corrections. Dropped: the stacked override blocks, the per-milestone reference files, the dated state archives, the separate changelog, and keeping the canonical copy outside the repository. [teacher/project-docs/PROFILE.md](teacher/project-docs/PROFILE.md) records where the earlier archive is held.
