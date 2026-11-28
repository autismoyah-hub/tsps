# Interfaces: what they are, and the RECIPES the real ones follow (v2)

Private to this machine. v2 of this file (v1 is `INTERFACES-v1.md`; what changed and why: `CHANGES.md`). v1 described
conventions; three blind rebuilds from it failed where it was silent, ambiguous or wrong. v2 gives every construction
as a RECIPE: pieces, numbers and draw order, each verified by drawing it with iface.py (cache 242, TSPS blend)
against every real instance found in the drawn state of the 239 groups (`state/<group>.jsonl`, the tooling client's
state after each group's onLoad scripts; 998 states), and says over how many real boxes and groups it holds
pixel-exactly, and its exceptions. The six held-out panels of the second test (`recipes/heldout.json`) are excluded
from every count. Tools: `recipes/census_frames.py` (frame recipes, verification), `recipes/census_misc.py`,
`census_titles.py`, `census_colours.py` (the rest); outputs in `recipes/out/`. `ifkit.py` is these recipes as code.

Notation. A box is W x H; a piece is (x, y, w, h) relative to the box's top-left; `W-9` means W minus 9. "Tiled": the
sprite repeats from the piece's own top-left (whole canvases, cut at the piece's edge); not tiled: drawn once,
scaled to the piece. Pieces are listed in DRAW ORDER (each later piece is drawn over the earlier). "Holds for n boxes
in g groups" = the recipe reproduces those real instances with 0 differing pixels.

## 1. What an interface is in the cache (unchanged from v1)

- **Group and components.** Archive 3, one group per interface; a component's id is `group << 16 | file`; its parent
  is a file in the same group, or none (a root, attached by the server). Fixed-mode panes (enum 1129): main modal
  548:41 at (4,4) 512x334; chatbox 548:11 at (0,338) 519x165 with the dialogue modal 162:567 (162:568 in 242) at
  (20,23) 479x96; orbs 548:25 at (516,4) 236x163; HUD pane 548:28 at (4,4) 512x334.
- **Kinds**: layer 0, rectangle 3, text 4, graphic (sprite) 5, model 6, line 9. Size modes per axis: 0 raw; 1
  parent minus raw; 2 raw x parent / 16384. Position modes: 0 from top/left; 1 centred + raw; 2 from bottom/right.
- **Draw order is the TREE, not the order you create components in**: depth first, children in file order, each
  child followed by its own children; every component is clipped to its ancestors. To put B over A, B must come later
  in that traversal (a later sibling, or inside a later sibling).
