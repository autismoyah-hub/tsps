# Saving unique work before storage is cleared

## When to use it

Storage is to be cleaned up and the deletion cannot be undone. Somewhere in it is work that exists nowhere else, mixed with bulk that can be regenerated. Three steps, three agents or runs: survey and classify; save and verify; then delete, which the Teacher does itself.

## Step 1: survey and classify

```
Read-only survey of <server>, to prepare a safe cleanup. Nothing may be changed there in this step.
The cleanup itself will be done later by someone else, from your report.

<Access. What the server is, how its storage is organised, which part is live and must be left
entirely alone, which part is old.>

The owner's rule: anything may be deleted except tooling or work that should be recovered and stored
safely first. So the question, volume by volume: what is here that exists nowhere else, and what is
bulk that can simply go? Already recovered from elsewhere, not to be counted as at risk unless the copy
here differs: <list>.

Survey: the storage units and which belong to something running now (mark those live); actual size and
newest modification per unit, in one pass, not one command per unit; the composition of the bulk;
every repository: remote, branch, uncommitted changes, commits on no remote, stashes, with branch names
and subjects so the value can be judged without another visit, and whether those commits are already on
<the known remotes> by hash; non-repository material that looks like tooling or evidence; what is large
outside the storage area; what is using the machine now. Note anything that looks like credentials by
name only.

Write the report to <path>, with the full per-unit table saved separately if long. Structure:
orientation; the system; composition; a table of units grouped as (a) live, (b) holds unique work:
save first, (c) possibly useful tooling or evidence: review before deleting, (d) bulk only; the unique
work in detail; "Proposed save set" with exact paths, total size and a method that only reads from the
server; "Proposed deletions" with the space each frees; "What I could not determine".
```

## Step 2: save and verify

```
Copy unique work off <server> so that its old storage can later be deleted. The deletion will be
irreversible, so the standard is: after you finish, every item on the save list exists on this machine
and has been checked to be complete and readable. You delete nothing, anywhere. On the server you only
read.

Read first, in full: <the survey report and table>. Follow its save set; where it is ambiguous, save
more rather than less.

<Access. Live work is running there: one heavy transfer at a time, lowest priority, never touch live
units.>

Part 1, must-save, completely before part 2: for each repository with unique commits, the whole
repository metadata directory as an archive (not a bundle of branches: some commits are reachable only
from detached positions and reference logs); for each working copy with uncommitted work, the
differences as patches, the status listing, the untracked files as an archive, and the commit it was on.
Verify before moving on: every archive lists cleanly; each repository copy passes an integrity check
and every unique commit hash resolves in it; every patch is non-empty where changes were reported;
hashes of every saved file are recorded.

Part 2, review material: <patterns>, with exclusions so bulk does not come along, and never any
credential file; list those under "left behind on purpose". Stop and report if this grows much beyond
<size>.

Write a manifest: every saved item with its source path, local path, bytes, and which line of the
survey's list it covers, so that list can be ticked off. End with "Not saved", each with the reason.

Finally, one independent completeness check against the server, read-only: re-list what is live, and
re-run the survey's search for unique work once more, including places the survey did not look
(reference logs, unreachable commits). Save anything it finds and say so.
```

## Step 3: delete

The Teacher does this itself, not an agent. Tell the owner first what is about to go, in which order, and what each step frees. Two tiers: the bulk-only group holds nothing unique and may go at once, after the notice and a fresh listing of what is live; every other group waits for step 2 and its completeness check. Do not hold the first tier for the second. An item left out of a save by a mechanical rule such as size has not been judged worthless. Re-list what is live immediately before each step; the live set changed during the survey. Never touch the current project's storage. Read back the free space afterward.

## What a good result looked like

Every one of 48 unique commits confirmed inside a copy that passed its integrity check; patches matching the server's own figures; and a final check that found 49 further commits reachable only through reference logs, plus a dropped stash, that the survey had missed.

## What to guard against

- **Saving by size limit.** The first pass left out captures and large archives because they exceeded a size threshold. Decide what is irreplaceable by kind, then deduplicate by content hash: 53 GB reduced to 16.
- **Agents paused on a permission prompt.** A long save can stop waiting for approval and nobody notices. Check on it.
- **Claims that still have a reference.** A storage claim referenced by a finished job will not delete cleanly. Leave it and note it.
- **The copy becoming the only copy.** After deletion the saved material exists in one place. Say so to the owner and ask where a second copy should go.
