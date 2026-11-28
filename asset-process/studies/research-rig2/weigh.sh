#!/bin/sh
eval "$("${PYTHON:-python}" -I "$(dirname "$0")/../../asset_paths.py" --sh)"  # paths: see SETUP.md
# Step 2, one trial: throw the real weights away, assign new ones, export the model, score it in motion.
# usage: sh weigh.sh NAME TRIAL [rig_weights.py options...]      -> work/NAME/w-TRIAL/{scene.blend,part-0.bin,score.txt}
set -e -o pipefail
cd "$(dirname "$0")"
BL="${BLENDER}"; P="${PYTHON}"
N=$1; T=$2; shift 2; D=work/$N/w-$T; mkdir -p $D
"$BL" -b work/$N/$N.blend --python-exit-code 1 -P rig_weights.py -- $D/scene.blend "$@" 2>&1 | grep -E "^(  |[0-9]+ bones|Error|Traceback)|rror" | tee $D/weights.log
"$BL" -b $D/scene.blend --python-exit-code 1 -P rig_export.py -- $D/export --actions none 2>&1 | grep -E "^(EXPORTED|  model|Error)|rror" >> $D/weights.log
"$P" -I rig_wscore.py work/$N $D/export/part-0.bin --json $D/score.json | tee $D/score.txt
