"""Dump male worn geometry with raw paint and y-up coordinates, as in real_dump.py worn."""
import argparse
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from handoff import osrscache as o
import scene


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('item')
    parser.add_argument('out', type=Path)
    a = parser.parse_args()
    if not re.fullmatch(r'item:\d+', a.item):
        parser.error('expected item:ID')
    iid = int(a.item[5:])
    definition = json.loads(next((scene.ARCH/'items').glob(f'*/{iid}.json')).read_text())
    models, vl, fl = [], [], []
    for mid in definition['wear_models'][0]:
        if mid in (-1, None):
            continue
        raw = o.unpack(o.group(7, mid))
        m = o.decode_model(raw)
        footer = {'legacy': 18, 'skinned-legacy': 23, 'extended': 23, 'skinned-extended': 26}[m['format']]
        models.append(m)
        vl.extend(m['vertex_skins'] or [None]*len(m['vertices']))
        fl.extend(m['face_skins'] if raw[-footer+8] else [None]*len(m['faces']))
    if not models:
        parser.error('item has no male worn model')
    m = o.merge(models)
    result = dict(kind='worn', id=iid, vertices=[(x,-y,z) for x,y,z in m['vertices']],
                  faces=m['faces'], colours=list(map(o.hsl_split, m['colors'])),
                  flat=[int(r == 1) for r in m['render_types']], priorities=m['priorities'],
                  vertex_labels=vl, face_labels=fl)
    a.out.write_text(json.dumps(result)+'\n')
    print(f'{a.item}: {len(vl)} vertices, {len(fl)} faces, {len(m["priorities"])} priorities -> {a.out}')


if __name__ == '__main__':
    main()
