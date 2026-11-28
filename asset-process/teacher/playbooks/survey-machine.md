# Read-only survey of a machine

## When to use it

A machine holds earlier work, and you need to know what is there and what is worth taking before anyone copies or deletes anything. A survey is not an extraction: its product is a ranked inventory.

## The brief

```
Read-only inventory of a remote machine. This is a survey, not an extraction: I need to know what is
there and what is worth pulling, so that a later step can copy the valuable material deliberately.

Access: <the exact command form, the remote shell, what is installed there, quoting advice>.

Context: <what the machine was, what was done on it, what terms appear in notes about it, why the work
moved away>. The owner wants everything valuable found so discoveries are not re-made.

Hard limits. These matter because that machine holds live credentials and other people's work:
- Change nothing there. No writes, deletes, moves, installs, no starting or stopping processes or
  services, no version-control operations that write, no tool commands other than ones that only list
  or read. If unsure whether a command mutates, do not run it.
- Do not open or print the contents of anything that holds secrets: <the known locations and file
  patterns>. Listing names and sizes is fine. If you meet a secret value by accident, do not repeat it.
- Do not copy anything off the machine in this step, except small text you read in order to describe it.
- Do not read private conversation records wholesale. List them, count them, and read the first lines
  of a few to classify what they are.
- Stay out of personal areas unrelated to the project unless a directory name clearly marks it as
  project material; in that case list it, do not read it.

What to survey: <the tool's own data directory, top level by top level>; <what its read-only commands
report: projects, skills, agents, sessions>; <every repository under the user's profile: path, remote,
branch, last commit date, uncommitted and unpushed work; unpushed work is the highest-value finding>;
<anything matching the project's terms anywhere>; <other drives, top level>.

Write your findings to one file: <path>. Structure: a dated orientation paragraph saying this is a
read-only survey and what was not examined; one section per item with tables of paths, sizes and dates;
"What is worth extracting", ranked, each with path, size, why it matters and how sensitive it is;
"What I could not determine". Wherever you record a status for earlier work (passed, approved, failed),
record who gave it. Acceptance of this work is <whose verdict counts>; a label given by anyone else is
that party's opinion and is reported as such. Under <size>. Exact paths in backticks. No secret values, no email
addresses.

Reply with: the file path; a ranked list of the ten most valuable things found; and anything
surprising or worrying about the machine's state.
```

## What a good result looked like

A ranked list whose first item was a commit that existed nowhere else, and whose "worrying" section reported that the disk was filling, that the tool was still running with sessions waiting, and that an automation had been starting a session every minute until that morning. Items the earlier record had called lost were found within the hour.

## What to guard against

- **Slow scans.** A recursive size scan of a large drive times out and leaves a process behind. Ask for layered listing: top level, then two levels with counts, then sizes only for what looks relevant.
- **Network access with the machine's own credentials.** A read-only version-control command that contacts a remote uses stored credentials. Say that commands which contact a network are out unless named.
- **Taking agents' labels as fact.** A survey recorded a study as passed because agents had passed it; the owner had called it a failure. The brief now names whose verdicts count and asks for every label with its judge.
- **The summary is not the inventory.** Read the "worth extracting" table yourself before commissioning the copy.
- **The machine changes under you.** A drive was unplugged and attached elsewhere while an agent was reading it. Tell a running agent at once when access moves.
