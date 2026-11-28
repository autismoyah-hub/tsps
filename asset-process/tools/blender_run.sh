#!/usr/bin/env bash
eval "$("${PYTHON:-python}" -I "$(dirname "$0")/../asset_paths.py" --sh)"  # paths: see SETUP.md
set -eu
if (( $# < 3 )); then
    echo 'usage: blender_run.sh BUILD_SCRIPT.py OUT.blend OUT.bin [exporter options]' >&2
    exit 2
fi
build=$1
blend=$2
bin=$3
shift 3
[[ $build == *.py && $blend == *.blend && $bin == *.bin ]] || { echo 'Expected .py, .blend and .bin paths' >&2; exit 2; }
rm -f -- "$bin"
success=0
trap 'if (( ! success )); then rm -f -- "$bin"; fi' EXIT
blender=${BLENDER:-'${BLENDER}'}
export PYTHONDONTWRITEBYTECODE=1
export FABLE_BLEND_OUT="$(cygpath -am "$blend")"
"$blender" --background --factory-startup -t 2 --python-exit-code 1 -P "$build" \
    --python-expr 'import bpy, os; dest=os.environ["FABLE_BLEND_OUT"]; bpy.ops.wm.save_as_mainfile(filepath=dest)'
"$blender" --background "$blend" -t 2 --python-exit-code 1 \
    -P ${ROOT}/tools/blender_export.py -- "$bin" "$@"
[[ -s $bin ]] || { echo "Export produced no data: $bin" >&2; exit 1; }
success=1
