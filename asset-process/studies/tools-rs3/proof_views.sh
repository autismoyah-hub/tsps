#!/bin/sh
eval "$("${PYTHON:-python}" -I "$(dirname "$0")/../../asset_paths.py" --sh)"  # paths: see SETUP.md
# Ten proof drawings with the finished viewer (bosses, weapons, armour).
P=${PYTHON}
B=${ROOT}/studies/tools-rs3
for pair in npc:27352=raksha npc:28069=kerapac npc:28070=echo-of-kerapac npc:30690=vorkath \
            item:47470=eldritch-crossbow item:31725=noxious-scythe item:53338=bow-of-the-last-guardian \
            item-worn:53375=havoc-hood-worn item-worn:29857=sirenic-hauberk-worn item-worn:28608=tectonic-mask-worn; do
  spec=${pair%%=*}; name=${pair#*=}
  "$P" -I -B "$B/rs3view.py" "$spec" "$B/proof-views/$name.png" --blend "$B/proof-views/$name.blend" 2>/dev/null || echo "FAILED $spec"
done
