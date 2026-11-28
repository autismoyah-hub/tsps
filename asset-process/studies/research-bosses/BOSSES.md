# End-game bosses: what the real ones are made of

Private to this machine (cache-derived). Research of 2026-11-26/27, step 1 of the boss study. Folder:
`{ROOT}/studies/research-bosses/`. `P` = `{PYTHON} -I`, run here.

```
P boss_find.py "REGEX[,REGEX]"     names in the NPC census (id, size, models, stand, walk, scale, light)
P boss_measure.py                  -> data/bosses.json   (definition, models, skeleton, sequences, relatives, cmesh.measure, style check)
P boss_table.py > data/bosses-table.txt    every table below, with all columns
P boss_sheet.py OUTDIR [cox tob toa own other]    contact sheets
```

Sources. MEASURED: the NPC census `studies/research-npc/census/*.jsonl` (made by `asset_npc.exe census` from this
cache; spot-checked: `asset_npc.exe CACHE dump 15742` gives "Maggot King", model 61512, 42,556 bytes, ambient 30,
stored contrast 30, size 5, stand 13926, walk 13927, as the census says); spot animations and objects read from
the cache (archive 2 groups 13 and 6, decoders ported from the client's `spot_animation.rs` and `object.rs`: 4,008
and all objects decoded, 0 failures); rest meshes through `asset_npc.exe merged`; measures by
`research-creatures/cmesh.py` and `style_npc.py` unchanged (their cache directory repointed here). KNOWLEDGE (mine,
not the cache's): which names are raid bosses, which NPC id is which phase, and some attack sequences (marked
INFERRED). Attack and death ids otherwise come from the owner's server tables through `npc.server_anims`.
"Contrast" in the tables is the client's value (stored value times 5), as the census prints it.

