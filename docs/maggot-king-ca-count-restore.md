# Maggot King kill-count restoration

The Maggot King plugin retains a lifetime kill count in `maggot-king:kills`.
The shared Combat Achievements system separately retains its count by monster
name. A character with an older local count ahead of the shared count needs the
existing login restore query to report that local count.

`Recovery.MaggotKing.js` answers `combat-achievements:kill-count` for `Maggot King`.
It reads a positive safe integer from a number or numeric string. Other values,
including booleans, arrays, objects, fractions and non-finite numbers, contribute
no answer. The query does not repair the saved value, increment a kill, send a
message or award a task directly.

The existing shared restore path takes the greater count and silently checks
Maggot Squasher (one kill) and Maggot Exterminator (25 kills). Existing task bits
prevent repeated points or completion events. No `exact` override is supplied:
a shared count already ahead stays ahead, and the local count is not written
back. A pre-existing difference between that higher shared count and the local
kill-count announcement remains separate.

The ordinary native death handlers and their messages remain unchanged. After
local-ahead catch-up, one owned death should add one to each existing counter.
This does not implement the conditional Maggot achievements, change Camping
the King, or complete the deferred corpse/reward chain.

## Verification status

The existing native Maggot fixture now contains proposed checks for malformed
query coercions, registered `PlayerSave` attribute serialization/loading and
login restoration, 1/25 thresholds, max merge and repeated-restore idempotence.
It also includes a supplied lethal `PendingHit` through the real owned
`NPCDeathTask` path after a local-24/shared-0 catch-up.

These cases are **unexecuted pending normal source/input/native review**. The
save probe round-trips the registered attribute JSON and applies the real
`PlayerSave` snapshot before dispatching registered login hooks. It does not
claim a disk or socket relog. The lethal hit and survival setting are supplied
checkpoints, not a natural full-fight or supply-acquisition proof. Existing
Camping results remain attributed to their original tested source and jobs.

## Sources

- [Maggot Squasher, revision 15282000](https://oldschool.runescape.wiki/w/Special:PermanentLink/15282000)
- [Maggot Exterminator, revision 15282002](https://oldschool.runescape.wiki/w/Special:PermanentLink/15282002)
- [Combat Achievements, revision 15372533](https://oldschool.runescape.wiki/w/Special:PermanentLink/15372533)

The shared plugin documents the content-owned restore query. General Wiki
retroactive-completion guidance has content-specific exceptions; this repair
uses the existing server contract for these plain kill-count tasks.

Refs #153
