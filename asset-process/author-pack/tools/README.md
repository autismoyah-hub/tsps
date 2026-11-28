# Author's toolkit

Small tools for the build / export / line-up loop, collected from the helpers three authors wrote for the crate,
the lectern and the bench. Private to this machine. They read only our own exported `.bin` files and line-up
folders; none of them opens a real object's mesh.

    PY="{PYTHON} -I"
    T={ROOT}/author-pack/tools

A line-up folder `OUT` (made by `tools/lineup.py`) holds one full capture per entry and view, `OUT/eKvR.png`
(K = entry, R = view, both from 0; 1280x900, the object centred), and the tiled `OUT/lineup.png`. All pixel
coordinates below are pixels of a capture `eKvR.png`, not of `lineup.png`.

## predict_light.py: what will the game's light do to this model?

    $PY $T/predict_light.py MODEL.bin --ambient A --contrast C              # table per colour and facing
    $PY $T/predict_light.py MODEL.bin --ambient A --contrast C --faces      # one line per face, three corners
    $PY $T/predict_light.py MODEL.bin --ambient A --contrast C --picture OUT.png --view 2 --dist 420 --elev 300

A port of the client's own lighting (`rust-client/src/model_light.rs`, `normals` and `light_with_normals`), with
its integer arithmetic. Give the definition's ambient and contrast, the same numbers you give `lineup.py`; with no
options it uses lineup.py's defaults, 10 and 30. It prints the lit lightness every face corner will have, so you
can choose a paint lightness or a triangulation without rendering. `--picture` draws the prediction from the
line-up camera, the same frame as a capture, using the client's colour table, its edge walker and its 4-pixel
shading step. From another script: `import predict_light as pl; m = pl.load(path); lit = pl.light(m, A, C)` gives per
face the three lit 16-bit colours (`& 127` = lightness); `pl.project` and `pl.picture` give screen positions.

The arithmetic, at contrast 0: `light = 64 + ambient + (n . L) / 213`, between about -21 and +149 before ambient;
`lit lightness = paint x light >> 7`, kept within 2..126. A face that gets no directional light shows HALF its
painted lightness at ambient 0. Flat-marked faces divide by 319 instead of 213 and do not vote in vertex normals.

Measured against the renderer (`check_predict.py`, below): flat faces exact; smooth faces within 1 lightness step
on 92 to 97% of pixels for the crate and lectern, 86 to 94% for the bench. Limits: the table and `--faces` give
corner values, and on a face narrower than about 8 pixels the screen shows 4-pixel blocks of them, up to 7 steps
from the plain mean of the corners; the picture hides surfaces by depth, the game by draw priority and face
sorting, so overlapping parts can differ; an object standing ALONE is assumed (the game merges vertex normals of
neighbouring objects that share vertex positions); textured or transparent faces are not handled beyond what the
source does; views are the four of `lineup.py`, rotation 0.

## check_predict.py: the predictor's test

    $PY $T/check_predict.py MODEL.bin LINEUP_DIR ENTRY --ambient A --contrast C --dist D --elev E [--save]

Compares the prediction with the captures of the same model in a line-up folder (ENTRY = its position in the
entry list, from 0), face by face at face centres and pixel by pixel. `--save` writes
`LINEUP_DIR/predicted_vR.png` (capture | prediction | where they differ). Run it after any change to
`predict_light.py` or to the renderer. `testdata/make_drum.py OUT.bin` builds the test drum (large flat-marked and
smooth faces in eight directions); `testdata/drum.bin` is its output.

## pairs.py: real against ours, enlarged, with a difference map and a score

    $PY $T/pairs.py LINEUP_DIR [--out DIR] [--scale 3] [--full 30] [--crop PX]

For a line-up of the real object and ours (either order), writes one picture per view, `LINEUP_DIR/pairs/pair_vR.png`:
real | ours | difference. In the difference panel grey is equal, orange to red is ours lighter, blue is ours darker,
green is where only the real one covers the pixel and magenta where only ours does. Prints per view the mean
absolute difference of grey level (0 to 255) over the pixels both cover, the mean signed difference, the share of
pixels more than 8 apart, and the outline mismatch in pixels. With several of ours in the line-up it scores each
against the real one (see `variants.md`).

Limits: the score is a difference, not a ratio, on purpose: ratios exaggerate on dark faces (at grey 25 one
lightness step is 10 to 15%), which is why two authors saw their ratio scores stall while the pictures improved.
It still cannot tell a wrong tone from a shifted edge: read the picture, not only the number. The ground is
recognised exactly by its colour, so a part painted in the ground's own hue and saturation (7/0 or 8/0) would be
taken for ground. Only the centre window that `lineup.png` shows is compared. Differences of a few hundredths
between runs are noise.

## px2col.py: read game colours off rendered pixels

    $PY $T/px2col.py OUT/e0v0.png 640,450 620,330          # single pixels
    $PY $T/px2col.py OUT/e0v0.png rect 540,220,740,580     # every colour in a rectangle
    $PY $T/px2col.py OUT/e0v0.png col 640 230 570          # runs of colour down pixel column x=640
    $PY $T/px2col.py OUT/e0v0.png row 500 300 980          # runs along pixel row y=500

Turns pixels of a capture back into (hue, saturation, lightness after lighting). Use it on the real object's
capture when its reference sheet lists no recolours: `rect` over the whole object lists its hue/sat pairs with
their share of the pixels; `col` and `row` give the height or width in pixels of each layer together with its tone,
for real and ours in turn. The lookup is exact, because every pixel of a classic capture is an entry of the client's
colour table (`gamecol.py`, a port of `palette()` in `rust-client/src/floor.rs` at brightness 0.8, not a fit).

Limits: you get the LIT lightness. The painted lightness is `lit x 128 / light`; take `light` for that facing from
`predict_light.py` on your own model, or compare with your own face of known paint on the same surface. Several
colours share one RGB when dark (below lit lightness of about 12 the hue is a guess): read hue and saturation on
the lit side, from a patch, not a pixel. Lighting never changes hue or saturation, so those two are what the
artist painted. Pictures that were resized (old `cmp_*.png`) or carry text (`lineup.png`'s numbers) are not exact;
such pixels are reported as nearest matches.

## gamecol.py: the colour table (module)

`gamecol.table()` (65536 RGB rows), `gamecol.rgb(h, s, l)`, `gamecol.nearest(rgb)`, `gamecol.ground_mask(image)`.

## variants.md

How to line up several variants of your own model in one call and score them.

## A round with these tools

    $PY {ROOT}/tools/lineup.py $R "REAL_ID,$B/mine.bin" --views 4 --ambient A --contrast C
    $PY $T/pairs.py $R                                   # look at $R/pairs/pair_v*.png, note the scores
    $PY $T/px2col.py $R/e0v0.png rect X0,Y0,X1,Y1        # the real one's colours
    $PY $T/px2col.py $R/e0v0.png col X ; $PY $T/px2col.py $R/e1v0.png col X     # layer heights, real then ours
    $PY $T/predict_light.py $B/mine.bin --ambient A --contrast C --faces        # before the next build: what will it do
