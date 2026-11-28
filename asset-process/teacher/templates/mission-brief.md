# Mission brief template

Copy the block below. Fill every section; write "none" where a section does not apply so the reader knows it was considered. The section on a lost session always applies. The finished brief should be executable by someone who has only this text and the repository. Guidance is in [core/missions.md](../core/missions.md).

```markdown
# Mission <id>: <outcome in a few words>

Status: proposed | authorized by <who> on <date> | dispatched <date>
Launched by: <who sends this, to which session or role>

## Why this mission exists
<Two or three sentences. The outcome wanted and what it enables. What a good
result looks like, in terms the executor can use to make judgment calls.>

## What is already known
<The baseline to build on: the strongest known-good example, with references to
exact revisions or documents. What has been tried and need not be repeated.>

## Scope
<The one deliverable. What is deliberately left out, and why.>

## Roles and placement
<Who owns the mission. Who implements, who reviews (independent of the author).
Where each runs: machine, project, account or runner. How they reach each other.>

## Permitted
<Actions, resources and spend covered by the authorization, with limits and the
time at which the authorization ends.>

## Not permitted
<Only the prohibitions that guard a real risk in this mission, each with its reason.>

## Steps
<The first capability check. The real task. What to do if a step is unsupported
or fails: the fallback, or stop and report.>

## If a session is lost
<Never "none". How often work is committed and pushed, so that a replacement can
continue from it. How a lost session is replaced: a fresh session from the
pushed work unless the project's runbook says a resume is safe. How many
replacements end the mission.>

## Deliverable
<The exact artifact: a branch and report path, a merged change, a recorded result.>

## Acceptance
<The observable condition that means the outcome occurred, decided now. Who
verifies it, on which exact artifact, and when they are available.>

## Report and stop
<Where the report goes and what it must contain: stage reached, evidence with
grades, what was not established, what was consumed. The point at which the
mission ends. What it does not authorize next.>
```

## Check before sending

- The outcome is a useful result, measurable, and more than setup.
- It builds on a named baseline and uses the existing team and tools.
- Every location, account and message route in it has been shown to work, or checking it is the first step.
- The real task is in this brief, with a fallback for the step most likely to fail.
- It says what happens when a session is lost: work pushed as it is made, how the session is replaced, and the number of replacements that ends the mission. A brief without this is incomplete.
- A reviewer independent of the author examines the final artifact, and has time booked to do so.
- Spend, access and side effects are covered by a recorded authorization that has not expired.
- It states where it stops and that nothing further is authorized.
- It is the smallest mission that produces the outcome.
