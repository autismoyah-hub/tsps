"""pkg_dump.py KEY [spots|locs|seqs|npcs|colour|skel]   detailed lines from data/packages.json"""
import sys, json, pathlib
D = json.loads((pathlib.Path(__file__).resolve().parent / 'data' / 'packages.json').read_text())
k = sys.argv[1]; what = sys.argv[2:] or ['npcs', 'colour', 'skel', 'seqs', 'spots', 'locs']
p = D[k]
if 'npcs' in what:
    for n in p['npcs']: print('npc', n['id'], n['gv'], '|', n['name'], 'size', n['size'], 'cb', n['combat'], 'models', n['models'], 'base', n['skeleton'], 'scale', n['scale'], n['role'])
if 'colour' in what:
    for i, fs in p['colours'].items():
        print('colour', i, [(f['family'], f['share'], f['tones'], f['hue'], f['sat'], f['light']) for f in (fs or [])[:7]])
        print('sectors', i, [(x['sector'], x['share'], x['sat'], x['low_mid_top']) for x in (p['sectors'][i] or [])[:8]])
        print('accent', i, p['accent'][i])
    for m in p['models']: print('model', m)
if 'skel' in what:
    for s in p['skeleton']: print('skel', s)
if 'seqs' in what:
    for s in p['seqs']: print('seq', s['id'], s['gv'], s['kind'], 'base', s['skeleton'], 'cyc', s['cycles'], 'ON' if s['on_base'] else '', s['role'])
if 'spots' in what:
    for s in p['spots']: print('spot', s['id'], s['gv'], s['reuse'], 'SRV' if s['server_uses'] else '', s['role'], 'f', s['faces'], 'c90', s['colours90'], 'a', s['alpha_share'], 'sz', s['size'], s['kind'], 'cyc', s['cycles'], 'tex', s['textured'], 'w', s['weights'], s['sector_relation'])
if 'locs' in what:
    for l in p['locs']: print('loc', l['id'], l['gv'], l['role'], 'size', l.get('size'), 'f', l.get('faces'), 'a', l.get('alpha_faces'), 'tex', l.get('textured'), 'anim', l.get('anim'), 'walk' if l.get('walk_through') else '')