- **Fields** (the component form of `proof/rebuild-148.json`, defaults `ifref.TEMPLATE`): rectangle `colors[0]`,
  `filled`, `transparency` 0 (opaque) to 255 (invisible); text `font`, `text` (markup `<col=rrggbb>`, `</col>`, `<br>`;
  a `<br>` also ends a colour span), `colors[0]`, `text_shadow`, `text_alignment` [x, y] (0 left/top, 1 centre,
  2 right/bottom), `line_height` (0 = the font's ascent); graphic `sprites[0]`, `sprite_tiling`, `transparency`,
  `flip_h`, `flip_v`; line `line_width`, `colors[0]`.
- **Static against script-driven.** 12% of cache components carry scripts; most frames are built at run time by
  onLoad scripts. TSPS custom interfaces have no script channel: you place every piece as a static component, as the
  recipes below list them.

## 2. Frame families (RECIPES)

A wire sheet names the family in words ([quoted words withheld], [quoted words withheld],
[quoted words withheld], [quoted words withheld], ...). Each recipe below is complete: build exactly these pieces, in this order.

### 2.1 Titled steel border (script 227) — holds for 245 boxes in 220 of 233 groups
In the panel's root (or the frame layer, the same size), in this order:

| # | piece | sprite | x, y, w, h | tiled |
|---|---|---|---|---|
| 1 | backing | 297 TRADEBACKING | 1, 1, W-2, H-2 | yes |
| 2 | title text | b12_full (496), colour from the sheet (orange ff981f in 166 of about 180 titled panels), shadow, centred both ways | 6, 6, W-12, 24 | - |
| 3-6 | corners TL TR BL BR | 310 311 312 313 | (0,0) (W-25,0) (0,H-30) (W-25,H-30), each 25x30 | yes |
| 7 | left edge | 172 | -15, 30, 36, H-60 | yes |
| 8 | right edge | 315 | W-21, 30, 36, H-60 | yes |
| 9 | top edge | 314 | 25, -15, W-50, 36 | yes |
| 10 | bottom edge | 173 | 25, H-21, W-50, 36 | yes |
| 11 | title divider | 2546 | 5, 14, W-10, 26 | yes |
| 12 | close button | 535 (hover 536, a script) | W-29, 6, 26, 23 | yes |

The panel's CONTENT (inner boxes, wells, texts) comes after the frame, as later siblings (in 648: a layer at (10,40)).
Edges and corners never overlap. The title box (6,6,W-12,24) and its style hold for 166 of about 180 titled steel
panels;
the others put a p11 or p12 title in a smaller box: if the sheet's title letters are not b12-sized (5.1), use the
font they match and keep it centred on its letters. Exceptions to the frame: 13 groups (51, 114, 285, 286, 298, 324,
325, 326, 369, 618, 677, 740, 770) place hand-cut 36px edge segments untiled or omit pieces; a wire sheet cannot
show this, it says [quoted words withheld] for them too (undecidable, section 13).

### 2.2 Steel border without title (inner boxes; scripts 392, 2413, 2635, 3737)
Pieces 1 and 3-10 of 2.1 for the box's own W x H, in its own layer (the box's rectangle). Backing (piece 1) only if
the sheet does not say [quoted words withheld]; if the sheet says "with a light well filling it (to its edges)
(mirrored left-right)", the backing is that well at (0, 0, W, H) with those flips instead (148). ORDER: the box's
backing, then its CONTENTS (wells, texts), then its frame pieces 3-10 (148: 4 boxes, all so). Adjacent inner boxes
may overlap by 4-5px; which one is on top is given by the sheet's `drawn` column (later = on top).

### 2.3 Stone border with ornaments (script 703) — holds for 29 boxes in 26 groups
| # | piece | sprite | x, y, w, h | tiled |
|---|---|---|---|---|
| 1 | backing (if any) | 297 | 1, 1, W-2, H-2 | yes |
| 2-5 | corners TL TR BL BR | 824 825 826 827 | (0,0) (W-32,0) (0,H-32) (W-32,H-32), 32x32 | yes |
| 6, 7 | top, bottom edge | 820, 822 | (32,-13,W-64,32), (32,H-20,W-64,32) | yes |
| 8, 9 | left, right edge | 821, 823 | (-13,32,32,H-64), (W-20,32,32,H-64) | yes |
| 10, 11 | title ornaments | 829, 830 | (0,17,32,32), (W-32,17,32,32) | yes |
| 12 | title divider | 828 | 11, 15, W-22, 32 | yes |
| 13 | close | 831 | W-26, 10, 16, 16 | yes |

Title: b12_full, orange, shadow, centred in (6, 6, W-12, 23) (21 of 24 stone titles). Variant (static, 24 groups:
319, 327, 560-581 one series, all 333x217): corners and ornaments NOT tiled, top edge (31,-13,W-63,26), left edge
(-13,31,32,H-63), right edge (W-20,31,23,155), bottom edge (32,H-20,W-64,24), divider (11,14,W-22,26), ornaments at
y 16, close (W-26,9); a wire sheet cannot tell the two apart except by that exact 333x217 size. 7 boxes in 6 groups
(104, 260, 334, 521, 565, 583) follow neither.

