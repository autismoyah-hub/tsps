# GAP: our best creatures against real ones, in the owner's recorded words

Private (cache-derived). Folder `{ROOT}/studies/research-gap/`. `P` = `{PYTHON} -I`.

```
CORPUS.md      44 owner statements on model quality, verbatim, grouped into 12 faults with status
PREREG.md      17 measures and the test, written before any was computed
LOOK.md        what I saw on our five, written before reading any measure
P probe.py     loads every entry (feasibility, run before PREREG was final)
P board.py contact | pairs     boards/contact-*.png, boards/pair-*.png, boards/big-*.png
P measures.py  the 17 measures on 41 creatures -> data/measures.json (2 minutes)
P stats.py     LOO + 20,000 relabellings -> data/stats.json (10 minutes)
P hotspots.py  boards/hot-*.png: where the surviving measure sits
P gapcheck.py MODEL_OR_NPC_JSON     the checker (exit 0 / 2)
```

Groups: REAL 30 (2019 on, plus Tekton; includes the Minotaur, Lynx, Nex, Vardorvis, Branda, Eldric, Araxxor); OURS 11
= BEST 5 (Warden-General III, watcher III, star hound 3, Armillary sentinel, Zodiac ray) + 6 rejected (watcher I, II,
hound 2, Warden-General I, II, the orrery spider). Rule: direction as predicted, leave-one-out balanced accuracy >= 0.80,
and at least 4 of the 5 BEST called ours.

## 1. Result: 1 of 17 survives, narrowly

