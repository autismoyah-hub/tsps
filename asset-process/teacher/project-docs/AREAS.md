# Areas

One row of truth per area of the asset system: where it stands against the five steps of [PLAN.md](PLAN.md) (harness, read the real ones, rebuild, originals, the owner's sitting). Rewritten in place, like [STATE.md](STATE.md); moved out of it on 2026-11-26 because the state kept outgrowing its limit. Figures and evidence are in `asset-system/FINDINGS.md`, by section.

Grades: Verified means observed by the asset Teacher on the asset machine on the date given; Owner means the owner's recorded answer in `docs/verdicts/`.

## Objects: furniture and props

- **Harness** (Verified 2026-11-25): line-ups on terrain in the game's classic renderer; several pieces together with `vignette.py`. Faithful to the live game (section 22).
- **Rules:** `asset-system/author-pack/RULES.md`, from the real meshes and the client's lighting code.
- **Rebuilds** (Verified; sections 6 to 10): crate and lectern by fresh authors from the written pack alone, colour distance 0.48 and 0.53 against a ceiling of 0.03. Owner: took a real bench for ours.
- **Originals** (Owner, 2026-11-26): a scribes' hall set of four (desk, chair, cabinet, bookcase), each graded Strong, and the set Strong. Direction A-022.
- **Not tried:** plants and organic shapes, stone and metal pieces, large architecture, transparency, animated scenery.

## Rooms and areas

- **Harness** (Verified 2026-11-26; section 21): pieces placed tile by tile as the game's own shapes (wall, corner, wall decoration, ground decoration, object), fixed cameras.
- **First room** (Owner, 2026-11-26): a scribes' hall, a kit of fifteen tile-sized pieces by one fresh author around the furniture set. Graded Strong; the owner prefers its look to the October lobby's and chose the stonework (relief and wear) as the first change. A second pass rebuilt walls and floor as modelled blocks with relief (section 27); its author calls it better by a modest margin. Owner, 2026-11-26, shown both unlabelled: prefers the new stone.
- **Not tried:** real walls and floors in line-ups (the tools place real objects as one shape only); a second room; the tower lobby (A-015); outdoor areas and terrain; roofs.

## Weapons

- **Harness** (Verified 2026-11-26; sections 16, 18, 22): a real figure holding real items and ours, standing or at any frame of a real animation, on a dedicated build of the owner's client (A-021). Since 2026-11-26 it paints the figure as the game does; before that it did not.
- **Rebuilds** (Owner, 2026-11-26): a sword and a crossbow; the owner could not tell either from its twin. Judged from the unfaithful pictures.
- **Originals** (Owner, 2026-11-26): a Meridian longsword, glaive and staff, each Strong, ranked glaive, staff, sword. First judged from the unfaithful pictures; corrected to the priority real weapons use, redrawn, and shown again: the owner let all three grades stand, and answered Yes to whether the glaive and staff move like weapons of the game in the server's own walk.
- **Movement** (Verified): ours carry the label real weapons do and follow the hand through stab and walk frames.
- **Server animations** (Verified 2026-11-26; section 24): read from the owner's TSPS server data with `tools/tsps_anims.py`; the glaive and staff were posed with slightly wrong stand and walk numbers and are to be redone.
- **Not established:** residual spots on the glaive and staff in the faithful mode; inventory and ground models; bows, shields, two-handed heavy weapons.

## Armour

- **First original set** (Candidate, with the owner as sitting 2026-11-26-e): the Meridian warden's armour, five pieces by five authors (helm, body with pauldrons and arms, gauntlets, tasseted legs, boots), clean in run and slash. The first cuirass, sleeveless, read as a bib and was replaced. House steel under choice: hue 21 draws green; real steel is hue 0.

- **Harness** (Verified 2026-11-26; section 19): real items in any slot, ours standing in a slot with the right body parts hidden, whole outfits. Real armour re-encoded through our encoder is pixel-identical to the real items in motion.
- **Rules** (Verified; section 20): a label atlas and a rule-based labelling tool; reliable for helms, gloves, boots and sleeveless bodies, not for sleeved bodies, legs, skirts or capes.
- **Rebuilds** (Verified 2026-11-26; sections 23 and 25): a chainbody rebuilt in the faithful mode matches the real one (size exact, 4 colours against 3, colour distance 0.74, 896 to 1,520 differing pixels in motion on a piece of about 2,000) with no tear or gap: the first armour rebuild to pass, a sleeveless body. Owner, 2026-11-26: could not tell the chainbody from its twin, and graded it Strong. The helm moves without tearing but was judged in the unfaithful mode; its author is re-judging it.
- **Not tried:** originals; the female figure; chat-head and inventory models; newer animations that move by bone weights.

## People and creatures

- Nothing built. Real figures are drawn as static yardsticks. An ordinary NPC is drawn like scenery (read from the source, not tested). The owner allows figures made from the game's own body parts (A-025).

## New animation

- Nothing built. The client's own code poses our models at real frames; an encoder for new actions exists in the owner's tools and has not been exercised.

## The system itself

- **Authors** (2026-11-26; FINDINGS section 29): on one held-out rebuild (Drawers 25702) Claude Fable 5.2, Claude Opus 5.5 and GPT-6.2 Astra all came close (picture score 1.49, 4.22, 2.76; lower is closer); GPT-6.2 Sol declines to rebuild real objects but made a good original map table, as did Astra. One object, unequal effort: feasibility, not ranking. A run is 10 to 60 minutes and 150 to 700 thousand tokens of the included subscriptions.
- **Reviewer:** one trial, at chance on telling ours from real (section 15).
- **Count of the owner's answers against the Teacher's predictions:** `docs/verdicts/COUNT.md`.
- **Where it lives:** `asset-system/` in this repository (A-024); the working copy and everything it needs to run are on the asset machine.
- **Not done:** playbooks from the briefs; cost per accepted asset; concept pictures against designing directly.