Contact sheets (each strip: game size standing | close standing | three frames of an attack), in
`{ROOT}/render/research-bosses-sheets/`: `bosses-cox-1.png`, `-cox-2`, `bosses-tob-1`, `-tob-2`,
`bosses-toa-1`, `-toa-2`, `bosses-own-1` (Nex, Nightmare, Phosani's, Doom, Doom burrowed, Maggot King),
`bosses-other-1..3`. Skeletal (keyframe) sequences are played by the existing harness from the cache.

## 1. "Maggot king": what was found

Names containing maggot, larva, grub, worm, marquess (all listed by `P boss_find.py "maggot,larva,grub,worm,marquess"`):

| id | name | size | combat | models | note |
|---|---|---|---|---|---|
| **15742** | **Maggot King** | **5** | **741** | **61512** | stand 13926, walk 13927; ambient 30, contrast 150; bone-weighted; base 2653 |
| 15741 | Maggot King Corpse | 5 | 0 | 61512 | same model, sequence 13937 |
| 15740, 16293 | Maggot marquess | 1 | 0 | 61513 | scale 25: the pet-sized version, same base 2653 |
| 15743 | Ur-maggot larvae | 1 | 65 | 61578 | its minion (borrows sequences 1270 / 1269 of an older grub) |
| 14810 | Grub | 1 | 0 | 59830 | scale 42, unrelated critter |
| 14710-14713, 14788-9 | Demonic (range / magic / melee) larva, Giant demonic larva | 1, 2 | 0 | 56457..56461 + 56459 | the Doom of Mokhaiotl's minions, not a boss |
| 7534-7537 | Lux grub | 3 | 0 | 32695, 32696 | Chambers of Xeric (Vespula's room) |
| 482, 966, 1678, 671.., 820, 4916, 13981-2 | Cave bug larva, Kalphite Larva, Bloodworm, Grubfoot, Wormbrain, Grubor, Worm Tongue | 1-2 | | | not candidates |

**The owner has since confirmed that [quoted words withheld] is the boss's exact name in the game. An NPC with exactly that
name IS in this revision 239 cache: NPC 15742 "Maggot King"** (census line: size 5, combat 741, models [61512],
stand 13926, walk 13927, option "Attack"). No other creature was substituted for it. It is the only boss-sized one
of the candidates above (5 tiles, combat 741), with its own corpse, pet and minion. Its model ids (61512; the highest boss ids in this cache; Doom is 56469)
say it is the newest boss here. I do not know this boss from my own knowledge (it postdates what I can vouch for);
everything said of it below is measured or seen on the sheet. Its attack (13928) and death (13936) are INFERRED
from the 15 sequences on its base by length and by looking; the server tables have no row for it.

## 2. The bosses found (48 forms; `data/bosses-table.txt` has every column)

Rig: `labels N` = classic vertex labels (moved by frame-base groups); `WEIGHTS` = bone weights (moved by keyframe
curves). Base = frame base id; g/b = groups / bones in it. "On base" = NPC definitions standing on it (distinct
model lists). Seqs = sequences on the base (how many are skeletal). Forms = NPC ids with the name / distinct model lists.

| npc | boss | tiles | models | faces | verts | rig | base (g/b) | on base | seqs (skel.) | forms | colours (for 90%) | amb / con | scale |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| obj 29881 | Great Olm head | 5x8 | 2 | 2,170 | 1,327 | labels 74 | 1661 (83/0) | 2 objects | 28 (0) | 2/1 | 37 (18) | 0/0 | 128 |
| obj 29884 | Great Olm hand (rock) | 5x8 | 2 | 2,543 | 1,716 | labels 67 | 1662 (81/0) | 2 objects | 19 (0) | 2/1 | 24 (13) | 0/0 | 128 |
| obj 29887 | Great Olm hand (crystal) | 5x8 | 2 | 1,213 | 776 | labels 34 | 1660 (53/0) | 2 objects | 4 (0) | 2/1 | 24 (12) | 0/0 | 128 |
| 7541 | Tekton | 4 | 1 | 2,073 | 1,119 | labels 89 | 1668 (129/0) | 10 (1) | 25 (0) | 4/1 | 20 (9) | 10/25 | 128 |
| 7566 | Vasa Nistirio | 5 | 1 | 2,774 | 1,461 | labels 80 | 1666 (131/0) | 5 (1) | 11 (0) | 2/1 | 32 (15) | 10/25 | 128 |
| 7530 | Vespula | 5 | 1 | 2,534 | 1,137 | labels 189 | 1670 (219/0) | 9 (2) | 17 (0) | 3/1 | 25 (11) | 0/0 | 128 |
| 7527 | Vanguard | 3 | 1 | 2,072 | 1,064 | labels 110 | 1669 (255/0) | 7 (1) | 22 (0) | 7/1 | 18 (10) | 0/0 | 128 |
| 7563 | Muttadile | 5 | 1 | 1,498 | 753 | labels 89 | 1667 (54/0) | 9 (4) | 12 (0) | 3/1 | 14 (7) | 0/0 | 200 |
| 8360 | Maiden of Sugadinti | 6 | 1 | 3,229 | 1,644 | labels 251 | 1812 (255/0) | 28 (13) | 7 (0) | 27/12 | 36 (8) | 30/0 | 128 |
| 8359 | Pestilent Bloat | 5 | 1 | 3,245 | 1,896 | labels 240 | 1823 (194/0) | 7 (1) | 5 (0) | 5/1 | 34 (12) | 0/220 | 132 |
| 8355 | Nylocas Vasilias | 4 | 1 | 3,424 | 1,766 | labels 162 | 1800 (232/0) | 62 (14) | 38 (0) | 17/3 | 40 (16) | 0/0 | 170 |
| 8388 | Sotetseg | 5 | 1 | 2,718 | 1,480 | labels 121 | 1822 (155/0) | 9 (1) | 6 (0) | 7/1 | 20 (12) | 0/0 | 128 |
| 8340 | Xarpus | 5 | 1 | 2,307 | 1,275 | labels 174 | 1811 (133/0) | 15 (2) | 7 (0) | 13/2 | 21 (8) | 0/0 | 64 |
| 8370 | Verzik Vitur P1 (throne) | 5 | 1 | 4,108 | 2,041 | labels 219 | 1796 (217/0) | 7 (2) | 5 (0) | 26/5 | 66 (28) | 0/0 | 128 |
| 8372 | Verzik Vitur P2 | 3 | 1 | 4,095 | 2,075 | labels 242 | 1808 (255/0) | 7 (1) | 6 (0) | 26/5 | 70 (34) | 0/0 | 128 |
| 8374 | Verzik Vitur P3 | 7 | 1 | 4,096 | 2,110 | labels 246 | 1810 (255/0) | 14 (3) | 19 (0) | 26/5 | 61 (22) | 0/0 | 128 |
| 11789 | Akkha | 3 | 3 | 5,906 | 2,997 | WEIGHTS | 2065 (134/73) | 15 (3) | 38 (38) | 9/1 | 185 (26) | 0/0 | 192 |
| 11778 | Ba-Ba | 5 | 2 | 3,350 | 1,603 | WEIGHTS | 2063 (108/129) | 18 (10) | 20 (20) | 4/1 | 101 (35) | 0/0 | 160 |
| 11719 | Kephri | 5 | 1 | 3,766 | 1,930 | WEIGHTS | 2079 (24/225) | 8 (2) | 13 (13) | 5/1 | 41 (15) | 0/0 | 175 |
| 11730 | Zebak | 9 | 1 | 2,034 | 1,031 | WEIGHTS | 2074 (0/47) | 6 (2) | 10 (10) | 5/1 | 79 (32) | 30/0 | 128 |
| 11753 | Elidinis' Warden P2 | 5 | 1 | 4,351 | 2,212 | WEIGHTS | 2062 (134/73) | 29 (8) | 38 (38) | 11/4 | 60 (28) | 0/0 | 300 |
| 11756 | Tumeken's Warden P2 | 5 | 1 | 4,309 | 2,224 | WEIGHTS | 2062 | 29 (8) | 38 (38) | 10/4 | 64 (31) | 0/0 | 300 |
| 11761 | Elidinis' Warden P3 | 5 | 2 | 3,198 | 1,653 | WEIGHTS | 2062 | 29 (8) | 38 (38) | 11/4 | 56 (15) | 0/0 | 300 |
| 11762 | Tumeken's Warden P3 | 5 | 2 | 3,543 | 1,803 | WEIGHTS | 2062 | 29 (8) | 38 (38) | 10/4 | 57 (15) | 0/0 | 300 |
| 11751 | Obelisk (Wardens) | 3 | 6 | 5,180 | 3,196 | labels 31 | 2061 (154/0) | 4 (3) | 13 (0) | 6/4 | 27 (10) | 40/300 | 112 |
| 11278 | Nex | 3 | 1 | 3,080 | 1,617 | labels 194 | 1980 (177/0) | 7 (1) | 15 (0) | 5/1 | 41 (19) | 0/0 | 128 |
| 9425 | The Nightmare | 5 | 1 | 4,078 | 2,516 | labels 243 | 1888 (255/0) | 32 (17) | 41 (0) | 15/12 | 31 (8) | 15/500 | 128 |
| 9416 | Phosani's Nightmare | 5 | 1 | 4,078 | 2,516 | labels 243 | 1888 | 32 (17) | 41 (0) | 13/7 | 31 (8) | 15/500 | 128 |
| 14707 | Doom of Mokhaiotl | 5 | 5 | 3,810 | 1,918 | WEIGHTS | 2441 (88/90) | 3 (2) | 18 (18) | 1/1 (+2 named forms) | 66 (25) | 15/100 | 90 |
| 14709 | Doom (Burrowed) | 5 | 3 | 3,794 | 1,901 | WEIGHTS | 2441 | 3 (2) | 18 (18) | | 64 (26) | 0/0 | 90 |
| 15742 | Maggot King | 5 | 1 | 3,733 | 1,855 | WEIGHTS | 2653 (212/155) | 4 (2) | 15 (15) | 1/1 (+corpse, pet) | 37 (14) | 30/150 | 128 |
| 8061 | Vorkath | 7 | 1 | 2,901 | 1,497 | labels 251 | 1782 (159/0) | 7 (1) | 15 (0) | 6/2 | 33 (10) | 0/0 | 104 |
| 2042 | Zulrah | 5 | 1 | 1,934 | 988 | labels 239 | 184 (171/0) | 3 (3) | 10 (0) | 3/3 | 33 (16) | 0/0 | 128 |
| 8615 | Alchemical Hydra | 6 | 1 | 3,168 | 1,556 | labels 246 | 1851 (231/0) | 3 (1) | 7 (0) | 10/9 | 22 (9) | 30/220 | 80 |
| 319 | Corporeal Beast | 5 | 1 | 2,904 | 1,456 | labels 166 | 304 (92/0) | 3 (1) | 11 (0) | 1/1 | 11 (5) | 0/0 | 300 |
| 5862 | Cerberus | 5 | 1 | 2,161 | 1,101 | labels 98 | 367 (98/0) | 5 (2) | 12 (0) | 5/2 | 30 (16) | 0/0 | 128 |
| 12077 | Phantom Muspah | 5 | 1 | 2,435 | 1,219 | WEIGHTS | 2095 (64/54) | 19 (6) | 20 (20) | 6/4 | 31 (9) | 0/0 | 112 |
| 12223 | Vardorvis | 2 | 2 | 1,995 | 1,006 | WEIGHTS | 2151 (6/99) | 10 (3) | 11 (11) | 6/2 | 42 (12) | 0/0 | 128 |
| 12191 | Duke Sucellus | 7 | 1 | 2,440 | 1,245 | WEIGHTS | 2134 (218/255) | 9 (2) | 10 (10) | 8/1 | 53 (19) | 0/175 | 128 |
| 12214 | The Leviathan | 7 | 2 | 2,599 | 1,303 | WEIGHTS | 2148 (72/57) | 6 (3) | 19 (19) | 4/2 | 44 (15) | 15/100 | 128 |
| 12204 | The Whisperer | 3 | 5 | 4,421 | 2,297 | WEIGHTS | 2138 (208/236) | 7 (2) | 32 (32) | 4/1 | 64 (11) | 0/380 | 150 |
| 12821 | Sol Heredit | 5 | 4 | 3,872 | 2,018 | WEIGHTS | 2245 (104/108) | 8 (5) | 24 (24) | 4/2 | 66 (19) | 25/100 | 300 |
| 13668 | Araxxor | 7 | 2 | 3,730 | 2,220 | labels 170 | 2351 (222/0) | 5 (3) | 18 (0) | 2/2 | 37 (10) | 0/0 | 128 |
| 13685 | Amoxliatl | 3 | 2 | 1,886 | 970 | WEIGHTS | 2244 (12/214) | 21 (5) | 50 (50) | 4/1 | 19 (7) | 0/0 | 100 |
| 14009 | The Hueycoatl | 7 | 2 | 4,003 | 2,038 | WEIGHTS | 2359 (16/163) | 5 (2) | 15 (15) | 5/2 | 47 (18) | 40/300 | 128 |
| 12596 | Branda (Royal Titans) | 3 | 1 | 2,510 | 1,287 | labels 139 | 2400 (224/0) | 4 (2) | 16 (0) | 2/2 | 49 (15) | 0/0 | 120 |
| 14147 | Eldric (Royal Titans) | 3 | 1 | 3,094 | 1,721 | labels 161 | 2395 (255/0) | 4 (2) | 16 (0) | 2/2 | 33 (7) | 0/0 | 120 |
| 14176 | Yama | 5 | 3 | 3,070 | 1,554 | WEIGHTS | 2404 (52/104) | 5 (2) | 12 (12) | 5/2 | 31 (9) | 0/0 | 160 |

**The Great Olm is not an NPC.** NPCs 7550-7555 named [quoted words withheld] are 12-face stubs (model 32709) with no
sequences: hit boxes. What the player sees are three animated OBJECTS of 5 by 8 tiles found beside the raid's
other objects, named as disguises: "Large hole" 29880/29881 (the head, models 32523 + 32522, base 1661, 28
sequences), "Large rock" 29883/4 (a hand, base 1662, 19 sequences), "Crystalline structure" 29886/7 (the other
hand, base 1660, 4 sequences). I identified them by model and sequence neighbourhood and by looking at the drawing
(a white dragon head rising from green crystal rock: `bosses-cox-1.png`, last strip); measured and drawn through the
NPC harness as definitions of the same model lists (object lighting and model merging differ slightly: the picture
is not a proof of how the client draws the object).

## 3. What an end-game boss is, in numbers, against the recent mid-sized creature

Median (10th to 90th centile). Mid-sized = the 37 recent quadrupeds, spiders and humanoid monsters of the era study
(`research-creatures/data/era.json`, group R, plans q, s, h; 1 to 3 tiles, median 2). Bosses = the 48 forms above;
"owner's" = the 31 forms of the three raids, Nex, the Nightmare, the Doom and the Maggot King.

| measure | mid-sized recent (37) | all bosses (48) | owner's bosses (31) |
|---|---|---|---|
| size, tiles | 2 (1-3) | 5 (3-7) | 5 (3-8) |
| extent in units, x / height / z | 171 / 226 / 234 | 516 / 736 / 576 | 513 / 796 / 530 |
| faces | 1,230 (836-1,920) | 3,130 (2,030-4,110) | 3,340 (2,070-4,300) |
| vertices | 638 (454-986) | 1,630 (1,030-2,220) | 1,770 (1,060-2,520) |
| median face, ten-thousandths of the surface | 5.65 (3.4-8.4) | 1.42 (0.61-2.8) | 1.38 (0.61-2.4) |
| colours | 18 (10-28) | 37 (20-66) | 37 (20-70) |
| colours covering 90% of the surface | 7 (4-13) | 14.5 (8-28) | 15 (8-31) |
| tones per hue family | 5.7 (2.9-11) | 5.4 (3.1-10.7) | 5.7 (3.8-11.2) |
| median lightness step across a tone edge | 8 (4-16) | 8 (4-12) | 8 (4-12) |
| largest one-colour patch, share of surface | 0.18 (0.08-0.44) | 0.125 (0.07-0.52) | 0.11 (0.06-0.29) |
| in-surface edges that are paint boundaries | 0.168 (0.107-0.235) | 0.192 (0.124-0.338) | 0.218 (0.155-0.338) |
| small contrasting islands inlaid in the skin | 26 (10-81) | 119 (43-352) | 105 (43-362) |
| surface in such islands | 0.079 | 0.119 | 0.121 |
| surface on thin parts | 0.43 (0.31-0.90) | 0.55 (0.39-0.77) | 0.58 (0.33-0.77) |
| median fold, degrees | 21.7 (15.3-28.4) | 20.1 (12.1-25.9) | 20.8 (10.5-24.7) |
| tone follows facing | 0.12 | 0.10 | 0.09 |
| tone by part | 0.46 (0.22-0.73) | 0.48 (0.22-0.77) | 0.56 (0.17-0.77) |
| vertices with exactly six faces | 0.31 (0.27-0.38) | 0.32 (0.25-0.39) | 0.31 (0.24-0.39) |
| separate shells | 5 (1-25) | 12.5 (2-83) | 27 (2-109) |
| largest shell, share of faces | 0.96 (0.56-1) | 0.75 (0.33-1) | 0.69 (0.33-0.98) |
| translucent faces, share | 0 (0-0.14) | 0.08 (0-0.30) | 0.10 (0-0.30) |
| split vertices | 0 | 0 | 0 |

Counts (all 48 / the owner's 31): bone weights 20 / 11 (mid-sized: 7 of 37); some translucent faces 42 / 27;
ambient or contrast set 16 / 10 (mid-sized 19 of 37); undrawn faces (alpha 255) 12 / 10; textured faces 1 / 1 (the
Wardens' Obelisk, 24 faces: the only textures in the set); flat faces over a tenth 4 / 3 (Obelisk, Eldric's ice,
Olm's crystal); several models in the definition 18 / 10; definition scale other than 128: 24 / 14; more than one
NPC id of the name 44 / 28; more than one MODEL LIST of the name 26 / 13; the base serves only that name 9 / 6;
objects animated on the boss's own base 9 / 7; 5 tiles or more 36 / 24; breaks no GAME style limit 30 / 16; breaks
at most one RECENT band 43 / 28. Sequences on the base: 16 (7-38). Groups in the base: 134 (52-255); bones in the
weighted ones' bases: 94 (57-225).

**Plainly, an end-game boss against a mid-sized recent creature:**
1. **Two and a half times everything a mesh is counted in**: 3,100 to 3,300 faces against 1,230; 37 colours against
   18; 15 colours to cover 90% of the surface against 7. Three times the linear size, so the median face is a
   QUARTER of the relative size (1.4 against 5.65 ten-thousandths of the surface). The ceiling is about 4,100 faces
   in one model (Verzik's three phases are 4,095 to 4,108, the Nightmare 4,078, Hueycoatl 4,003); more only by
   several models (Akkha 5,906 in three, the Obelisk 5,180 in six).
2. **Four times the painted detail**: 105 to 119 small contrasting islands inlaid in the skin against 26, and more
   of the in-surface edge length is a paint boundary (0.19 to 0.22 against 0.17). The Wardens and Kephri carry 350 to
   410 islands. No large one-colour hide: largest patch 11 to 12%.
3. **The same STYLE otherwise.** Tones per hue (5.4 / 5.7), tone steps (8), folds (20 degrees), tone by part and
   not by facing (0.48 to 0.56 / 0.09), irregular hand split (0.32 six-face vertices), welded (0 split vertices),
   smooth: all inside the mid-sized band. A boss is not painted by different rules; it is the recent creature at a
   larger budget. 43 of 48 break at most one RECENT band of `style_npc.py`.
4. **More separate shells.** 12.5 shells (owner's: 27) against 5; the largest shell holds 0.69 to 0.75 of the faces
   against 0.96. The one GAME limit bosses commonly break is [quoted words withheld] (13 of the 18 that break any):
   Bloat 113 (flies), the Nightmare 109 (rune ring, spores), Araxxor 173, Verzik P3 64. Armour, weapons, props,
   orbiting things and a ground decal are built as separate shells around ONE body; the rule for bosses should be
   [quoted words withheld], not a shell count.
5. **Translucency is ordinary** (42 of 48; median a tenth of the faces): veils, wings, glows, ghosts, blood, a
   shadow or pool under the body. Whole-boss translucency exists (Phantom Muspah 100%, Amoxliatl 80%, Eldric 68%,
   Vasa's crystal 53%).
6. **No textures** (one exception of 24 faces). Lighting values are NOT more common than on mid-sized creatures
   (16 of 48 against 19 of 37) but when set they are stronger: contrast 100 to 500 (the Nightmare 15 / 500, the
   Whisperer 0 / 380, Hueycoatl 40 / 300, Bloat 0 / 220).
7. **Scale in the definition is a tool**: half the forms are scaled (the Wardens and Sol Heredit are authored at
   human size and drawn at 300; Corporeal Beast 300; Xarpus 64; Hydra 80), and pets and minions are the same model
   scaled down (Maggot marquess 25, Vanguard 40, Kephriti 38).

**What a boss HAS that a mid-sized creature does not:**
- **Phases as separate NPC ids, often separate models and separate bases.** Verzik: 26 ids, 5 model lists, three
  different frame bases (1796 on the throne, 1808, 1810) plus a spider form. The Nightmare: 15 ids over 12 model
  lists on one base (awake, asleep, the totems' phases, sleepwalker forms). The Maiden: 27 ids over 12 model lists
  (blood-loss stages as different meshes). Hydra: 10 ids, 9 model lists (heads lost per phase), several bases. The
  Wardens: 4 model lists each (whole, damaged, the P3 torso with a second model 46331). Doom: three named forms
  (normal, Shielded, Burrowed with other sequences and no light values). Phase ids differ in stand and walk
  sequence even when the mesh is the same (Tekton 4 ids: 7473, 7476, 7480, enraged 7485, 7490).
- **Its own skeleton.** Median 7 NPC definitions stand on a boss's base, but they are the boss's own ids, its pet
  (`Lil' Zik`, `Tektiny`, `Smol Heredit`, `Nexling`, `Maggot marquess`: every boss here has a scaled pet on its
  base) and its minions; only 4 of 48 bases serve unrelated creatures widely (Nylocas Vasilias shares base 1800
  with 62 definitions: the Nylocas, the Araxytes; Ba-Ba with the baboons; the Wardens with their guardians;
  Amoxliatl with the Nagua). So: a boss is authored WITH a skeleton, and its family is then built on that skeleton.
- **Big skeletons.** 134 groups at the median, 6 classic bases at the hard limit of 255 (Vanguard, Maiden, Verzik P2 and P3, the Nightmare, Eldric); label counts of
  240 to 251 (the limit is 256) on Maiden, Bloat, Verzik, Nightmare, Vorkath, Zulrah, Hydra. Weighted bosses: 47 to
  255 bones (Zebak 47, Akkha and the Wardens 73, Doom 90, Maggot King 155, Kephri 225, the Whisperer 236, Duke 255).
- **Many sequences**: 16 at the median, 38 to 50 on the largest (Akkha and the Wardens 38, the Nightmare 41), where a
  mid-sized creature has stand, walk, one or two attacks, a block and a death.
- **Arena pieces animated on the boss's own base**: the Maiden as an object (32972), Vasa as an object (40245), the
  Nightmare's (37724), the Royal Titans' lava and ice (55993-56000), and the whole Great Olm. Spot animations on
  the base are rare (3 of 48): projectiles and effects have bases of their own, which this count cannot see.
- **A ground decal or pool in the mesh itself**: seen on Tekton (orange disc), Vanguard, the Maiden (red pool),
  Bloat, Nylocas Vasilias, Xarpus, Akkha, the Leviathan (black). The harness draws them; I have NOT checked how the
  live scene treats them (some are black translucent shadows, some are faces the sequences fade by alpha).
  Not verified: treat as an observation of the pictures.
- **Undrawn pivot faces** on the classic ones (Nightmare 37, Nex 30, Xarpus 41, Maiden 17, the Titans 11 to 13).

**Classic against skeletal.** Bone weights and keyframe sequences always come together: all 20 weighted forms have
only skeletal sequences on their base, all 28 labelled forms have none. By date: everything up to Nex (2022) is
classic; every boss from the Tombs of Amascut (2022) on is skeletal EXCEPT Araxxor and the Royal Titans. Weighted
bosses carry more colours (56 against 31) and somewhat more faces (3,640 against 2,900).
Of the owner's named bosses: Chambers of Xeric and Theatre of Blood, Nex, the Nightmare: CLASSIC. Tombs of Amascut
(except the Obelisk), the Doom of Mokhaiotl, the Maggot King: SKELETAL.

## 4. What each is known for visually (from the sheets; honest, short)

- **Great Olm**: a white horned dragon head with green glowing mouth and eye lines on a long pale neck coming out of
  dark green crystal rock; two hands as separate objects. It is scenery that animates.
- **Tekton**: a black faceted iron golem with orange seams, an orange visor and a blade arm. Few colours (20), all
  silhouette and glow lines. **Vasa**: grey boulders with glowing blue-violet crystal faces around a small dark
  wizard figure; half translucent. **Vespula**: a red wasp with acid-yellow bulbs and black spiked legs: three
  saturated colours. **Vanguard**: a dark pod with long yellow-orange tendrils. **Muttadile**: a plain green
  crocodile, the plainest thing here (14 colours, 1,498 faces; breaks two RECENT bands).
- **Maiden**: a pale thin woman rising on a column of red cloth, black hair lashing far outside the body in attack.
  **Bloat**: a grey stitched giant with a lit orange wound in the chest and a cloud of flies. **Nylocas Vasilias**:
  a pale armoured egg-bodied spider. **Sotetseg**: a dark red-black hunched beast edged with bright red spikes.
  **Xarpus**: a dark bat-winged creature, pale wing membranes. **Verzik**: P1 a fat grey-blue vampire queen on a
  purple and gold throne under chains (66 colours); P3 a huge spider-bodied form.
- **Akkha**: a pale warrior with a red-crested helm, a long spear and a great painted shield; 185 colours, the
  highest here, spent on the shield's device. **Ba-Ba**: a tawny baboon giant with a painted skull face and
  wrist bangles. **Kephri**: a green and maroon scarab with checkered wing tips on a dung ball. **Zebak**: a
  slate-blue crocodile nine tiles long in orange, gold and striped harness. **The Wardens**: black and gold
  pharaoh figures with red accents and translucent red or dull-gold blades, authored small and drawn at scale 300.
- **Nex**: a violet winged woman with gold armour and long pale blade-feathers; the smallest of the owner's set (3
  tiles) and built almost as a grid (six-face vertices 0.475: breaks two GAME limits). **The Nightmare**: a dark
  blue-black hooded figure with a pale chest, long claws, rising from a black pool, ringed by bright cyan runes;
  contrast 500. **Doom of Mokhaiotl**: a red demon that is mostly a huge fanged face-mask with dark horns, a small
  gold-headed body above it, cream eyes and orange mouth: three hues, read at any distance. **Maggot King**: a
  segmented yellow-to-red grub body with a dark red hood, red limbs and great dark teal-blue spined wings or
  fins: warm body against cold wings.
- Others: **Duke Sucellus** a violet-grey toothed maw under blue ice; **Leviathan** an anglerfish head with a green
  lure on a pink serpent body; **Whisperer** a thin pale figure on black tentacles; **Sol Heredit** a gold gladiator
  with a sun crest, trident and round shield; **Araxxor** a dark spider with an orange-ribbed abdomen;
  **Vardorvis** a grey armoured man overgrown with dark red branches.

**What the owner's named bosses share** (seen, and consistent with the numbers): (1) one idea readable in two or
three hues at game size (Doom red / horn / gold; Nightmare black / pale / cyan; Maggot King yellow-red / teal;
Vespula red / yellow / black), with the 37 colours spent as TONES and small inlaid marks inside those hues;
(2) a silhouette that leaves the body: wings, hair, tendrils, horns, weapons, a ring of runes (thin parts 0.58 of
the surface); (3) one glowing or saturated accent against a dark or neutral mass; (4) props and effects built into
the model as separate shells and translucent faces; (5) size 5 tiles, 3,000 to 4,300 faces, its own skeleton with a
pet and minions on it, 15 to 40 sequences, and several NPC ids for its phases.

## 5. First rebuild target: candidates

What decides it: classic or skeletal (our encoder writes labels; weights depend on the encoders of this study),
faces (the rebuild authors have passed at 1,000 to 1,600 faces, never at 3,000), one model list, a base that is
its own, sequences known.

| candidate | for | against |
|---|---|---|
| **Muttadile 7563** (CoX) | classic; 1,498 faces, 14 colours, 89 labels on a 54-group base: inside what authors have already rebuilt; attack and death in the server tables | the plainest and least "end-game" of the set; teaches little about boss detail |
| **Tekton 7541** (CoX) | classic; 2,073 faces, only 20 colours, 89 labels, base 1668 its own, 25 sequences, breaks no style limit; a rigid faceted body (what our authors do best: the sentinel) | 120 inlaid islands (the orange seams); a ground disc to settle |
| **Nex 11278** | the owner named it; classic; ONE model, 3 tiles, 3,080 faces, 41 colours, base 1980 its own (only Nex and the pet), 15 sequences, attack 9180 and death 9184 known | 194 labels and 30 undrawn pivot faces; twice the faces of any rebuild so far; wings |
| **Doom of Mokhaiotl 14707** | the owner named it; the clearest design; base 2441 serves only its three forms; 18 sequences | skeletal (90 bones), five models, 3,810 faces |
| **Maggot King 15742** | the owner named it; ONE model, 3,733 faces, 37 colours; base 2653 its own | skeletal (155 bones); attack and death only inferred |
| The Nightmare, Verzik, Maiden | named | 4,078 to 4,108 faces at the label limit, 12 model lists or three bases: last, not first |

My recommendation from step 1 alone: **Tekton as the first classic boss rebuild** (boss-sized, its own skeleton,
smallest palette, rigid forms), then **Nex** as the first of the owner's named ones; and for the skeletal route,
**the Maggot King** (one model, own base) before the Doom (five models). The parent study decides once the encoders
are proven.

## 6. Not proven, not run

- Which NPC id is which phase, and that the listed forms are the fought ones, is my knowledge plus the combat level
  in the definition; not checked against a server spawn table. NPC transform lists (opcodes 106 / 118) were not
  decoded: the census has only a flag, and none of the 48 has it set.
- Attack and death marked INFERRED (`data/inferred-seqs.json`: Bloat 8082, Akkha 9770 / 9784, Tumeken's Warden 9661,
  the Nightmare 8594 / 8612, Maggot King 13928 / 13936, Hydra 8234, Cerberus 4491 / 4495, Muspah 9920, Amoxliatl
  11530, Hueycoatl death 11681, the Olm's 7345 / 7356 / 7352) were chosen from the base's sequence list and looked
  at once on the sheet; they may be another attack, a special or a transition.
- The Great Olm was identified by me, measured through the NPC harness, not through the object path.
- "Spot animations on its base" undercounts effects (they have their own bases); which spot animations and
  projectiles a boss USES is server data and was not read. Arena objects were found only where they animate on the
  boss's base or carry its name.
- The ground discs and pools seen in the pictures were not checked against the live scene.
- Skeleton figures for weighted bosses are counts only (groups, bones); the bone hierarchy, weights per vertex and
  curves are the subject of SKELETAL.md, not of this file. Weighted models carry NO vertex labels at all (all 20: no label section),
  so `cmesh.measure`'s [quoted words withheld] is absent for them: that row of section 3 is the 28 labelled forms only.
- No measure here has been tied to the owner's eye; [quoted words withheld] is my reading of my own sheets.
- Close cells on the sheets crop the largest bosses (Duke, Zebak, Xarpus in attack); game-size cells are true size.

## 7. After the encoders (added by the parent study): routes to an original boss, and the first target

The three skeletal encoders now exist and are proven (SKELETAL.md section 6), and the rebuild test works for a
weighted creature (REPORT.md section 4). What follows is ESTIMATE, built from the measured numbers above and from
what this project has already made; none of it has been tried at boss size except the first row.

| route | what exists | an end-game boss by it needs | authors | tools still missing |
|---|---|---|---|---|
| borrowed classic skeleton | done once: the warden general on Graardor's base (about 2,000 faces, 52 labels; owner: Usable) | 3,000 to 4,300 faces, about 37 colours (15 covering 90%), 100+ inlaid islands, 120 to 250 labels followed part for part; 0 new sequences (the lender's 15 to 40) | 1 mesh author, 1 look reviewer, 2 to 3 returns | none mechanical; a mesh intersection audit in motion; the style gate retuned to the boss band (shell count, faces) |
| own classic skeleton | proven for a rigid machine (17 joints, 5 sequences; owner: Strong) | 75 to 106 joints on 134 to 255 groups; 16 sequences at the median (Tekton 25, Nex 15) of 16 to 65 frames each; an organic skin across joints, which is unproven | 1 skeleton, 1 to 2 mesh, 2 to 3 animation | pose tools (mirror, named poses, chains); skin-across-a-joint proof; intersection audit |
| skeletal | encoders for model, base and keyframes; sheet, compare, gate, line-up; a curve author with bone probe; one authored flinch | 47 to 155 bones (Zebak to Maggot King) with rest matrices; every vertex weighted (real: 76% one bone, 24% two to four, sums 255); 10 to 18 sequences of 30 to 210 cycles (about 1,100 to 1,450 cycles: 370 to 480 poses at one per 3 cycles); the mesh modelled in a spread bind pose and lit in it | 1 rig, 1 to 2 mesh, 2 to 3 animation | a bone-tree compiler to rest matrices (small: `encode_base` already writes them); WEIGHT ASSIGNMENT BY SCRIPT (nearest bones with a falloff, checked by the gate's bone lines) or an export from a rigged Blender file; pose tools on bones (the probe is a start); intersection audit; a rule for painting a mesh that is lit in its bind pose |

The skeletal route is the game's own for every boss since the Tombs of Amascut (20 of the 48 forms; all of the
owner's from 2022 on except Nex). Its real animations are baked poses every 3 or 5 cycles, which is what an export
from any animation program gives; the natural tool is therefore an armature export, not hand-written curves.

**First rebuild target: Tekton 7541** (a demi-boss of the Chambers of Xeric, so inside the owner's standard):
classic, so every proven tool applies today with no new tooling; 4 tiles, ONE model, 2,071 drawn faces (the
largest rebuild so far is about 1,600), 20 colours, 89 labels, its own base 1668, stand 7476, walk 7477, attack
7482, death 7495 from the server tables; rigid faceted forms, the kind the owner graded Strong. Its sheet is built:
`sheets/tekton/sheet-7541.png` and `.json`. Watch for: 250 translucent faces, 99 depth-offset faces, 24 shells.
**Second: Nex 11278** (named by the owner; classic; 3,050 faces, 194 labels, 30 undrawn pivot faces): sheet at
`sheets/nex/sheet-11278.png`.
**First skeletal rebuild: the Maggot King 15742** (named by the owner; one model; the ceiling and the degraded copy
are already through the gate): sheet at `sheets/maggot-king/sheet-15742.png`. It should wait for the weight
assignment tool: without it an author has no way to produce 1,855 weighted vertices on 155 bones.
