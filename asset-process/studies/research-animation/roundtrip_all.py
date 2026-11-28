"""Whole-cache codec check: every sequence definition, every frame base, every classic frame is decoded and
re-encoded; counts how many come back byte-identical, and for the rest whether they decode to the same thing.

usage: roundtrip_all.py
"""
import sys, collections
sys.path.insert(0, str(__import__('pathlib').Path(__file__).parent))
import animlib as A

c = A.Cache()
seqs = c.files(2, 12)
same = 0; classic = 0; keyed = 0; ops = collections.Counter(); bad = []
for sid, data in seqs.items():
    s = A.decode_sequence(data)
    for o in s['ops']: ops[o] += 1
    if s['cached_animation'] is not None: keyed += 1
    elif s['frames']: classic += 1
    if A.encode_sequence(s) == data: same += 1
    else: bad.append(sid)
print(f'sequences: {len(seqs)} definitions, highest id {max(seqs)}; classic (opcode 1 frames) {classic}, keyframe (opcode 13) {keyed}, '
      f'neither {len(seqs) - classic - keyed}')
print(f'sequences re-encoded byte-identical: {same} of {len(seqs)}; not identical: {bad[:10]}')
print('opcode use:', dict(sorted(ops.items())))

ref1 = c.reference(1)['groups']
same = 0; tails = 0; kinds = collections.Counter()
bases = {}
for bid in ref1:
    data = c.file(1, bid, 0)
    b = A.decode_base(data); bases[bid] = b
    tails += bool(b['tail'])
    for t in b['types']: kinds[t] += 1
    same += A.encode_base(b) == data
print(f'frame bases: {len(ref1)}; re-encoded byte-identical {same}; with a bone section (for keyframe animation) {tails}')
print('transform types over all bases:', dict(sorted(kinds.items())))

ref0 = c.reference(0)['groups']
nframes = same = sem = 0; fail = []; mixed = 0
for gid in ref0:
    try:
        files = c.files(0, gid)
    except Exception as e:
        fail.append((gid, repr(e))); continue
    first = next(iter(files.values()))
    bid0 = int.from_bytes(first[:2], 'big')
    base = bases[bid0]
    for fid, data in files.items():
        nframes += 1
        if int.from_bytes(data[:2], 'big') != bid0: mixed += 1
        try:
            f = A.decode_frame(data, base)
        except Exception as e:
            fail.append((gid, fid, repr(e))); continue
        if A.encode_frame(f, base) == data: same += 1
        canon = A.encode_frame(dict(base=f['base'], groups=f['groups']), base)
        if A.decode_frame(canon, base)['groups'] == f['groups']: sem += 1
print(f'frame groups: {len(ref0)}; frames {nframes}; re-encoded byte-identical (stored masks kept) {same}; '
      f'canonical re-encoding (masks derived from values) decodes to the same values {sem}; frames whose own base id differs from the group\'s first frame {mixed}; failures {len(fail)} {fail[:5]}')
