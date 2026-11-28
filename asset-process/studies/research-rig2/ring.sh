#!/bin/sh
eval "$("${PYTHON:-python}" -I "$(dirname "$0")/../../asset_paths.py" --sh)"  # paths: see SETUP.md
# Step 1, one trial on a real creature. usage: sh ring.sh NAME TRIAL MODE [options...]   [GATE="--walk SEQ" in the environment]
#   MODE real     the real weights' heaviest bone per vertex, then the ring rule:  options are ring_lab.py's (--style, --kinds none ..)
#   MODE rule     the real weights THROWN AWAY: rig_weights.py in Blender with the options given (e.g. --method nearest --rings soft)
# -> work/NAME/TRIAL/{part-K.bin, score.txt, gate.txt}; prints the score line and the gate's motion lines
set -e -o pipefail
cd "$(dirname "$0")"
BL="${BLENDER}"; P="${PYTHON}"
N=$1; T=$2; M=$3; shift 3; D=work/$N/$T; mkdir -p $D
if [ "$M" = real ]; then
  "$P" -I ring_lab.py work/$N "$@" --out $D | grep -E "rings by rule|joints painted" | tee $D/score.txt | cut -c1-260
else
  "$BL" -b work/$N/$N.blend --python-exit-code 1 -P rig_weights.py -- $D/scene.blend "$@" 2>&1 | grep -E "^(  |[0-9]+ bones|Error|Traceback)|rror" | tee $D/weights.log | cut -c1-200
  "$BL" -b $D/scene.blend --python-exit-code 1 -P rig_export.py -- $D --actions none 2>&1 | grep -E "^(Error)|rror" || true
  "$P" -I ring_lab.py work/$N --score $D | tee $D/score.txt | cut -c1-260
fi
"$P" -I rig_gate.py work/$N $D/part-*.bin --txt $D/gate.txt $GATE > /dev/null || true
grep -E "outline overlap, mean|lost against|centroid gap, median|VERDICT" $D/gate.txt | sed -E 's/ \| provisional.*\| / | /' | cut -c1-90
