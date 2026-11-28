"""Apply part labels after the atlas and template rules, before balancing and face labelling.

PARTS is a JSON object. "indices" maps zero-based vertex indices to labels. "boxes" is an
ordered list of {"min":[x,height,z],"max":[x,height,z],"label":n}, inclusive. Later
matches win. Optional "points_file" references [x,height,z,label-or-tag] records relative
to PARTS. "tags" maps named parts to a label or {"label":n,"preserve":[seam labels]}.
Processing order is points, boxes, indices. "pivot_targets" maps comma-joined pivot label
numbers to [x,height,z]. "pivot_keep" sets the minimum retained by the atlas balancer.
"""
import argparse
from pathlib import Path
import json
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from handoff import decode
import label_armour as la
import numpy as np


def overrides(spec, folder, vertices, labels, unsure):
    def assign(indices, value):
        rule = value if isinstance(value, dict) else {'label': value}
        lab = rule['label']
        if type(lab) is not int or not 0 <= lab <= 255:
            raise ValueError(f'invalid label: {lab}')
        for i in indices:
            if not 0 <= i < len(labels):
                raise ValueError(f'vertex index out of range: {i}')
            if labels[i] not in rule.get('preserve', []):
                labels[i] = lab
                unsure.pop(i, None)

    if 'points_file' in spec:
        points = json.loads((folder/spec['points_file']).read_text())
        points = {(x,h,z): value for x,h,z,value in points}
        for i, vertex in enumerate(vertices):
            value = points.get(tuple(vertex))
            if value is not None:
                assign([i], spec['tags'][value] if isinstance(value, str) else value)
        missing = points.keys() - set(map(tuple, vertices))
        print(f'point overrides without a matching vertex: {len(missing)}')
    for box in spec.get('boxes', []):
        low, high = np.asarray(box['min']), np.asarray(box['max'])
        if low.shape != (3,) or high.shape != (3,) or (low > high).any():
            raise ValueError('invalid inclusive box bounds')
        assign(np.flatnonzero(np.all((vertices >= low) & (vertices <= high), axis=1)), box['label'])
    for index, label in spec.get('indices', {}).items():
        assign([int(index)], label)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path)
    p.add_argument('slot')
    p.add_argument('out', type=Path)
    p.add_argument('--parts', type=Path, required=True)
    p.add_argument('--template', type=int)
    p.add_argument('--near', type=float, default=3)
    for flag in ['sleeveless','no-balance','no-face-labels','quiet']:
        p.add_argument('--'+flag, action='store_true')
    a = p.parse_args()
    slot = la.SLOTS[a.slot] if a.slot in la.SLOTS else int(a.slot)
    if slot not in la.SLOT_NAME or (a.sleeveless and slot != 4):
        p.error('invalid slot or sleeveless used outside body slot')
    spec = json.loads(a.parts.read_text())
    if not isinstance(spec, dict):
        p.error('parts must be a JSON object')
    if spec and all(k.isdigit() for k in spec):
        spec = {'indices': spec}
    unknown = spec.keys() - {'points_file','tags','boxes','indices','pivot_targets','pivot_keep'}
    if unknown:
        p.error(f'unknown parts keys: {sorted(unknown)}')
    m = decode(a.source)
    v, f = np.array(m['vertices'], float), np.array(m['faces'], int)
    labels, unsure, stats = la.label_model(slot, v, f, a.template, a.near, balance=False, sleeveless=a.sleeveless)
    overrides(spec, a.parts.parent, v * [1,-1,1], labels, unsure)
    stats = stats or {}
    if not a.no_balance:
        targets = {}
        if a.template is not None:
            _, tv, tl, _, _ = la.template_mesh(a.template)
            if la.coverage(v, tv) >= la.COVER:
                targets = la.template_pivots(slot, tv, tl)
        targets.update({tuple(map(int, k.split(','))): t for k,t in spec.get('pivot_targets', {}).items()})
        labels, stats['pivot notes'] = la.balance_pivots(slot, labels, v, targets, keep=spec.get('pivot_keep',3),
                                                         sleeveless=a.sleeveless)
    fl = None if a.no_face_labels else la.face_labels(slot, labels, f)
    if a.sleeveless and (set(labels) & la.ARM_LABELS or set(fl or []) & la.ARM_FACE_LABELS):
        p.error('arm label on sleeveless body')
    data = la.encode(m, labels, fl)
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_bytes(data)
    print(f'{a.out}: {len(data)} bytes')
    la.report(slot, labels, fl or [], unsure, v, stats, a.quiet, sleeveless=a.sleeveless)


if __name__ == '__main__':
    main()
