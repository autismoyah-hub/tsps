"""Add three safe NUMBERS of the real boss to its v2 sheet (Teacher-side): the median fold between neighbouring faces,
the count of small contrasting islands, and how many faces each kind of pose shows (rest only / alive / death only /
never). They are statistics, not geometry: targets an author can check their own mesh against.
usage: sheet_numbers.py NPC_ID SHEET_DIR"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections
import numpy as np
import tl, layers
from tl import cmesh
npc = int(sys.argv[1]); sheet = pathlib.Path(sys.argv[2]); d = tl.real_def(npc)
meas = cmesh.measure(cmesh.load(f'npc:{npc}'), tuple(d['scale']))
m, kv = tl.real_mesh(npc); cls = layers.table('REAL', m, layers.vis(m, d, f'real-{npc}', sheet))
info = json.loads((sheet / 'sheet.json').read_text())
info['style_targets'] = dict(fold_med_deg=round(meas['fold_med_deg'], 1), fold_over30_share=round(meas['fold_over30_share'], 3), detail_islands=meas['detail_islands'], colour_edge_share=round(meas['colour_edge_share'], 3),
                             valence_6_share=round(meas['valence_6_share'], 3), faces_for_half_surface=round(meas['faces_for_half_surface'], 3))
info['faces_by_pose'] = dict(collections.Counter(cls.tolist())); info['faces_by_pose_note'] = 'both = shown at rest and alive; rest only = shown in the rest pose but not in stand, walk or attack; live only = the reverse; death only; never = no pose-set cell shows it'
(sheet / 'sheet.json').write_text(json.dumps(info, indent=1)); print(info['style_targets'], info['faces_by_pose'])
