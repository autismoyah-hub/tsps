#!/bin/sh
eval "$("${PYTHON:-python}" -I "$(dirname "$0")/../../../asset_paths.py" --sh)"  # paths: see SETUP.md
# Step 3: the hard bend. A thick arm (elbow) and a thick leg (knee) on a new 7-bone skeleton; weights by the ring rule in
# each style; bent to 140 and 130 degrees; drawn close up and measured, beside real creatures' joints at the same angles.
set -e -o pipefail
cd "$(dirname "$0")/.."
BL="${BLENDER}"; P="${PYTHON}"
W=bend/work; R=${ROOT}/render/research-rig2-bend; mkdir -p $W $R
"$BL" -b --python-exit-code 1 -P bend/build_bend.py -- $W/mesh.blend 2>&1 | grep -E "^bend|rror|Trace"
for S in ${STYLES:-hard medium soft}; do
  "$BL" -b $W/mesh.blend --python-exit-code 1 -P rig_weights.py -- $W/w-$S.blend --method nearest --shift 0 --rings $S --parts bend/bend.parts.json $EXTRA 2>&1 | grep -E "^  |rror|Trace"
  "$BL" -b --python-exit-code 1 -P bend/animate_bend.py -- $W/w-$S.blend $W/a-$S.blend 2>&1 | grep -E "poses at|rror|Trace"
  "$BL" -b $W/a-$S.blend --python-exit-code 1 -P rig_export.py -- $W/x-$S --base-id 12710 2>&1 | grep -E "^EXPORTED|rror" | cut -c1-200
done
