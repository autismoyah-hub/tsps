# Comparing several variants of your own model in one line-up

A line-up takes a few seconds per entry and view. When you are unsure between two ways of splitting a surface,
two tones, or with and without an edge ring, do not guess and do not spend a round on each: export each variant
to its own `.bin` and put them all in ONE line-up next to the real object. `lineup.py` already does this; no extra
tool is needed.

```sh
PY="{PYTHON} -I"
BL="{BLENDER}"
B={ROOT}/builds/MY-BUILD

# 1. one .bin per variant. Let the build script read its options from the environment, e.g.
#    OPT = os.environ.get('VARIANT', 'a')   and   save(os.environ.get('OUT_BLEND', default_path))
mkdir -p $B/var
for v in a b c; do
  VARIANT=$v OUT_BLEND="$B/var/$v.blend" "$BL" -b -P $B/build_blender.py 2>&1 | grep -E "built|Error|Traceback"
  "$BL" -b $B/var/$v.blend -P {ROOT}/tools/blender_export.py -- $B/var/$v.bin 2>&1 | grep -E "EXPORT|WARNING|Error"
done

# 2. one line-up: the real object first, then every variant (same camera options as your normal rounds)
$PY {ROOT}/tools/lineup.py {ROOT}/render/MY-BUILD-var1 \
    "REAL_ID,$B/var/a.bin,$B/var/b.bin,$B/var/c.bin" --views 4 --ambient A --contrast C

# 3. look at render/MY-BUILD-var1/lineup.png (column 1 = real, then a, b, c), then score each against the real
$PY {ROOT}/author-pack/tools/pairs.py {ROOT}/render/MY-BUILD-var1
```

With more than two entries `pairs.py` compares every one of ours with the real one, writes
`pairs/pair_eK_vR.png` (K = position in the entry list, counted from 0) and ends with a table "variants, best
first". Tested with `"15704,crate.bin,crate_r11.bin"`: 4.93 against 4.98.

Things to know:

- **One `--ambient/--contrast` per call.** All of our entries get the same lighting values. To compare lighting
  values, make one line-up per pair of values; or skip the renderer and run `predict_light.py MODEL.bin --ambient A
  --contrast C` for each, which answers [quoted words withheld] in under a second.
- **`OUT_DIR` must not exist.** Number your variant line-ups (`-var1`, `-var2`); do not delete old ones mid-round,
  they are the record.
- **Use backslashes or a full `drive:/...` path with care in environment variables for Blender**; a path given as
  `drive:/...` works in both.
- **A difference of a few hundredths in the score is noise**; the two crate builds above differ by 0.05. Trust a
  variant when the score moves by several tenths AND the pair picture shows why. Change one thing per variant, or
  you will not know what moved the score.
- **Keep the winner's reason in your notes** (what the variants were, the scores, which you kept), as the crate's
  author did; a later author will otherwise retry the losers.
- Up to about six entries stay readable in `lineup.png`. In the turned views the neighbouring entry can show at the
  edge of the full captures `eKvR.png`; the centre window that `lineup.png` and `pairs.py` use is clear of it.
