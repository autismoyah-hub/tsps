# Missions

A mission is a bounded piece of work handed to other agents or to the owner: one outcome, one accountable owner, a defined finish. The Teacher designs missions and judges their results. Doing the work is the job of the system being built; if the Teacher does it instead, nothing has been learned about whether the system can.

Small things are an exception. When verifying a fact or making a minor fix costs less than writing the brief, do it yourself and say so.

## Before writing a brief

Answer these first. If any answer is missing, getting it is the next action, not the mission.

1. **What useful thing will exist afterward?** A merged fix, a working capability exercised on a real task, a decision made on real data. "We will know more about our telemetry" does not qualify unless a named decision is waiting on that knowledge.
2. **What already works that this builds on?** Name the strongest known-good example. If the record shows something worked, do not commission its rediscovery.
3. **Where does each step run, and under whose authority?** Draw the path: who launches, which machine, which account, which project, who can message whom. Capabilities are directional and scoped; a path that failed before will fail again unless something changed.
4. **Is it the smallest version that produces the outcome?** If it could be half the size and still deliver, halve it.
5. **Who verifies it, and are they available when it finishes?** An independent reviewer with no slot booked is an unfinished design.
6. **What does it consume, and is that covered?** Check the standing authorizations. If it needs a new one, request it with the brief.
7. **Who will judge the result, and by what standard?** Read that standard at its source before writing a rule about quality. A brief stricter than the standard it serves produces nothing.
8. **Can the sources be prepared first?** Evidence gathered into a pack before the team starts turned two hours to a first failing test into seventeen minutes.

## The brief

Use [templates/mission-brief.md](../templates/mission-brief.md). It must be complete as written: someone who has only the brief and the repository can execute it through to the verified result. Never plan to send the essential part later.

Write for the reader. Explain why the mission exists and what a good result looks like; a capable agent given the purpose makes better choices than one given only prohibitions. Keep prohibitions to those that guard a real risk in this mission.

## Authorization

An authorization states who granted it, for what scope, with what limits on spend, time and side effects, and when it ends. Record it in `project/DECISIONS.md` when it is given.

- It should be sized to the work. One envelope for a mission's routine actions is better than a permission for each; ask for a per-action approval only where a single action carries the risk.
- Different kinds of cost are different decisions. Cheap metered compute for testing, model usage on an included subscription, and paid model API calls each need their own answer. Do not let caution about one block another.

## Acceptance

A mission is accepted when the useful outcome occurred and someone independent of the author confirmed it on the exact artifact. The reviewer confirms the artifact; you judge whether the mission met its acceptance condition; merging or publishing the result is the owner's call. If the project has its own closing procedure for missions, PROFILE.md names it and it applies as well. None of these stands in for another.

Decide the acceptance condition before the mission runs and write it in the brief. Deciding afterward lets any result be called a success.

## Reviewing what comes back

1. Read the primary record: the diff, the report at its exact revision, the logs. Do not review a summary of them.
2. Establish what stage was reached, using the vocabulary in [evidence.md](evidence.md).
3. Check the outcome against the acceptance condition written in the brief.
4. Check the review: independent, on the final artifact, examining the result.
5. Note what the report says it did not establish, and what it should have said and did not.
6. Give a verdict and the next action. Use [templates/report-review.md](../templates/report-review.md).

| Verdict | Meaning |
| --- | --- |
| Accepted | The outcome occurred and was independently confirmed |
| Accepted in part | State exactly which part; the rest is open work, not a footnote |
| Not accepted | The outcome did not occur; say what did |
| Not established | It may have occurred; the evidence does not show it |

A mission that correctly declined to proceed has produced a hold, which is information about the design. It is not an accepted outcome and should not be counted as progress.

## When a mission holds or fails

A hold is acceptable only when it states:

- the specific fact or decision that would release it;
- who can supply that, and who clears the hold;
- what work continues meanwhile that does not depend on it.

A hold is cleared by whoever owns the record it is written in: a team for its own log, the owner for the owner's decision. When the hold is not yours, establish the fact that lifts it, say what the evidence does and does not show, and hand it to that party. Do not edit their record. Clearing a hold restores what was authorized before it and authorizes nothing new.

Then ask why the design allowed the hold. If it was predictable from what was known beforehand, such as a missing input, a reviewer slot already past or a gate that no real case could pass, the brief was wrong and the fix is in the design. Say so, correct the brief, and send the same team back at once, while its sessions and what it learned are still open, without waiting on other preparation. For every rule you relax, give the reviewer a matching check, so that a looser brief does not mean a looser result.

**Two-strike rule.** A thread is a run of missions aimed at the same outcome, whatever they are called. If two consecutive missions on a thread end without an accepted useful result, do not write a third on the same pattern. Stop, find what the two have in common, and change the approach: a smaller target, a different route, a supervised manual step in place of an automated gate, or dropping the thread. Record in the state which threads the rule has stopped.

**When an agent declines.** Give it once the context it lacked, plainly, and say that no is an acceptable answer. If it declines again, close the task; do not reword it to slip past the objection or send it a third time. Then ask what part of the task avoids the objection.

**When a message cannot arrive.** Platforms limit who may message whom and how far a chain may run. Design so that nothing depends on a message arriving: results go to a place you read (a pushed branch, a report, the session's own record), and you address each worker directly instead of through a relay.

**Ambiguous outside effects.** When a request to an outside service may or may not have taken effect, do not retry. Find out from the authoritative side what happened, record it, and only then decide.

## Patterns to refuse

When a proposed mission has one of these shapes, redesign it before it is sent. The causes and cases are in [failure-modes.md](failure-modes.md).

- Measurement with no decision waiting on the reading.
- A rehearsal on fabricated data when a real task is available.
- A repeat test of a capability that has not regressed.
- One more precondition added because the last mission stopped on a different one.
- Setup now, with the real task to follow.
- Every routine step waiting on one coordinator or one approval.
- A new tool, store or scheduler where an existing one would do with a small change.

## Improving by measurement

Keep a record of each mission that a tool can read: when it was sent, the first failing test, acceptance, publication, the outside verdict, what it consumed, what was refused or redone. After each mission find the largest delay or waste in the record, change one thing to remove it, and compare the next record. Change one thing at a time, or the comparison says nothing.

## Comparing options by experiment

When a choice between providers, models or designs matters enough to test:

- Use real tasks from the backlog, chosen before any run.
- Change one thing. Write down what is held equal, settings included, and apply the same baseline, limits and acceptance rubric to each side.
- The judge does not know which side produced which result and is not one of the things compared. That rules you out when your own model is a side.
- Before the run, write down each possible outcome and what it would decide, including that neither side succeeds.
- Record every attempt, including the blocked and abandoned ones. A rate without its denominator means little.
- One pair is a feasibility result. Swap the assignments on a later pair before concluding which is better.
