# Interfaces: what they are, and the rules the real ones follow

Private to this machine. Measured from all 969 interface groups of the revision-239 cache (26,564 components;
the 242 cache TSPS runs has 992 groups, 942 of the 969 byte-identical). "Drawn" means: as the client draws the
group after its own onLoad scripts have run (the tooling client's script engine, `state/<group>.jsonl`), which is
what a player sees before the server fills in names, numbers and items. Numbers come from `harness/measure.json`
(`measure.py`) unless a file is named. Sprite ids are cache sprite groups; names in capitals are RuneLite's (TSPS
`client/runelite/api/gameval/SpriteID.ts`), the others have no public name and are given by id.

## 1. What an interface is in the cache

- **Group and components.** Archive 3, one group per interface (ids 0 to 1000 in 239), one file per component;
  a component's id is `group << 16 | file`. Its parent is a file number in the same group, or none (a root). A
  root is attached by the server: opened as the top level (the game frame: 548 fixed, 161 resizable) or mounted
  into a layer of another group (`if_opensub`). Fixed-mode panes, from enum 1129: main modal 548:41 at (4,4)
  512x334; chatbox 548:11 at (0,338) 519x165 with the dialogue modal 162:567 (162:568 in 242) at (20,23) 479x96
  inside it; orbs 548:25 at (516,4) 236x163; the HUD pane 548:28 at (4,4) 512x334.
- **Two formats.** IF3 (first byte 255; 92% of components, all groups above id 599) and the older IF1 (2,096
  components in 62 groups). Field order is in `ifcache.py decode_widget` (a port of the tooling client's
  `widget.rs`); 26,560 of 26,564 decode with no bytes left over (four model components keep 2).
- **Kinds** (IF3 / IF1 counts): layer 9,467 / 247; text 4,984 / 420; model 4,817 / 539; graphic (sprite) 3,328 /
  719; rectangle 1,695 / 163; line 177 / 0; IF1 tooltip 8. No IF1 inventory kinds (2, 7) remain in 239 or 242.
- **Size and position modes** (per axis; P = parent size, raw = stored value):
  size 0 = raw; 1 = P - raw; 2 = raw x P / 16384; 4 = aspect of the other axis. Position 0 = raw from the
  top/left; 1 = centred + raw; 2 = from the bottom/right (P - size - raw); 3 = raw x P / 16384; 4 = that plus
  centring; 5 = from the far edge by a fraction. A layer's children are laid out in its scroll size if it has one.
  Children are drawn in file order, each followed by its own children; a child is clipped to its ancestors.
- **Fields by kind.** Layer: scroll size, no-click-through. Rectangle: colour (24-bit RGB), filled, transparency
  0-255 (0 opaque); scripts can add a vertical gradient (fill mode 1) or an alpha gradient (2). Text: font id,
  text with markup (`<col=rrggbb>`, `<br>`, `<shad>`, `<str>`, `<u>`, `<img=n>`), line height (0 = the font's
  ascent), alignment x (0 left, 1 centre, 2 right) and y (0 top, 1 centre, 2 bottom, 3 spread), shadow (1px
  black, down-right), colour. Graphic: sprite id, angle, tiling, transparency, outline (1 black; 2 adds white),
  shadow colour, flips. Model: model id, offset, three angles, zoom, sequence, orthographic. Line: width, colour,
  direction. All IF3 kinds: ops (right-click options), drag, 18 listener slots (onLoad, onMouseOver, onOp,
  onTimer, onVarTransmit, ...) each `[script id, arguments...]`, and the varps/inventories/stats that trigger them.
