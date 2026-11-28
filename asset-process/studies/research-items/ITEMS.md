# Anatomy of a revision 239 equipment item

A Rune longsword has three distinct views of the same design. Its inventory sprite is a camera shot of the drop mesh. A ground pile uses that mesh in world space. Equipping the sword selects a different model and places it in the figure’s weapon slot. A helm adds a fourth representation for dialogue. The cache stores the mesh IDs and the rules for selecting and drawing them in the object definition.

## What the definition controls

| Use | What to supply | What the client does |
|---|---|---|
| Inventory | Model ID, scale in 128ths, zoom, xan/yan/zan in 2,048 units per turn, signed x/y offsets, ambient and contrast | Resolves quantity variants, scales, recolours and retextures the model. Renders a sprite on demand. The six view values belong to the icon camera. |
| Ground | The same inventory model and materials | Selects the quantity variant, lights the prepared mesh, places its origin at tile centre and aligns it to the ground normal when there is no support. A pile can select multiple objects. Authored grip geometry, soles and the empty interior of a helm must read from above. |
| Worn | Up to three model IDs per sex and one vertical offset per sex | Selects by appearance gender, combines the pieces, adds the offset to cache y (positive is down), applies item materials, then combines with the player kits and applies player colours. Inventory scale and icon zoom do not fit armour to a body. |
| Slots | Primary position and up to two positions to clear | A cuirass with its own sleeves uses 4,6,-1. The open barbute uses 0,8,-1, retaining the jaw. A two-handed halberd uses 3,5,-1. Legs, hands and feet occupy 7, 9 and 10. A blank slot removes a whole kit. |
| Dialogue | Up to two head model IDs per sex | Loads those IDs and the surviving kits' head models, using dialogue coordinates and expression labels. The head path skips the worn offset. An open-front helm that removes hair generally has to supply the face as well. |
| Interaction | Name, examine, five inventory actions, ground actions, price, stackability, membership, trade flags and any links, parameters or variants | Take and Drop have decoder defaults. An equippable item needs an equip action. Combat animations and equipment statistics depend on server gameplay data. |

Recolour entries are ordered pairs of packed HSL values. Retexture pairs replace texture IDs. Apply these before lighting. Of 17,311 named plain definitions, 44.3% use recolours, 88.5% have a nonzero icon offset and 8.2% have a nonzero third icon angle. The commonest pairs of worn offsets are (0,0) in 4,415 wearables and (0,6) in 1,485. A distinct female model can already contain the full displacement. Adding it again in the definition would lower the item twice.

## How often the geometry carries over

These percentages count item definitions, so colour variants can contribute multiple votes. Each row uses the readable, comparable models for that test. The two weapon denominators differ by one. “Different topology” includes re-indexed exports and multi-part rebuilds. It does not establish that an artist sculpted the design again. Equality of indexed triangles is a stricter test than visual similarity.

| Inventory against male worn | n | Same ID | Translation only | Re-posed | Reshaped, same topology | Different topology |
|---|---:|---:|---:|---:|---:|---:|
| Weapon | 1,832 | 0.4% | 26.1% | 3.9% | 1.5% | 68.1% |
| Head | 1,049 | 0.5% | 7.8% | 5.5% | 2.6% | 83.6% |
| Body | 619 | 0% | 0% | 1.1% | 0% | 98.9% |
| Legs | 570 | 0% | 0% | 8.1% | 0.5% | 91.4% |
| Hands | 219 | 0% | 0% | 0% | 8.2% | 91.8% |
| Feet | 237 | 0% | 5.1% | 0.4% | 0.4% | 94.1% |
| Shield | 441 | 0.5% | 0.5% | 10.2% | 2.0% | 86.8% |
| Cape | 374 | 0.5% | 0% | 1.3% | 5.6% | 92.5% |

No additional indexed, identical copies rounded above 0.0% in these rows. The re-pose test fits translation, orthogonal transform and uniform scale, accepting a maximum vertex residual of 2.5 units. Example patterns include bodies and legs laid on their backs and helms and boots moved upright to the ground origin. The median inventory-to-worn face ratio among different-topology models is 0.98 for weapons, 0.83 for bodies and 0.84 for legs. A worn model can therefore be a useful starting mesh even when its original camera framing and construction are unsuitable.

| Female against male | n | Same IDs, same offset | Same IDs, other offset | Moved copy | Axis scale | Other same-topology reshape | Different topology |
|---|---:|---:|---:|---:|---:|---:|---:|
| Weapon | 1,833 | 3.0% | 58.2% | 14.1% | 7.3% | 14.7% | 2.7% |
| Head | 1,049 | 0.7% | 0.8% | 9.4% | 7.9% | 15.4% | 65.8% |
| Body | 619 | 0% | 0% | 0% | 0% | 1.3% | 98.7% |
| Legs | 570 | 0% | 0% | 0% | 0% | 17.2% | 82.8% |
| Hands | 219 | 0% | 0% | 0% | 3.2% | 4.1% | 92.7% |
| Feet | 237 | 0% | 0% | 0% | 0% | 39.2% | 60.8% |
| Shield | 441 | 2.0% | 55.6% | 16.3% | 15.2% | 6.3% | 4.5% |
| Cape | 374 | 1.9% | 2.1% | 7.8% | 5.1% | 11.5% | 71.4% |