| measure (owner's word) | REAL median | OURS median | LOO bal. | BEST as ours | survives |
|---|---|---|---|---|---|
| C9 `concave` ([quoted words withheld]) | 0.065 | 0.040 | **0.81** | **4/5** | **YES** |
| P4 `sil_turn` (silhouette cut) | 0.80 | 0.71 | 0.76 | 4/5 | no (closest) |
| C5 `sil_cut` | 0.64 | 0.46 | 0.69 | 2/5 | no |
| C6 `join_line` (how parts meet) | 0.46 | 0.38 | 0.62 | 4/4 | no |
| C3 `plane_by_part` | 0.62 | 0.48 | 0.63 | 2/5 | no |
| others (C1 C2 C4 C7 C8 C10 C11 C12 C13 P1 P2 P3) | | | 0.41-0.65 | | no; C1 C2 C10 C13 P2 point the WRONG way |

Chance: by 20,000 relabellings of the same 41 values, each measure passes the full rule with probability 0.0001 to
0.0013; expected survivors by chance **0.009** of 17 (the measures are correlated, so fewer independent chances). One
survivor is therefore unlikely to be luck, but it is thin: 0.81 against a 0.80 bar; six real creatures sit on our side
(Nex, Vardorvis, Araxxor, the Whisperer, Zebak, Kephri), the sentinel sits far on the real side (0.258, its rings), and
watcher I is called real.

**Band for `concave`** (share of edge length inside one paint that folds INWARD 30 degrees or more): REAL p10..p90
0.033..0.314, min 0.019, median 0.065; cut **0.047**. Ours: Warden-General III 0.041, watcher III 0.046, hound 3 0.045,
ray 0.031, sentinel 0.258; rejects 0.020 to 0.058.

Wrong-way findings (they cannot count, but they correct beliefs): our creatures have MORE edge flow aligned with their
parts (C1: watcher III 0.85 against real p90 0.80) and MORE relief from a smoothed shell; real ones are LESS mirror
symmetric. Real meshes are not tidier lofts than ours; ours are the tidy ones.

Post-hoc, cannot count: on C6 (`join_line`) our BEST are lower than our rejects (Warden-General III 0.14, sentinel 0.06
against rejects 0.32 to 0.55; real median 0.46): the newer, cleaner builds join parts without a line (no crease or
paint change where a label changes). A lead for the next pre-registration, not a finding.

## 2. Where the survivor points (`boards/hot-*.png`), against LOOK.md

On the Minotaur and the Lynx the red (concave folds) lies BETWEEN masses: pectorals, abdominal blocks, the shoulder
behind the scapula, the elbow, the thigh; on Tekton between every slab. On ours it lies at TRIM and joins: the
Warden-General's gold borders and plate edges, the watcher's chest plates and wrists; the hound's flank is almost
clear. So it points at LOOK items [quoted words withheld] (watcher III), "bevelled boxes, no decided
plane" (Warden-General III) and [quoted words withheld] (hound): forms built convex everywhere, with depth only at seams.
It does NOT point at: printed-on features, paint that does not follow form, uncut silhouettes, low value range with
marks too thin for game size, the mottle patchwork, or the sentinel's low density of detail. Those stay as questions.

## 3. Review questions (seen faults no measure caught; look at the named boards, real left, ours right)

1. **Are the features carved or printed?** (F5 [quoted words withheld]) Watcher III's eyes are flat yellow diamonds with a black
   outline, a diamond forehead gem, two black nostril slits, on a pale mask with no brow fold or socket
   (`big-stone-watcher-iii-ours.png`); the Minotaur's eyes sit dark under a brow fold with a red lid patch
   (`big-stone-watcher-iii-real.png`). Ask: does each eye, nostril, mouth have a FORM around it (a socket, a brow, a
   cheek plane) that shades on its own before paint?
2. **Does paint follow the form?** (F3, F8 [quoted words withheld]) The watcher's tan/blue boundary crosses head and shoulder
   yoke in camouflage blobs (`pair-stone-watcher-iii.png`); the Minotaur's hair line follows the mane and the belt.
   Ask: can each colour region be named as an anatomical or material part?
3. **Are there big decided planes, or a patchwork?** (F8 "uneven", F3 "splotchy") The hound's flank and legs are many
   small triangles in four or five near-equal blues; the Lynx is a few big calm planes with one decided white bib
   (`pair-star-hound-3.png`). Ask: per view, how many planes can be counted at game size, and is any small tone change
   there for no drawn reason?
4. **Is the silhouette cut?** (F8 "crisp", "misshaped") The Lynx's outline has ear tufts, cheek ruff points, a tail
   stub; Tekton tapers to a tall spike with big fists; ours are rounder and rectangular (the Warden-General from
   shoulder to boot; the ray a plain rhombus) (silhouette rows of the pair boards). Ask: name the three features that
   make the black shape; where does it taper?
5. **Do the marks carry at game size?** (F6 "definition", [quoted words withheld]) Tekton is dark with BIG bright marks;
   the Warden-General is mid-grey on mid-grey with 1 to 2 pixel gold borders on every plate (`pair-warden-general-iii.png`,
   game-size rows). Ask: at game size, which marks survive, and is there a value contrast between body and marks?
6. **Is there enough designed detail for a genuine boss?** (F9 [quoted words withheld]) The
   sentinel is clean but plain tubes and rings beside the Warden of Elidinis's skulls, patterns, framed face and fan
   crest (`pair-armillary-sentinel.png`). Ask: what designed small things (motifs, not noise) does it carry, and where
   is the focus?
7. **Do parts meet at a decided line?** (F5, post-hoc C6) Ask: where a limb, plate or horn meets the body, is there a
   crease or a colour change, or does the surface roll over without one?

## 4. Proposed lines for the creature brief (every one UNTESTED)

- UNTESTED (answers [quoted words withheld], [quoted words withheld]; the one surviving measure): carve the forms. Concave folds of 30
  degrees or more inside one paint on at least 0.047 of the edge length (real median 0.065), and they must lie BETWEEN
  masses (muscle groups, slabs, the brow, under the jaw, behind the shoulder), not along trim; check with
  `gapcheck.py` and look at `hotspots.py`'s picture. A groove cut along a border to pass the number is a fail.
- UNTESTED (answers [quoted words withheld], "playdough"): every eye, nostril and mouth has a form of the skin around it (socket,
  brow fold, cheek plane) that reads in the lit grey picture BEFORE paint; a feature that exists only as paint on a
  smooth mask is not done.
- UNTESTED (answers "splotchy", [quoted words withheld], [quoted words withheld]): every colour region has a name (a part or
  a material); a boundary that crosses a form at random is removed.
- UNTESTED (answers "crisp", "misshaped", [quoted words withheld]): review the black silhouette at game size beside the
  nearest real one; name its taper and three cut features (tufts, spikes, notches, fists).
- UNTESTED (answers "definition", [quoted words withheld]): at game size, the body and its marks differ in value; marks
  are bigger than 2 pixels; no border trim on every plate.
- UNTESTED (answers [quoted words withheld]): a boss carries designed motifs with one focus (the
  face or chest), not plain tubes; compare density beside a real boss of its kind.
- UNTESTED (answers "uneven"): no tone patchwork of small near-equal triangles; few big planes per view.

## 5. Could not establish

- 16 of 17 measures failed: as for the helms, most of what I see is not in a whole-mesh number.
- The survivor is near the bar and has a plain loophole (grooves along trim); nobody has built to it.
- The owner has not seen these boards; sitting 2026-11-30-d is unanswered, so [quoted words withheld] is not tied to any place.
- Pictures are rest pose, by our renderer; weighted real bosses draw in bind pose (Maggot King, Whisperer,
  Nightmare), which moves picture measures; translucent faces were left out of mesh measures (Eldric is 68%
  translucent). The ray's neighbour (Maggot King) is a poor match.
- Edge flow and rings were measured with label-free k-means parts, not anatomical parts; a better segmentation might
  change C1 to C3.