**Stone variant 846-849** ("stone-border" boxes with 828/841 edges, 10 groups): corners 846 847 848 849 (32x32,
tiled) at the four corners; 828 at (32,-14,W-64,32) and again at (32,H-20,W-64,32); 841 at (-14,32,32,H-64) and
again at (W-20,32,32,H-64); holds for 12 boxes in 10 groups (7, 17, 335, 701, 702, 705, 721, 723, 726, 762).

### 2.4 Dark bevel box, 9px (scripts 92, 93, 3746, 5861) — holds for 239 boxes in 67 of 72 groups
| # | piece | sprite | x, y, w, h | tiled |
|---|---|---|---|---|
| 1 | backing (if any) | 297 | 1, 1, W-2, H-2 | yes |
| 2-5 | corners TL TR BL BR | 913 914 915 916 | (0,0) (W-9,0) (0,H-9) (W-9,H-9), 9x9 | yes |
| 6 | left edge | 917 | 0, 9, 9, H-18 | yes |
| 7 | top edge | 918 | 9, 0, W-18, 9 | yes |
| 8 | right edge | 919 | W-9, 9, 9, H-18 | yes |
| 9 | bottom edge | 920 | 9, H-9, W-18, 9 | yes |

153 of the 239 have the backing. Variant: backing at (0, 0, W, H) instead (8 boxes in 4 groups: 642, 713, 932,
943); the sheet then says "(to its edges)". This is the commonest BUTTON body (5.4).

### 2.5 Thin border, 32px corners, optional header band (script 991) — holds for 92 boxes in 13 of 16 groups
In the box's own layer. ORDER: first the box's own backing if the sheet says [quoted words withheld] (at
(0, 0, W, H): 297 in 63 boxes, 1040 in 18, none in the rest), then ALL ITS CONTENTS (wells, rectangles, texts), then:

| # | piece | sprite | x, y, w, h | tiled |
|---|---|---|---|---|
| 1 | top edge | 987 | 16, -13, W-32, 32 | yes |
| 2 | bottom edge | 989 | 16, H-20, W-32, 32 | yes |
| 3 | left edge | 988 | -14, 16, 32, H-32 | yes |
| 4 | right edge | 990 | W-20, 16, 32, H-32 | yes |
| 5-8 | corners TL TR BL BR | 991 992 993 994 | (0,0) (W-32,0) (0,H-32) (W-32,H-32), 32x32 | yes |
| 9 | header band | 995 | 0, L-17, W, 32 | yes |
| 10 | band left end | 996 | 0, L-15, 32, 32 | yes |
| 11 | band right end | 997 | W-32, L-15, 32, 32 | yes |