Capes also had 0.3% identical copies with different IDs. Offset-adjusted corresponding vertices determine translation. The axis-scale category allows a maximum residual of 1.5 units. Among different-topology bodies the median female/male width, height and depth ratios were 0.82, 0.95 and 0.96. For hands they were 0.87, 1.04 and 0.78. Multiplying all coordinates by the same value moves the waist, wrists and ankles to the wrong heights.

## Fitting the standard female figure

The harness uses the lowest enabled kit ID for each of body parts 7–13, with the same five colour selections as the male harness. This includes jaw kit 292. Use the normal jaw for comparison or explicitly select the harness's jaw override when reviewing other appearances.

| Landmark, positive height above ground | Male | Female |
|---|---|---|
| Full figure bounds (x, height, z) | (−35,−6,−20) to (35,196,12) | (−31,−6,−17) to (30,192,13) |
| Body kit bounds | (−18,114,−13) to (18,165,11) | (−16,121,−15) to (16,162,10) |
| Front collar | x ±7, height 164, z −8 | x ±4, height 161, z −3 |
| Back collar | x ±7, height 165, z 4 | x ±5, height 162, z 5 |
| Right wrist | x −32 to −25, height 114, z −2 to 4 | x −26 to −21, height 110, z 0 to 4 |
| Waist | Back x ±13, height 114, z 8 | Back x −13 and 12, height 121, z 5 |
| Ankle | Front height 18, back 19 | Front height 17, back 18 |

The female front collar uses label 49 on the head and 8 on the torso. The rear uses 2 and 94. The female waist uses 4 on the body and 30 on the legs. Snap coordinates and copy the slot's label convention. A shared bind coordinate alone does not prove that the seam stays closed when posed. Literal coordinates for both sides, the neck, the hips and the root are in the seam map in female_warp.py. The frequency survey counts distinct meshes here.

For weapons, first test the real neighbour's offset and inspect the hand. The common distinct-mesh translation across all moved weapons was (+4, −5, +1) in x, height, z. By family the modal moved-copy shifts differ. The sweep found (+1, −3, 0) for swords/scimitars, (+6, −5, 0) for halberds/spears/glaives, and (+4, −5, 0) for staves. No single displacement fits every handle.

## Framing and lighting the inventory icon

The client prepares the mesh, lights it at ambient 64 plus the item value and contrast 768 plus the decoded item value, with direction (−50,−10,−50), then projects at focal length 512. It rotates using the item's zan, yan and xan stages, includes half the model height, and applies signed view offsets. Increasing zoom generally makes the icon smaller. The 36×32 path uses the half-height origin, including x = 16. A black outline uses colour 1 around the four-connected silhouette. Level two adds white. Shadow fills a transparent pixel diagonally below and right of a drawn pixel. The board setting is outline 1, shadow 0x302020, quantity 1, no count text, palette brightness 0.8.

Pose the inventory mesh first. Close the gap left for the fist in a weapon, remove the wearer from an empty helm, and arrange paired hands. Set the mesh’s base near the neighbour's base and compare the real-sized silhouettes. Start with the nearest design's camera values, fit zoom and offsets to the median box of eight neighbours, and keep the distinctive parts legible. Check the ground independently, since its camera ignores the icon angles. The eight view JSON files preserve the selected transform, neighbour IDs, target box and fit attempts.

## Making a dialogue helm

Of 1,049 male worn head-slot items, 941 had a different-topology dialogue mesh, 103 had a reshaped mesh with the same indexed topology and five had no head model. Of 274 that clear hair alone, all 274 had different topology. Across 1,044 readable dialogue meshes, median extent ratios relative to worn were 3.97×, 4.00× and 4.00×. The median face-count ratio was 1.43. After fitting a single translation the median fraction of matching vertex positions was only 2%. There were 1,044 female dialogue models, of which 302 used the same IDs as the male.

For the barbute, identify the 121 embedded low-detail kit faces, remove them, insert the actual 286-triangle dialogue face, retain its expression labels and attach the shell to dialogue label 15. Scale the shell fourfold about the embedded face and translate it into dialogue space. Keep the jaw slot available. This registration is specific to the B shell. A fully enclosed helm needs a different face-visibility decision.

## Evidence

Run the following in the research-items directory. The first command re-reads the cache in a temporary output directory. Its two retained transcripts contain the underlying denominators, distributions by family, definitions of the tests and SHA-256 comparisons with the inherited data. The second command compares the file-input pose path with the client item path.

~~~powershell
{PYTHON} -I recheck_survey.py
{PYTHON} -I pose_controls.py controls
~~~

[First survey](completed/measure_items.txt) · [Family and icon survey](completed/measure_items2.txt) · [Female pose controls](completed/pose-controls.txt) · [Kit-by-kit bounds](data/kits-compare.txt) · [Seam frequencies](data/seams-frequency.txt) · [Cape construction](capes/CAPES.md)

