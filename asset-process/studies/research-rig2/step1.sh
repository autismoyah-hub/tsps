#!/bin/sh
eval "$("${PYTHON:-python}" -I "$(dirname "$0")/../../asset_paths.py" --sh)"  # paths: see SETUP.md
# Step 1: a real creature into Blender and back. usage: sh step1.sh NAME NPC_ID [SEQS for the pure run]
# Writes work/NAME/{src,manifest.json,NAME.blend,export,export-pure} and logs/step1-NAME-*.log
set -e
cd "$(dirname "$0")"
BL="${BLENDER}"; P="${PYTHON}"
N=$1; ID=$2; mkdir -p logs
"$P" -I rig_fetch.py $ID work/$N --check | tee logs/step1-$N-fetch.log
"$BL" -b --python-exit-code 1 -P rig_import.py -- work/$N/manifest.json work/$N/$N.blend $3 2>&1 | grep -E "^(action|IMPORTED|Error)" | tee logs/step1-$N-import.log
"$BL" -b work/$N/$N.blend --python-exit-code 1 -P rig_export.py -- work/$N/export 2>&1 | grep -E "^(EXPORTED|  model|  action|Error)" > logs/step1-$N-export.log
"$BL" -b work/$N/$N.blend --python-exit-code 1 -P rig_export.py -- work/$N/export-pure --pure 2>&1 | grep -E "^(EXPORTED|  model|  action|Error)" > logs/step1-$N-export-pure.log
"$P" -I rig_compare.py work/$N work/$N/export --json work/$N/compare.json | tee logs/step1-$N-compare.log
"$P" -I rig_compare.py work/$N work/$N/export-pure --json work/$N/compare-pure.json --keep ${ROOT}/render/research-rig-step1/$N-pure | tee logs/step1-$N-compare-pure.log
