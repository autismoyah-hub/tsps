#!/bin/sh
eval "$("${PYTHON:-python}" -I "$(dirname "$0")/../../../asset_paths.py" --sh)"  # paths: see SETUP.md
# Step 4: a new bone skeleton from nothing, a weighted mesh that bends, two posed animations, packed and drawn by id.
set -e -o pipefail
cd "$(dirname "$0")/.."
BL="${BLENDER}"; P="${PYTHON}"
W=worm/work; R=${ROOT}/render/research-rig-worm; mkdir -p $W $R
"$BL" -b --python-exit-code 1 -P worm/build_worm.py -- $W/worm-mesh.blend 2>&1 | grep -E "^worm|rror|Trace"
"$BL" -b $W/worm-mesh.blend --python-exit-code 1 -P rig_weights.py -- $W/worm-weighted.blend --method chain --shift 0 --band 0.5 --max 2 --steps 4 --snap 0.9 2>&1 | grep -E "^  |bones|rror|Trace"
"$BL" -b --python-exit-code 1 -P worm/animate_worm.py -- $W/worm-weighted.blend $W/worm.blend 2>&1 | grep -E "poses at|rror|Trace"
"$BL" -b $W/worm.blend --python-exit-code 1 -P rig_export.py -- $W/export --base-id 12700 2>&1 | grep -E "^EXPORTED|^  |rror" | cut -c1-300
"$BL" -b $W/worm.blend --python-exit-code 1 -P rig_export.py -- $W/export-spline --base-id 12700 --mode spline --step 3 2>&1 | grep -E "^EXPORTED|rror" | cut -c1-300
printf 'name=Rig worm\nmodels=%s\nscale=128,128\nambient=30\ncontrast=20\nzbuffer=1\n' "$(cd $W/export && pwd -W)/part-0.bin" > $W/worm.npcdef
"$P" -I bonecodec_summary.py $W/export
rm -rf $W/cache-packed
"$P" -I rigpack.py $W/export $W/cache-packed worm/pack.json
"$P" -I rig_byid.py $W/cache-packed $W/export $W/worm.npcdef --out $R/byid
"$P" -I rig_show.py $W $R/worm-sheet ours:$W/export/anim_sway.dat:sway ours:$W/export/anim_strike.dat:strike --model $W/worm.npcdef --base $W/export/base_12700.dat --every 4 --cell 300 --zoomk 2.0 --yaw 512 --wrap 10
"$P" -I rig_show.py $W $R/worm-bend-close ours:$W/export/anim_strike.dat:strike ours:$W/export-spline/anim_strike.dat:strike-as-true-curves --model $W/worm.npcdef --base $W/export/base_12700.dat --every 3 --cell 420 --zoomk 1.15 --yaw 512 --wrap 10
"$P" -I rig_pair.py $W $W/export/anim_strike.dat $W/export-spline/anim_strike.dat --model $W/worm.npcdef --base $W/export/base_12700.dat --frames every:3
