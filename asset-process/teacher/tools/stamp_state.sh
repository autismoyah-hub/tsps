#!/bin/sh
# Set STATE.md's "As of" line from the clock. Run this instead of typing a time: a typed time has been wrong more than once.
dir=$(cd "$(dirname "$0")/.." && pwd)
sed -i "s/^As of: .*/As of: $(date -u +%Y-%m-%dT%H:%MZ)/" "$dir/teacher/project/STATE.md"
grep -n "^As of:" "$dir/teacher/project/STATE.md"