- **Sprites** (archive 8). A group is a sheet: canvas width and height, a palette of up to 255 RGB colours (index 0
  transparent, colour 0 stored as 1), and one or more frames, each an offset and size inside the canvas with one
  palette index per pixel (row- or column-major); an alpha plane may follow and the client ignores it. A widget
  draws frame 0 of its sprite, scaled to its size (tiled instead when tiling is set: whole canvases from the
  top-left, cut at the widget's edge). Border sprites carry their offset: an edge sprite 36x36 holds only a 6px
  strip 15px in, so it is placed 15px outside the box it edges.
- **Fonts** (archive 13 metrics + the sprite group of the same id: 256 glyph frames). In use: p11_full (494),
  p12_full (495), b12_full (496), q8_full (497), quill_oblique_large (645), quill_caps_large (646), the lunar
  alphabets (647/648), five verdana sizes (1442-1447) and unnamed 764, 819, 5607/5608, 5675/5676, 6315. Metrics:
  p11 ascent 10, glyphs 10 up / 2 down; p12 and b12 ascent 12, 12 up / 4 down; q8 ascent 15, 15 up / 5 down.
  Digits are proportional (p12 6-8px wide, b12 7-9px); space is 3px (p11, p12), 4px (b12), 7px (q8). None of the
  four main fonts has kerning in 239.
- **Static against script-driven.** 3,150 of 26,564 components (12%) carry a listener or trigger. Opening every
  group and running its onLoad scripts creates 56,583 components (29,377 visible), more than the cache holds:
  frames, borders, buttons and lists are mostly built by scripts. The frame of a typical panel is not in its
  definition at all; the definition holds an empty layer whose onLoad is `[227, self, "Title"]`. What scripts
  cannot invent comes from the server: names, numbers, items (`interface_inv_init`, script 149), varps.

## 2. Frame styles (the borders panels are built from)

Each is drawn by one family of scripts into the layer that calls it; W x H is that layer. Positions below are
relative to its top-left; "tiled" means the sprite repeats across the given span.

1. **Steel border with title and close (script 227; 126 groups, by far the commonest panel).**
   background TRADEBACKING 297 (88x60) tiled at (1,1) size (W-2, H-2); corners 310/311/312/313 (25x30) at the four
   corners; left edge 172 at (-15,30) size (36, H-60), right edge 315 at (W-21,30), top edge 314 at (25,-15) size
   (W-50, 36), bottom edge 173 at (25, H-21): all edges show a 6px strip; title divider 2546 at (5,14) size
   (W-10, 26), its 6px strip at y 29-34; title text b12_full, colour ff981f, shadow, centred both ways, in the box
   (6,6) size (W-12, 24); close button 535 (26x23, drawn 21x21 at offset 1,1; hover 536) at (W-29, 6).
   Content starts at (10, 40) and keeps 10px from the other three sides: exactly so in 38 of the 73 panels whose
   content could be measured, within 5px in 47; most others put content flush (0,40,0,0) or use the whole frame
   for one list (7,7,10,10). These margins are a convention of the content, not of script 227.
   Panel sizes: width 188-745, median 484 (quartiles 380 and 500); height 133-503, median 308 (280 and 325).
   Commonest: 484x304 (9), 480x314 (8), 512x334 (6, the whole modal), 488x300 (5), 342x228 (5), 500x324 (4).
2. **Steel border without title** (scripts 392: 24 groups, 2413: 20 groups, 2635, 3737): the same corners and
   edges, with (2413) or without (392) the background. Used for inner boxes and side panes inside a steel panel.
3. **Stone border with ornaments** (script 703: 23 groups; 712: 13): background 297 at (1,1); corners 824-827
   (32x32); horizontal edges 820/822 (7px strip, at y -13 and H-20), vertical 821/823 (at x -13 and W-20); title
   ornaments 829/830 at (0,17) and (W-32,17); title divider 828 at (11,15) width W-22; title b12_full ff981f shadow
   centred in (6,6, W-12, 23); close 831 (16x16) at (W-26,10). Variant 712 uses corners 846-849 and edges 828/841.
4. **Thin bevel box, 9px corners** (script 92: 11 groups, 93, 3746, 5861): corners 913-916 (9x9), edges 917-920
   (3px strips); with background 297 at (1,1) when filled. This is the standard *button* body: buttons are
   W x H boxes of this kind with a centred label (p12_full ffffff shadow on 3746's buttons). The lighter variant
   929-936 with TRADEBACKING_LIGHT 1040 is script 97.
5. **Thin border, 32px corners** (script 991: 8 groups, e.g. 648): corners 991-994, edges 987-990 (3px strips at
   13/17 offset), a title band 995 with ends 996/997.
6. **Two-tone rectangle bevels** (no sprites): script 714 (99 groups, the most used of all onLoad scripts) draws two
   1px outlines, 0e0e0c and 474745; script 1191 the pair 383023 and 5a5245. These separate rows and boxes inside
   panels.
7. **Backgrounds**: TRADEBACKING 297 (tiled, 1,103 uses in 313 groups), TRADEBACKING_LIGHT 1040 (438 in 85; inner
   wells, often at transparency 125-150), TRADEBACKING_DARK 897 (292 in 71; HUD boxes).

## 3. Text

- **Fonts in use** (drawn text): p12_full 5,677; p11_full 1,668; b12_full 1,221; q8_full 545; quill_oblique_large 55;
  everything else under 40. Verdana appears in two components only.
- **Roles.** Titles: b12_full, ff981f, shadow, centred, 24px box (187 frame titles; 70 more in ff9933). Labels
  (short text): p12_full in ffffff (1,594), ff981f (1,272) or ff0000 (963); p11_full in ff0000 (421) and ff981f (264).
  Numbers: p12_full ff981f (213) or ffffff (150), p11_full ffffff (81) or ffff00 (53). Body text (over 40
  characters): p12_full ff981f (134), bfbfbf (71) or 9f9f9f (53). Dialogue and books: q8_full, no shadow, black or
  dark brown 403020/46320a on parchment.
- **Colours** (drawn text, count): ff981f orange 2,655; ffffff 2,016; ff0000 1,425; 9f9f9f 423; 000000 356;
  ffcc33 150; ebd5b2 128; ffff00 124; bfbfbf 121; cfcfcf 115; ff9933 103; ccccff 85; bb0000 84; f5e9d5 82.
  Dialogue: name 800000, body 000000, "Click here to continue" 0000ff (CHAT_LEFT 231). Orb numbers ffff00 by
  default (scripts recolour them; full is 00ff00). Boss bar name ff981f, value eaeaea.
- **Shadow** is the rule on dark backgrounds: 99% of ff981f, ffffff and ff0000 text has it. Black and dark text on
  parchment never does (245 of 356 black texts have none).
- **Layout.** Line height 0 (the font's ascent) for 85%; otherwise p11 11, p12 13-16. Alignment: left+middle 3,927,
  centre+middle 2,788, left+top 1,179, centre+top 950. Text boxes: p12 13 or 15 tall, p11 16, b12 24 (titles) or
  14, q8 17.

## 4. Rectangles and lines

Filled: 000000 1,003; ffffff 906 (usually translucent highlights); 9e9e9e 300; 555555 224; bfbfbf 146; the bevel
pair 474745/0e0e0c ~100 each. Outlines: 000000 1,507; 0e0e0c 437; 474745 364. Transparency on drawn
rectangles: 0 for 3,882, 255 (invisible until a script fades it in) 1,057, then 225, 200, 150, 245, 50.
Lines (177) are table rules: 2e2b23 in the bank, ff981f for small marks.

## 5. Buttons and controls

- Operable components (IF3, an op or onOp): layers 1,767 (invisible hit boxes over drawn parts), text 798,
  graphics 748, rectangles 329. Fixed sizes most used: 26x23 (221: close buttons), 32x32 (218), 36x36 (178: slot
  backs, sprite 170), 24x24 (110), 13x12 (74: sprite 1341), 16x16 (68), 40x25 (63), 64x35 (48).
- Close buttons: 535/536 on steel frames (26x23), 831 on stone frames (16x16), 539/541/537 elsewhere.
- Hover is a script, not a field: `onMouseOver [273, self, 75] / onMouseLeave [273, self, 0]` on the Lunar chest;
  script 273 is a single `if_settrans` (opcode 2103), so hovering sets transparency 75 and leaving resets 0. IF1
  components switch to colour index 2 instead.
- Scrollbars: sprite group 316 SCROLLBAR (16x16 arrows), track 792 and thumb 789-791 (16x5 pieces), 16px wide.

## 6. Item containers

- An item grid is an empty layer the server fills with `interface_inv_init` (script 149: layer, inventory, columns,
  rows, ..., up to five op names). It creates one 36x32 graphic per slot and spreads them over the layer: the first
  cell flush with the top-left, the last flush with the bottom-right. Measured with the tooling client: the 190x261
  inventory with 4x7 gives a pitch of 51x38 (gaps 15x6); the Lunar chest's 143x75 layer with 3x2 gives 53x43.
  So pitch = (W - 36) / (columns - 1), (H - 32) / (rows - 1), whole pixels.
- Slot backs where shown are sprite 170 (36x36). Quantities are drawn by the item icon itself (yellow, p11-style
  digits, top-left).
- Reward chests are steel panels (227) with the grid layer in the middle and 26x23 or 36x36 buttons in the bottom
  corners (Lunar chest 868: 188x190; Barrows 155: 230x200; ToB/ToA chests 23/771: 240x220; CoX 539: 260x220).

## 7. Boss and status overlays

- The generic boss health bar is HPBAR_HUD 303, mounted at login in the HUD pane (TSPS `ClientProtocol.ts:2803`).
  In 239 it is a 300x42 box centred at the top (y 2); in 242 (TSPS, seen live) its scripts size it 150x42 at
  (2,25), top-left. Layers: frame rectangle 191511 (1px), background TRADEBACKING_DARK 897 inset 2; name in
  p11_full ff981f, shadow, centred in the top 19px; bar 140x17 (242; 290x17 in 239) inset 5, drawn as red cc0000
  (c80000 in 242's script colours) under green 009500/00cc00, whose width is current/maximum; value "34 / 60" in
  p11_full eaeaea centred over the bar. Everything starts at transparency 255 and a script fades it in.
- Older boss HUDs follow the same plan with their own sizes: INFERNO_HP_HUD 596 (300 wide, bar 296x16 inset 2,
  cc0000 under 00cc00, name p11 ff981f "TzKal-Zuk", value p11 black); NIGHTMARE_TOTEMS 413 (139x64 box, four 61x9
  bars 2d2b00 under dbd300, labels p11 ff981f); TOB_HUD 28 (120x106 TRADEBACKING panel at transparency 100,
  b12 ff981f title, p12 ff981f body, 34x34 portrait backs sprite 2138).
- Placement: overlays are fill-parent roots (size mode 1,1) with content anchored by position modes: centred top
  (303 in 239), top-left (303 in 242, 413), or the right edge (orbs). 81 groups are named HUD, OVERLAY, TIMER or
  similar; their bars are filled rectangles 9-21px tall.
- Orbs (160): each orb is a 57x34 frame (1071) holding a 26x26 icon at (27,4) and a p11_full number in a 23x13 box
  at (4,16), ffff00 shadow until a script recolours it; orbs step down 34px (y 37, 71, 103, 128 in fixed mode).

## 8. Older against newer

Grouping by id (roughly the order groups were added): IF1 is 10% of the components of groups 0-99, 5-21% of
those up to 399, 9% up to 599 and none from 600 on. Static sprite frames (the steel corners placed by hand in the definition) appear
only below id 400 (52 components) and in 4 components above 900; from 400 on every frame is script-built. The
share of drawn components that scripts create is 27-76% below 700 and 57-87% above. Font use moves from q8/p12
(ids 0-299) to p12 and b12 (400 on), with b12 the title font throughout. Newer content keeps the old look: the
steel border, its title style and its colours are the same in groups 2 and 938.

## 9. How the client draws (what the harness reproduces)

- Draw order: depth first in file order; a component is clipped to its parent's rectangle (text, graphics and lines
  to the parent's, rectangles to their own); hidden components hide their subtree.
- Sprites: scaled with 16.16 fixed-point steps unless tiled; transparency t blends with alpha (256 - t)/256 in the
  game client's software path. **TSPS differs:** it blends translucent fills and sprites in WebGL with alpha
  (255 - t)/255, rounded per channel (`renderWidgetTree.ts:3037, 3051`), off by 1 level where t is not 0;
  `iface.TSPS_BLEND` reproduces it exactly. TSPS also draws a translucent outline rectangle as four overlapping bars,
  so its four corner pixels blend twice (`renderWidgetTree.ts:3063-3066`), and its scrollbar thumb differs in 4 to 6
  pixels.
- Text: wrapped at spaces and hyphens unless the box is shorter both than ascent + descent + one line height and
  than two line heights; y alignment 1 puts the first baseline at top + ascent + (height - ascent - descent -
  (lines - 1) x line height) / 2; shadow is the same glyph 1px down-right in black.
- Harness record (`iface.py`): against the tooling client, 0 differing pixels outside model and item rectangles in all
  969 groups, both as defined and after their onLoad scripts (566 and 503 groups entirely exact); against the TSPS
  client, 28 of 54 captured panels exact over every comparable pixel; in the rest the differing pixels sit, by
  component, on items, models, numbers and texts the server filled in (not in the default state), plus the
  scrollbar and outline-corner deviations above; the boss HUD and the dialogue exact given TSPS's live values.

## 10. What a new interface must match (the short version)

1. Build panels on the steel border (section 2.1) unless the brief says otherwise: content from (10,40), 10px
   margins, title b12_full ff981f shadow centred in (6,6,W-12,24), close 535 at (W-29,6).
2. Inner boxes: steel border without title (2.2) or the 9px bevel (2.4) or the two-tone rectangles 0e0e0c/474745.
3. Text: p12_full for labels and numbers, b12_full for titles and headings, p11_full where space is tight;
   ff981f for headings and labels, ffffff for values, ff0000 for warnings, 9f9f9f for inactive; shadow on.
4. Grids: 36x32 slots spread by the inv_init rule; buttons are 9px bevel boxes with a p12 label.
5. Boss HUD: the 303 plan (dark box, name above, red-under-green bar, value centred), or 303 itself.
6. Sizes fit the 512x334 modal; overlays anchor to an edge with position modes, never absolute in a frame.
