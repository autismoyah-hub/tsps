"""Which way does a bone's channel turn? One picture per (bone, channel): the stance, and the stance with that one
channel offset, drawn by the client from a keyframe file written by bone_author.

usage: bone_probe.py NPC_ID STANCE_SEQ OUT.png BONE[,BONE..] [--amount 0.4] [--channels rot.x,rot.y,rot.z] [--yaw 192] [--tran 30]
Rows = bones; columns = stance, then each channel at +amount. Also checks that a file with no tracks draws exactly the
real stance frame (0 differing pixels) and prints the count.
"""
import sys, pathlib, json, tempfile
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
from PIL import Image, ImageDraw
import bonelib as L, bonecodec as B, bone_author as BA
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
npc, seq, out = int(a[1]), int(a[2]), pathlib.Path(a[3]); bones = [int(x) for x in a[4].split(',')]
amount = float(opt('--amount', 0.4)); chans = opt('--channels', 'rot.x,rot.y,rot.z').split(','); yaw = int(opt('--yaw', 192))
base = L.base_of_seq(seq); d = L.npc_def(npc); tmp = pathlib.Path(tempfile.mkdtemp(prefix='probe-'))
M = L.merged(f'npc:{npc}', tmp / 'rest.txt'); ext = (M['V'].max(0) - M['V'].min(0)); zoom = int(float(opt("--zoomk", 1.2)) * max(ext)); cs = 300
def cell(tracks):
    anim, _ = BA.build(dict(base=base, stance=dict(seq=seq, frame=0), length=3, mode='baked', step=3, tracks=tracks))
    p = tmp / 'a.dat'; p.write_bytes(B.encode_anim(anim))
    g, _, _ = L.grid(f'npc:{npc}', anim=p, count=4, frames='3', zoom=zoom, yaw=yaw, pitch=128, cell=f'{cs}x{cs}'); return g
real, _, _ = L.grid(f'npc:{npc}', seq=seq, frames='0', zoom=zoom, yaw=yaw, pitch=128, cell=f'{cs}x{cs}')
st = cell([]); print('stance file against the real sequence at frame 0:', int((real != st).any(2).sum()), 'differing pixels of', cs * cs)
rows = []
for b in bones:
    row = [st]
    for ch in chans:
        amt = float(opt('--tran', 30)) if ch.startswith('tran') else amount
        row.append(cell([dict(bone=b, channel=ch, keys=[[0, 0], [3, amt]])]))
    rows.append(np.concatenate(row, 1))
im = Image.fromarray(np.concatenate(rows, 0)); dd = ImageDraw.Draw(im)
for i, b in enumerate(bones):
    dd.text((4, i * cs + 4), f'bone {b}: stance', fill=(255, 255, 255))
    for j, ch in enumerate(chans): dd.text(((j + 1) * cs + 4, i * cs + 4), f'bone {b} {ch} +{amount if not ch.startswith("tran") else opt("--tran", 30)}', fill=(255, 255, 255))
out.parent.mkdir(parents=True, exist_ok=True); im.save(out); print(out, im.size)
