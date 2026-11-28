# Building a small tested tool by pull request

## When to use it

Something the Teacher has been doing by hand needs to become a tool in the repository: reading a number, building a record, comparing two results. Small means one module of a few hundred lines with its tests. If it needs a framework, a database or a service, it is not this playbook and probably not yet justified.

## The brief

```
Build a small, tested tool that <does what>, in a repository that already has conventions for this
kind of thing. Work in your own worktree, open a pull request, and stop; do not merge.

Why this matters: <the decision or measurement waiting on it, and what went wrong when it was done by
hand>.

Read first, from the default branch: <the conventions file; the plan or state that says what is
measured; the reports or inputs the tool will read; an existing module for house style>. Do not import
from or modify <existing modules>; this is new and separate.

Build:
1. <The format or interface, documented in one file. Every value carries where it came from. A value
   that is not known is null with a reason, never zero and never guessed. Version the format.>
2. <The tool, runnable as one command. So that tests are deterministic and need no network, it accepts
   its inputs as files, with a thin optional wrapper that fetches them when a network is available.>
3. <A second command if needed, e.g. a comparison that never computes a difference when either side is
   unknown.>
4. Tests with small frozen fixtures: <the derivations; unknown in, unknown out; rejection of malformed
   input; the awkward real case>.
5. Real outputs for <the cases that already exist>, built with the tool, with the inputs committed so
   they can be regenerated. Values only a report holds are taken exactly as the report states them and
   tagged as such; where a report gives no value, leave it null.
6. A short section: what these real outputs show, stating only what the numbers support; and what is
   still taken by hand, with what would be needed to collect each automatically.

Constraints: <language and dependency rules>; no network in tests; keep it small; do not touch <areas>.
No secrets, tokens or addresses in any file.

Version control: <the worktree and branch commands; the push form; how to open the pull request; that
the repository runs tests on pull requests>. Wait for the checks to finish, by polling until every job
reports a result, and report them. If you lack the access to open the pull request, say so and give me
the branch name.

Before finishing, run the whole test suite locally and report counts.

Reply with: the pull request or branch; the interface in brief; test counts; a real output, pasted;
anything in the inputs that was contradictory or too vague to record; and what you would build next.
```

## What a good result looked like

A record format in which every value carries a source tag, a collector, a comparer, 29 tests including one that rebuilds each real record from its committed inputs and requires identical bytes, and four real records. Its list of "contradictory or too vague" items showed that three figures the Teacher had been quoting were not in the reports.

## What to guard against

- **Guessing to fill a field.** State the unknown rule in the brief's first lines. The value of the tool is that its nulls are honest.
- **Access.** The agent may be able to push but not to open a pull request. Ask it to report that and stop; open it yourself.
- **Merging on a wait that returned early.** Poll for a result on every job before merging. See the project's runbook.
- **A tool that confirms the hand count.** Ask for the discrepancies explicitly. If the tool agrees with you everywhere, check that it is reading the sources and not your summary.
- **Tests that need the machine it was built on.** Fixtures as files, no network, no local paths; the pull request's own test run on other systems proves it.
