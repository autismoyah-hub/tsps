# Section proposed for `teacher/project-docs/PRODUCER.md` (not applied; for the Teacher or a Fable review to place)

Proposed position: a new section after "One order, start to finish", titled as below, and one line added to step 6 of
that list ("For a room, also follow 'Composing a room' below"). Evidence: the research report of 2026-11-29 (returned to the Teacher), `studies/research-rooms/SCORES.md` and
`ROOMS.md` on the asset machine; the armoury graded Usable (judged too sparse, with placement that seemed arbitrary) (sitting 2026-11-28-b).

---

## Composing a room

Ordering good pieces does not make a good room. The wardens' armoury had six passed pieces and was graded Usable,
[quoted words withheld]: five pieces in 69 floor tiles where the game's
armouries hold 8 to 14, none repeated where theirs repeat a suit of armour 4 to 8 times, two of four wall pieces out in
the floor, a dummy in the centre that 15 of 16 real armouries keep clear, nothing on the walls. When an order is a
whole room, compose it by `asset-system/author-pack/ROOMS.md` (the measured rules: how many pieces per 10 floor tiles
for the room's kind, what goes against the walls, what groups with what, the signature piece repeated, what faces the
door, what the centre holds, how much hangs on the walls). Before the room's pieces are ordered, count from its plan
what it needs and order that many (repeats of one piece count; a room usually needs 2 to 3 times more pieces than it
has kinds of piece, and wall decorations as well as floor furniture). After composing, run the checker and put its
full output in your report. The checker measures closeness to how the game furnishes rooms, not the owner's grade:
a PASS is the floor, not the verdict; the sitting still decides, and both rooms the owner graded Strong miss some of
its rules. Do not loosen a rule to pass; if a rule cannot be met with the pieces that exist, say which and order the
missing pieces.

### Room checklist (each line answered in the report, with the number)

1. Kind of room and its plan: floor tiles N, wall edges E, door tile(s), from `roomcheck.py`'s first line.
2. Piece count: rate for the kind from ROOMS.md D1 x N / 10, at least the p25 count, aiming at the median. Ordered
   pieces cover the count, including repeats. Wall decorations ordered: at least E / 10, aiming at 2 x E / 10.
3. The signature piece chosen and placed 4 to 6 times (a library 6 to 10), along walls at even intervals or in runs,
   in mirror pairs about the door's or the room's axis.
4. Far zone: the main piece or the best decoration stands on the door's line against the far wall.
5. Walls: every bed, chest, cabinet, shelf, bookcase, rack, stand, statue, plant, barrel and light stands against a
   wall (at least 3 in 4 of them; name any that does not and why). No straight wall bare for more than the kind's limit
   (armoury 5 tiles, library 3, most rooms 4).
6. Groups: seats touch tables or workstations; a workstation has a companion (its pail, stool, crate); storage in
   heaps of 2 or 3; lights flank the main piece or the door.
7. Centre: clear (armoury, workshop, smithy, bedroom, temple; a floor feature may mark it) or the main table group
   (library, kitchen, bank, throne room), by the kind.
8. Paths: every piece reachable from the door; a two-tile path to most; free floor inside the kind's range.
9. Pictures from the room's fixed cameras (overview, three-quarter, played), the composed room beside the previous
   version if there is one.
10. `python -I studies/research-rooms/roomcheck.py PLACEMENTS.csv --kind KIND`: paste the whole output. PASS, or each
    MISS with what you did or why it stays; and the list of pieces a better room would still need that nobody has built,
    as an order ready to place (piece, placement shape, size, count, tile and rotation).
11. Prediction in `docs/verdicts/COUNT.md` before the owner sees the room.
