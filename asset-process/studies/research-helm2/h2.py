"""h2.py: the helm registry and drawing helpers of the research-helm2 study. Private: cache-derived.

Groups: ACC = accepted (the owner's five references, five more real end-game helms, and the one helm of ours the owner
passed); RR = rejected recently (the two Umbral helms); RE = rejected earlier (v4-a/b/c fitted but "missing sharpness,
depth and detail", and two helms designed by eye).
"""
import sys, pathlib, json
import numpy as np
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(ROOT / 'builds' / 'research-fit'))
import lib as L          # noqa: E402
import deflib            # noqa: E402

BB = (ROOT / 'builds').as_posix()
WORK = HERE / 'work'


def m(path, slot):
    return f'model:{path}@{slot}'


def v5set():
    return '+'.join([m(f'{BB}/meridian-cuirass-v5/meridian-cuirass-v5-labelled.bin', 'body+arms'),
                     m(f'{BB}/meridian-legs-v5/meridian-legs-v5-labelled.bin', 'legs'),
                     m(f'{BB}/meridian-gauntlets-v5/meridian-gauntlets-v5-labelled.bin', 'hands'),
                     m(f'{BB}/meridian-boots-v5/meridian-boots-v5-labelled.bin', 'feet')])


def folder_set(folder, suffix=''):
    return '+'.join(m(f'{folder}/{k}{suffix}.bin', s) for k, s in
                    (('cuirass', 'body+arms'), ('legs', 'legs'), ('gauntlets', 'hands'), ('boots', 'feet')))


# key: (group, name, helm piece (pose.py PIECE), body outfit (rest of the entry), mesh loader arg, body mesh arg, dark?)
H = {}


def real(key, grp, hid, body, legs):
    H[key] = dict(group=grp, name=L.item_name(hid), helm=f'item:{hid}', rest=f'item:{body}+item:{legs}',
                  mesh=('real', hid), body=('real', body), spec='head+hair+jaw' if 11 in L.wear_positions(hid)[1:] else 'head+hair')


def ours(key, grp, name, path, spec, rest, body):
    H[key] = dict(group=grp, name=name, helm=m(path, spec), rest=rest, mesh=('ours', path), body=('ours', body), spec=spec)


for r in L.REFS:
    real(r[0], 'ACC', r[1], r[3], r[4])
real('inquisitor', 'ACC+', 24419, 24420, 24421)
real('masori', 'ACC+', 27235, 27238, 27241)
real('dharok', 'ACC+', 4716, 4720, 4722)
real('statius', 'ACC+', 22625, 22628, 22631)
real('armadyl', 'ACC+', 11826, 11828, 11830)
V5C = f'{BB}/meridian-cuirass-v5/meridian-cuirass-v5-labelled.bin'
ours('v5', 'ACC', 'Meridian v5 (passed)', f'{BB}/meridian-helm-v5-astra/meridian-helm-v5-astra-labelled.bin', 'head+hair+jaw', v5set(), V5C)
UA = f'{BB}/umbral-helm-a'
UB = f'{BB}/umbral-helm-b'
ours('umbralA', 'RR', 'Umbral A (closed)', f'{UA}/umbral-helm-a-labelled.bin', 'head+hair+jaw', folder_set(UA, '-watch'), f'{UA}/cuirass-watch.bin')
ours('umbralB', 'RR', 'Umbral B (open)', f'{UB}/umbral-helm-b-labelled.bin', 'head+hair', folder_set(UB), f'{UB}/cuirass.bin')
for k in 'abc':
    ours(f'v4{k}', 'RE', f'Meridian v4-{k}', f'{BB}/meridian-helm-v4-{k}/meridian-helm-v4-{k}-labelled.bin', 'head+hair+jaw', L.SET_V2, L.CUIRASS_V2)
ours('v3a', 'RE', 'v3-a crescent (by eye)', f'{BB}/meridian-helm-v3/meridian-helm-v3-a-labelled.bin', 'head+hair+jaw', L.SET_V3, L.CUIRASS_V3)
ours('v2a', 'RE', 'v2-a closed (by eye)', f'{BB}/meridian-helm-v2/meridian-helm-v2-a-labelled.bin', 'head+hair+jaw', L.SET_V2, L.CUIRASS_V2)

ORDER = ['torva', 'oathplate', 'justiciar', 'faceguard', 'serpentine', 'v5', 'inquisitor', 'masori', 'dharok', 'statius',
         'armadyl', 'umbralA', 'umbralB', 'v4a', 'v4b', 'v4c', 'v3a', 'v2a']
CORE = ['torva', 'oathplate', 'justiciar', 'faceguard', 'serpentine', 'v5']


def add_ours(key, name, path, spec, rest, body, group='NEW'):
    ours(key, group, name, path, spec, rest, body)
    if key not in ORDER: ORDER.append(key)


def entry(key, helm=True):
    h = H[key]
    return (h['helm'] + '+' if helm else '') + h['rest']


def mesh(key):
    k, a = H[key]['mesh']
    return L.mesh_real(a) if k == 'real' else L.mesh_ours(a)


def body_mesh(key):
    k, a = H[key]['body']
    return L.mesh_real(a) if k == 'real' else L.mesh_ours(a)


def draw(tag, entries, yaw=0, zoom=300, cell=(240, 450), aim=None, seq=None, frames=None):
    """-> raw picture (int array), cell width, cell height. Columns = entries."""
    WORK.mkdir(exist_ok=True)
    ex = ['--yaw', yaw, '--zoom', zoom, '--cell', f'{cell[0]}x{cell[1]}']
    if aim: ex += ['--aim', aim]
    if seq: ex += ['--seq', seq, '--frames', frames or '3']
    a = L.pose(WORK / f'{tag}.png', entries, *ex)
    return a


def cells(a, n):
    cw = a.shape[1] // n
    return [a[:, k * cw:(k + 1) * cw] for k in range(n)]


def helm_mask(with_helm, without):
    """pixels the helm changes: the helm as drawn in the outfit (also catches the hair/jaw it removes; the head-only
    aim keeps that harmless: where the helm is absent and the bare head was, the pixel is not background in `without`
    and differs, so we additionally require it to be non-background WITH the helm)."""
    bg = with_helm[2, 2]
    d = np.abs(with_helm - without).sum(2) > 0
    fig = np.abs(with_helm - bg).sum(2) > 0
    return d & fig


def grey(a):
    return a.sum(2) / 3.0

# the test helm of this study (umbral-helm-a2/), on A's stand-in body and on the body recoloured to match
_A2 = (HERE / 'umbral-helm-a2' / 'umbral-helm-a2-labelled.bin').as_posix()
if pathlib.Path(_A2).exists():
    _D = (HERE / 'umbral-helm-a2').as_posix()
    add_ours('umbralA2', 'Umbral A2 (test, A\'s body)', _A2, 'head+hair+jaw', folder_set(UA, '-watch'), f'{UA}/cuirass-watch.bin')
    add_ours('umbralA2m', 'Umbral A2 (test, matched body)', _A2, 'head+hair+jaw', folder_set(_D, '-match'), f'{_D}/cuirass-match.bin')
    H['umbralAm'] = dict(H['umbralA'], name='Umbral A on the matched body', rest=folder_set(_D, '-match'), body=('ours', f'{_D}/cuirass-match.bin'), group='NEW')
    ORDER.append('umbralAm')
