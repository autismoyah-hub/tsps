"""Search the NPC census by name. usage: boss_find.py PATTERN[,PATTERN..] [--all]
Prints id, name, size, combat, models, stand/walk, transforms, scale, ambient/contrast; groups identical model lists."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, pathlib
C = pathlib.Path(_AP.fix(r'<ROOT>\studies\research-npc\census'))
npcs = [json.loads(l) for l in (C / 'npcs.jsonl').read_text(encoding='utf-8').splitlines() if l.strip()]
pats = [re.compile(p, re.I) for p in sys.argv[1].split(',')]
for n in npcs:
    if any(p.search(n['name']) for p in pats):
        print(n['id'], repr(n['name']), 'size', n['size'], 'cb', n['combat'], 'models', n['models'], 'stand', n['anims'][0], 'walk', n['anims'][3],
              'tf' if n['transforms'] else '', 'scale', n['scale'], 'amb', n['ambient'], 'con', n['contrast'], 'recol', len(n['recolor']))