L is the top row of the band's visible 3px line, relative to the box: the sheet gives it ("its line 360x3 at (0,27)
of the box" -> L = 27; 648). The band is set by the content, not by W or H (L-17 = 5, 6 or 10 in the 11 bands seen).
The edges start 16px in and run under the corners; the left edge is at -14 (not -13). Drawing a well or text AFTER
these pieces covers the 1px of the edge strips that overlap it (648's blind rebuild lost 988, 990 that way).
Variant (2 groups, 706 and 744, hand-placed): edges (32,-13,W-64,32), (32,H-20,W-64,32), (-13,32,32,H-64),
(W-20,32,32,H-64); visibly different by 1px on the left. Exceptions: 642, 910, 946 (left edge at -13).

### 2.6 Medium, light and beige bevel boxes, 9px — pieces as 2.4 with these sprites and backings
| family (sheet words) | corners TL TR BL BR | edges L T R B | own backing (1, 1, W-2, H-2) | holds for |
|---|---|---|---|---|
| medium bevel box | 921 922 923 924 | 925 926 927 928 | 897 TRADEBACKING_DARK if present (29 of 52) | 52 boxes, 34 groups (all) |
| light bevel box (script 97) | 929 930 931 932 | 933 934 935 936 | 1040 TRADEBACKING_LIGHT (119 of 119) | 119 boxes, 30 groups (all) |
| beige bevel box | 1554 1555 1556 1557 | 1558 1559 1560 1561 | 1545 TRADEBACKING_BEIGE (36 of 36) | 36 boxes, 9 groups (all) |

All tiled; order backing, corners, edges L T R B. v1 called 929-936 [quoted words withheld] and said nothing of the
others.

### 2.7 Grey and red tile boxes (MISCGRAPHICS_BUTTONTILE) — grey holds for 72 boxes in 32 groups, red 9 in 8
Nine pieces with 6px corners; corners NOT tiled, the rest tiled; order as listed:

| piece | grey | red | x, y, w, h |
|---|---|---|---|
| top-left corner | 1141 | 1150 | 0, 0, 6, 6 |
| top edge | 1142 | 1151 | 6, 0, W-12, 6 |
| top-right corner | 1143 | 1152 | W-6, 0, 6, 6 |
| left edge | 1144 | 1153 | 0, 6, 6, H-12 |
| middle | 1145 | 1154 | 6, 6, W-12, H-12 |
| right edge | 1146 | 1155 | W-6, 6, 6, H-12 |
| bottom-left corner | 1147 | 1156 | 0, H-6, 6, 6 |
| bottom edge | 1148 | 1157 | 6, H-6, W-12, 6 |
| bottom-right corner | 1149 | 1158 | W-6, H-6, 6, 6 |

This is the TAB and small-button body (383's "Exchange" tab, 57x20; commonest sizes 36x36, 31x31, 30x30). Exception:
group 240 draws all nine at transparency 220 (a dimmed state; the sheet would say "translucent").

### 2.8 Two-tone rectangle outlines (script 714, 99 groups; no sprites)
An unfilled rectangle (x, y, w, h) in the outer colour and a second one at (x+1, y+1, w-2, h-2) in the inner colour,
both opaque, outer first. Pairs: bevel dark 0e0e0c outside / bevel light 474745 inside in 121 groups; brown bevel
dark 383023 / brown bevel light 5a5245 in 22; 303030 / 5d5848 in 5. The sheet names the two colours.

## 3. Backings and wells (RECIPES)

- **Sprites.** backing = 297 TRADEBACKING (88x60), light well = 1040 TRADEBACKING_LIGHT, dark well = 897
  TRADEBACKING_DARK, beige well = 1545 TRADEBACKING_BEIGE. Always tiled.
- **Tiling phase.** Tiles start at the well's own top-left (the gate's 1px phase damage fails every panel). Put the
  component exactly at the sheet's rectangle; do not extend it under a frame to [quoted words withheld] tiles.
- **Transparency.** Opaque (0) unless the sheet says [quoted words withheld]: then exactly t. Real values: 0 in
  almost all; 120 (4 groups), 125 (648), 100 (5), 150 (1); 255 means hidden until a script fades it in and is not
  shown on a sheet. v1's [quoted words withheld] was wrong as a default.
- **Flips.** None unless the sheet says [quoted words withheld] (1040 flipped in 148's boxes: 2 of 427 uses).
- **Draw order.** A well inside a box is drawn after the box's own backing and BEFORE the box's frame pieces: so in
  every steel box (148, 551, 944), stone box (154, 260, 319, 327, 521, 560-581) and script-991 thin box (648); only
  the hand-placed thin boxes of 706 and 744 draw their dark wells after the frame. A well that is the box's own
  backing is a piece of the box recipe.
- **Where wells sit** (outside frames; the sheet always gives the rectangle): filling their layer (DARK 13 groups,
  LIGHT 8, TRADEBACKING 11), inset 1 (DARK 24, LIGHT 12), inset 2 (LIGHT 8).

## 4. Rectangles and lines

- Filled boxes, outlines, translucent highlights and bars are kind 3; the sheet gives each one's rectangle, colour
  (palette name, section 12) and transparency. Outline = `filled: false`, 1px inside its rectangle.
- A rectangle wholly covered by a later opaque one changes no pixel: it is not on the sheet and the gate v2 does not
  score it (303's dark green 009000 under 00c800).
- Rule lines (kind 9; the sheet: [quoted words withheld]): a horizontal line has height 0 and width w, a vertical
  one width 0 and height h; commonest 494034 vertical (P20), 000000, ebd5b2 (parchment), 383023, 5d5848, ff981f.
- Rectangles are clipped to their own rectangle and their ancestors; texts, graphics and lines to their parent.

## 5. Text (RECIPES)

### 5.1 Font: from the letters box
The sheet gives each string's letters box (the pixels its glyphs and shadow cover). Draw the string alone in each of
p12_full 495, p11_full 494, b12_full 496, q8_full 497 (with the shadow rule 5.3 and the sheet's line pitch): the
font whose letters have exactly the sheet's width and height is the font (trying p12, p11, b12, q8 in that order,
the first exact match was the real font for all 26 strings of the 8 seen panels). Metrics: p11 ascent 10; p12 and b12 ascent 12; q8 ascent 15. Roles: titles b12 (166 steel, 21 stone);
headings in a header band b12; buttons and tabs p12 or p11 (by letters); labels and values p12 or p11; dialogue q8.

### 5.2 Colour: from the sheet
The sheet names every string's colour by its palette name (section 12) and every emphasised part's colour in
brackets ("Total: (light orange)"). No rule decides these: white labels (1,594) outnumber orange ones (1,272), and an
emphasised part is white (22 groups), red (20), grey (14), green 0dc10d (12) or light orange ffb83f (13) on an
orange label. Write an emphasised part as `<col=rrggbb>part</col>`; if it runs over a line break, close it before
the `<br>` and open it again after (a `<br>` ends a colour span).

### 5.3 Shadow
1px black down-right on light text; none on dark text (black, the browns 403020/46320a/4c3d26, maroon 800000, blue
0000ff on parchment or beige). 99% of orange, white and red text has it; held without exception in the 8 seen panels.

### 5.4 Boxes and alignment (where to put the text component)
- Title of a steel frame: (6, 6, W-12, 24), centred both ways (2.1); stone (6, 6, W-12, 23).
- Button or tab label: the text box IS the button's box (0, 0, W, H), centred both ways (dark bevel 109 of 128
  labels, medium bevel 45, grey tile 45 of 49, red tile 6 of 6; 383's tab).
- Header band headings and table cells: 5.5.
- Anything else: any box that holds the letters, centred both ways, then move it until the letters sit on the sheet's
  letters box (measure your own drawing). Text boxes do not change pixels, only where the letters land; but a box
  narrower than a line wraps it, so give each line explicitly with `<br>` as the sheet lists the lines.
- Multi-line texts: the sheet gives the line pitch (baseline to baseline) and how the lines align ("left", "centre",
  "right"): set `line_height` to the pitch (0 if it equals the font's ascent) and `text_alignment[0]` to match.
  Real pitches: p12 at ascent (43 groups), 15 (10), 14 (8), 16 (5); p11 at ascent (10); q8 at ascent (27); 648's
  table cells 25. Body text in boxes is mostly left-top (133) or centred (91).

### 5.5 Tables and column division (thin box with a header band; 648, 4 groups)
Columns are equal; their width is the distance between neighbouring heading centres (648: 120 = 360/3) and the first
column starts at the box's left. A heading sits in (column x, 0, column width, L+3) and a cell in (column x, L+3,
column width, H-L-4), both centred both ways (b12 orange headings: 12 of 12 in 4 groups). A table row in a
two-tone box uses the same idea with rule lines between columns (section 4) where the sheet shows them.

## 6. Buttons, tabs and controls

- A sheet's [quoted words withheld] is the box recipe of family X (2.4, 2.6, 2.7) with its label filling the box (5.4).
  Label styles found (uses, groups): dark bevel p12 orange shadow centred (70, 34); light bevel p11 dark red bb0000
  centred-top (42, 6) or p12 orange (32, 15); grey tile p12 orange (28, 12) or p11 orange (16, 13); medium bevel p12
  orange (20, 12); beige bevel b12 or p12 dark brown 403020 without shadow (19+11, 7). v1's "p12_full ffffff" on
  buttons was right for 3746's buttons only (10 uses in 6 groups). Colour: from the sheet (5.2).
- Close buttons: 535 (26x23) at (W-29, 6) on steel frames (182 of 188); 831 (16x16) at (W-26, 9 or 10) on stone;
  539, 541, 537 (26x23) elsewhere (104, 81, 49 uses; where, the sheet shows).
- Hover is a script (`onMouseOver [273, self, 75]`): not static, not on a sheet.
- Checkboxes 8378 (off) / 8380 (on), 17x17 (15 groups), and skill icons 197-221 (25x25): icons (13: undecidable).

## 7. Bars and HUDs (RECIPES)

### 7.1 The boss health bar HPBAR_HUD 303, as TSPS's 242 scripts draw it (the live state; holds for 303)
A fill-parent root (size mode 1,1) in the HUD pane; a 150x42 box layer at (2, 25) (the visible box is its 148x40
outline at (3, 26) of the pane, which the sheet gives). Inside the box layer, in order:
1. outline rectangle frame black 191511, unfilled, (1, 1, 148, 40);
2. dark well 897 tiled (2, 2, 146, 38);
3. name layer (0, 0, 150, 19) with the name: p11_full, orange, shadow, centred both ways in (8, 2, 134, 19);
4. bar layer (3, 18, 144, 21): outline 191511 unfilled (1, 1, 142, 19); an inner layer (2, 2, 140, 17) holding the
   red base hud red c80000 filled (0, -2, 140, 21), then the dark green hud dark green 009000 (0, -2, F, 21) and the
   green hud green 00c800 (0, -2, F, 21), F = 140 x current / maximum rounded down (34/60 -> 79);
5. the value "34 / 60" in the bar layer: p11_full, off-white eaeaea, shadow, centred both ways in (0, 1, 144, 21).
The 009000 rectangle is wholly covered by the 00c800 one (invisible; not scored by the gate v2). In the 239 cache 303
is 300x42 centred at the top with red cc0000 under green 00cc00 (health red / health green). The value has the
shadow (v1 was silent). `ifkit.Kit.hpbar_hud` builds this.

### 7.2 Other bars
A bar is an opaque filled base rectangle with filled parts drawn over it from its left end; the sheet gives the
base's and each part's colour and the parts' sizes; a rim, where present, is a separate outline box (section 4).
Bars seen: black (200) under 005f00 full (2 groups), 2d2b00 under dbd300 (NIGHTMARE_TOTEMS), ff0000 under 00ff00,
cc0000 under 00cc00. Heights 9-21px.

### 7.3 Overlays and orbs (v1, unchanged)
Overlays are fill-parent roots with content anchored by position modes; orbs (160) are 57x34 frames (1071) with a
26x26 icon at (27,4) and a p11 number in (4,16,23,13), yellow ffff00 with shadow, stepping down 34px.

## 8. Item grids (v1 rule, measured with the tooling client)

An item grid is an empty layer the server fills (`interface_inv_init`, script 149): one 36x32 slot per cell, the first
flush top-left, the last flush bottom-right: pitch = (W-36)/(columns-1), (H-32)/(rows-1) (190x261 4x7 -> 51x38;
143x75 3x2 -> 53x43). Slot backs, where shown, are sprite 170 (36x36, 25 groups). Items and models are not drawn by
iface.py and not scored; build the empty layer at the sheet's grid rectangle.

## 9. Scrollbars — holds for 101 of 104 scrollbars in 92 groups
A 16 x H layer: track 792 (0, 16, 16, H-32), thumb 790 (0, t, 16, h), thumb top cap 789 (0, t, 16, 5), thumb bottom
cap 791 (0, t+h-5, 16, 5), top arrow 773 (0, 0, 16, 16), bottom arrow 788 (0, H-16, 16, 16), in that order, none
tiled. t and h follow the scroll state (on a sheet: the thumb's rectangle, if shown). 793-798 is a second, rarer set
(7 groups). v1 named 316 SCROLLBAR, which is used in one group only.

## 10. Dialogue (CHAT_LEFT 231 / CHAT_RIGHT 217, 239 state)

In a 481x110 layer inside the 506x129 chat root: the head model 32x32 at (35, 43) (left, 231) or (415, 43) (right,
217); in a 479x96 layer at (1, 7): name (96, 0, 380, 17) q8_full maroon 800000 no shadow centred-top; body
(96, 16, 380, 67) q8 black 000000 no shadow, line height 16, centred-top (231) or centred-middle (217); "Click here to
continue" (96, 80, 380, 17) q8 blue 0000ff. For 217 the text column starts at x 5 instead of 96. The parchment is the
chatbox's (162), not the dialogue's.

## 11. How the client draws (what the harness reproduces; v1 section 9)

Depth first in tree order, clipped to ancestors; sprites scaled with 16.16 steps unless tiled; transparency t blends
with alpha (256 - t)/256 in the game client, (255 - t)/255 in TSPS (`iface.TSPS_BLEND`, what the gate uses). Text
wraps at spaces unless the box is too short for two lines; y-centring puts the first baseline at top + ascent +
(height - ascent - descent - (lines-1) x line height)/2; the shadow is the glyph 1px down-right in black.

## 12. The named palette (what a sheet's colour names mean)

Every colour used by visible text, colour spans, rectangles or lines in at least 2 drawn groups (held-out panels
excluded), plus the three colours the 242 HPBAR_HUD scripts set (`ifpalette.py`). A sheet colour outside this list
is "unlisted": it cannot be recovered (section 13).

| name | rgb | name | rgb | name | rgb |
|---|---|---|---|---|---|
| orange | ff981f | black | 000000 | white | ffffff |
| bevel dark | 0e0e0c | bevel light | 474745 | red | ff0000 |
| grey | 9f9f9f | yellow | ffff00 | brown bevel dark | 383023 |
| mid grey | 808080 | brown bevel light | 5a5245 | deep orange | ff7d1f |
| light orange | ffb83f | light grey | bfbfbf | green | 0dc10d |
| olive grey | 5d5848 | silver | aaaaaa | parchment | ebd5b2 |
| dark umber | 30201c | dim grey | 7f7f7f | gold | ffcc33 |
| pale silver | cccccc | near black brown | 170801 | dark brown | 403020 |
| maroon | 800000 | khaki brown | 726451 | bright green | 00ff00 |
| table rule brown | 2e2b23 | tangerine | ff9933 | brown | 46320a |
| amber | ffa82f | dark rule | 28251e | dark red | bb0000 |
| lavender | ccccff | off-white | eaeaea | health red | cc0000 |
| health green | 00cc00 | frame black | 191511 | blue | 0000ff |
| pale grey | cfcfcf | cream | f5e9d5 | hud red | c80000 |
| hud dark green | 009000 | hud green | 00c800 | P1 | 202020 |
| P2 | 303030 | P3 | 008000 | P4 | 999999 |
| P5 | 4c3d26 | P6 | 888800 | P7 | 2a1e06 |
| P8 | ff891f | P9 | ffaa44 | P10 | 404040 |
| P11 | 505050 | P12 | 333333 | P13 | 282315 |
| P14 | 37242b | P15 | 333300 | P16 | fcfce7 |
| P17 | 00c000 | P18 | c00000 | P19 | 2f2a25 |
| P20 | 494034 | P21 | ff991f | P22 | ff6600 |
| P23 | 990000 | P24 | 585040 | P25 | 666666 |
| P26 | faa500 | P27 | dfdfdf | P28 | 29ffbd |
| P29 | ffc920 | P30 | 9fd6ff | P31 | ff9720 |
| P32 | ff9800 | P33 | 140b02 | P34 | f7f0df |
| P35 | eb9a2f | P36 | 321e25 | P37 | 2d0000 |
| P38 | b70000 | P39 | 03171b | P40 | 1a88a2 |
| P41 | 140d20 | P42 | 563a8c | P43 | 5a7b86 |
| P44 | ff9820 | P45 | 7f7fff | P46 | 339900 |
| P47 | cc6600 | P48 | 005f00 | P49 | 11d311 |
| P50 | ac1414 | P51 | bf751d | P52 | 525252 |
| P53 | 676725 | P54 | ee1111 | P55 | ff9900 |
| P56 | ffa53d | P57 | ff3333 | P58 | 302c27 |
| P59 | 0000cc | P60 | 1d7878 | P61 | c8c8c8 |
| P62 | 191919 | P63 | 0a2133 | P64 | 0e2537 |
| P65 | 12293b | P66 | 162d3f | P67 | 1a3143 |
| P68 | 1e3547 | P69 | 22394b | P70 | 263d4f |
| P71 | 2a4153 | P72 | 2e4557 | P73 | 32495b |
| P74 | 364d5f | P75 | f5bf18 | P76 | 8080ff |
| P77 | 888888 | P78 | 9e9e9e | P79 | 6b6b69 |
| P80 | d299ff | P81 | 322b19 | P82 | 281515 |
| P83 | 99e7ff |  |  |  |  |

## 13. Decision rules, and what a wire sheet cannot decide

How to choose from what a v2 rules sheet shows:
1. Family: from the region words (section 2). Placement: the region's rectangle is the box (for the titled frame: the
   whole panel). Its backing: the family default unless the words say otherwise ([quoted words withheld], "with a
   light well filling it (to its edges) (mirrored left-right)").
2. Nesting and order: put each box in its own layer at its rectangle; regions inside a box go in that layer; build
   siblings in the sheet's `drawn` order; inside a steel inner box or thin box draw the contents before the frame
   pieces; inside a button draw the box, then its label.
3. Wells: sprite by word, rectangle from the sheet, transparency and mirroring only as the words say.
4. Texts: font by letters size (5.1), colour and emphasis by palette name (5.2), shadow by 5.3, box by 5.4/5.5, then
   shift until the letters land exactly.
5. Bars and rectangles: rectangles and colours from the sheet (palette names), base first, parts after.

Undecidable from a wire sheet (the sheet does not say, and no recipe can):
- **Icons**: a sheet names pictures only by letter (sprite ids would leak the decode); icons stay unmatched (INFO).
- **Models and item icons**: not drawn by iface.py, not scored.
- **Unlisted colours** (used by one group only) and fonts outside the four main ones.
- **The old static frame variants** (2.1 exceptions, 2.3 variant, 2.5 variant): same words, slightly different
  pixels; only the exact sizes hint at them.
- **Script state**: hover (transparency 75 by script 273), hidden parts (transparency 255 until faded in), scroll
  thumbs not shown on the sheet, server-filled texts that are empty in the captured state.
- **Invisible components** (wholly covered): not on the sheet; the gate v2 ignores them.

## 14. What a new interface must match (the short version)

1. Panels: the titled steel border 2.1 exactly as listed (pieces, order); content from (10, 40) with 10px margins in
   38 of 73 measured panels (within 5px in 47); content after the frame.
2. Inner boxes: steel 2.2, thin 2.5 (with a header band for tables), dark/medium/light bevel 2.4/2.6, two-tone 2.8;
   contents before their frame pieces.
3. Buttons and tabs: a bevel box or grey tile box with its label filling it, p12 (or p11) orange, shadow, centred.
4. Text: b12 titles and headings, p12/p11 labels, values and buttons, q8 dialogue; orange ff981f by default; shadow on
   light text; emphasis by `<col>` spans of palette colours.
5. Wells: opaque, unflipped, tiled from their own corner, unless you mean otherwise.
6. Boss HUD: 7.1 (`ifkit.Kit.hpbar_hud`); overlays anchor to an edge with position modes.
