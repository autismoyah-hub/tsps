# Cold-start test of a role or instruction set

## When to use it

You have written something another agent is supposed to be able to pick up with no other context: a role, a kit, a handoff. Before saying it works, give it to an agent that has nothing else and see what it does. You cannot judge this yourself; you know too much.

Run it again after any substantial change, and when a different model is to take the role. For a Teacher candidate, follow it with the examination in [exam/](../exam/README.md).

## The brief

```
This is a cold-start usability test of a role definition. You have no other context, and that is the
point. Do not modify any files, do not run commands that change anything, and do not contact any
network host. Read only.

The repository checkout is at: <path>

Step 1. Follow this instruction exactly as a new agent would: "<the one-line instruction that is the
whole handoff>". Read whatever that file directs you to, as you judge necessary.

Step 2. Using only what the module and the documents it links to gave you, answer these as the role
would. A few sentences each.
  a. What is the goal, and how is success measured?
  b. What is the current state, and which parts would you personally vouch for versus treat as
     someone else's report?
  c. What exactly would you do in the first ten minutes of a real session, and what would you tell
     the owner?
  d-g. <Four situations the role must handle, taken from real cases: a tempting bad idea from the
     owner; a second tempting bad idea of a different kind; a report claiming completion; a repeated
     failure on one thread.>
  h. <A mechanical task: exactly which files are edited and what steps follow.>

Step 3. Critique the module honestly. I wrote it and want defects found, not praise. Report:
  - Anything you needed that was missing, or that you had to guess.
  - Contradictions between files, or between the module and the documents it links to.
  - Instructions that are ambiguous, or that you could follow in two materially different ways.
  - Anything over-prescriptive, redundant, or likely to be ignored in practice.
  - Places where the general part is not actually independent of this project.
  - Broken or misleading links or references.
  - Whether the size is right: what you would cut, and what you would add.
Rank the defects by how much they would hurt a real session, most harmful first, and cite file and
section for each.

Return steps 2 and 3. No preamble.
```

## What a good result looked like

Correct answers to all eight situations from the module alone, and eighteen ranked defects. The three most harmful were real and were not visible to the author: the state file described a commit that had not been made; nothing said where state is saved between sessions; and three files gave three different answers about what spending was authorized.

## What to guard against

- **Giving it your conclusion.** Do not tell the tester what you think is weak. An agent handed a hypothesis returns it confirmed.
- **Fixing only the easy defects.** Rank by harm and fix from the top. Record which were left and why.
- **Not re-running.** The fixes change the module; the first run says nothing about the fixed version. The first session here did not re-run and said so.
- **Testing reading, not doing.** The situations in step 2 are what show whether the role transfers. Take them from real decision points, including ones the author got wrong.
