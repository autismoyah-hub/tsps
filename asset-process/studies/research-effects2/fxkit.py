"""fxkit.py: from a softkit mesh (JSON written inside Blender) + a skeleton + a clip to the loose files the harness draws:
a model, a compiled frame base, a sampled sequence, and a definition: `.def` (a spot animation / projectile) or `.loc`
(an OBJECT with a sequence).

  blender(script, *args)                       run a build script inside Blender headless (softkit.py is importable there)
  finish(outdir, name, mesh.json, skeleton, clip, kind='spot'|'loc', ids={seq, base, [spot|loc], model}, **definition)
      skeleton: research-skeleton/skelc.py description (joints own vertex-label NAMES; alpha channels list face-label NAMES)
      clip:     research-skeleton/skelanim.py clip, or a LIST of clips for an object with several sequences
                (each needs its own "name"; the first is the definition's; the others are written to anim-NAME/ for LOC_ANIM)
      definition (spot): scale='128,128', ambient, contrast            (object): ambient, contrast (stored), size='1,1',
                 randomstart=0|1, contour=0 (follow the ground), solid=0
  Names: every vertex label a joint owns or pivots on must exist in the mesh (checked); a mesh label no joint names is written
  as 0 (it never moves). Face labels likewise.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, subprocess, shutil
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
SK = HERE.parent / 'research-skeleton'
sys.path.insert(0, str(SK)); sys.path.insert(1, str(HERE.parent / 'research-npc')); sys.path.insert(2, str(HERE.parent.parent / 'tools'))
import meshio
PY = _AP.fix('<PYTHON>')
BLENDER = _AP.fix('<BLENDER>')


def blender(script, *args):
    r = subprocess.run([BLENDER, '-b', '--python-exit-code', '1', '--python', str(script), '--', *map(str, args)], capture_output=True, text=True)
    out = [l for l in (r.stdout + r.stderr).splitlines() if l.startswith('softkit:') or 'Error' in l or 'Traceback' in l or l.startswith('  File')]
    if r.returncode: raise SystemExit('blender failed:\n' + r.stdout[-3000:] + r.stderr[-3000:])
    return out


def finish(outdir, name, mesh_json, skeleton, clip, kind='spot', ids=None, quiet=False, **defn):
    d = pathlib.Path(outdir); d.mkdir(parents=True, exist_ok=True)
    M = json.loads(pathlib.Path(mesh_json).read_text())
    skeleton = dict(skeleton, name=name, base_id=ids['base'])
    (d / 'skeleton.json').write_text(json.dumps(skeleton, indent=1))
    if (d / 'skel').exists(): shutil.rmtree(d / 'skel')
    r = subprocess.run([PY, '-I', '-B', str(SK / 'skelc.py'), 'compile', str(d / 'skeleton.json'), str(d / 'skel')], capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    warn = [l for l in r.stdout.splitlines() if l.startswith('W ') or ' W ' in l[:6]]
    rig = json.loads((d / 'skel' / 'rig.json').read_text())
    vl = rig['labels']; fl = dict(rig['face_labels'])
    have_v = set(M['vlabels']); have_f = set(M['flabels'])
    for j in skeleton['joints']:
        for n in j.get('own', []) + j.get('pivot', []):
            if n not in have_v: raise SystemExit(f'{name}: joint {j["name"]} names vertex label "{n}", which the mesh does not have (mesh: {sorted(have_v)})')
        for c in j.get('channels', []):
            if isinstance(c, dict) and c.get('kind') == 'alpha':
                for n in c['faces']:
                    if n not in have_f: raise SystemExit(f'{name}: joint {j["name"]} fades face label "{n}", which the mesh does not have (mesh: {sorted(have_f)})')
    nf = len(M['faces'])
    m = dict(vertices=[tuple(v) for v in M['vertices']], faces=[tuple(f) for f in M['faces']], colors=list(M['colors']), priorities=[0] * nf,
             vertex_skins=[vl.get(n, 0) for n in M['vlabels']], face_skins=[fl.get(n, 0) for n in M['flabels']], render_types=[1 if f else 0 for f in M['flat']], alpha=list(M['alpha']))
    meshio.save(m, d / f'{name}.bin')
    clips = clip if isinstance(clip, list) else [clip]
    lines = []
    for k, c in enumerate(clips):
        sid = ids['seq'] + k
        c = dict(c, rig='skel/rig.json', id=sid, frame_group=sid)
        cname = c.get('name', name)
        ad = d / ('anim' if k == 0 else f'anim-{cname}')
        if ad.exists(): shutil.rmtree(ad)
        (d / f'{name}{"" if k == 0 else "-" + cname}.anim.json').write_text(json.dumps(c, indent=1))
        r2 = subprocess.run([PY, '-I', '-B', str(SK / 'skelanim.py'), str(d / f'{name}{"" if k == 0 else "-" + cname}.anim.json'), str(ad)], capture_output=True, text=True)
        if r2.returncode: raise SystemExit(r2.stdout + r2.stderr)
        lines.append(r2.stdout.splitlines()[0] if r2.stdout else '')
        warn += [l for l in r2.stdout.splitlines() if 'BROKEN' in l]
    what = defn.pop('what', name)
    if kind == 'spot':
        L = [f'# {name}: {what}', f'name={name}', f'model={name}.bin', 'anim=anim'] + [f'{a}={b}' for a, b in defn.items()]
        (d / f'{name}.def').write_text('\n'.join(L) + '\n')
    else:
        L = [f'# {name}: {what}', f'name={name}', f'model={name}.bin', 'anim=anim'] + [f'{a}={b}' for a, b in defn.items()]
        (d / f'{name}.loc').write_text('\n'.join(L) + '\n')
    (d / 'ids.json').write_text(json.dumps(dict(ids, kind=kind, sequences={c.get('name', name): ids['seq'] + k for k, c in enumerate(clips)})))
    if not quiet:
        print(f'{name}: {len(m["vertices"])} vertices {nf} faces, translucent {sum(1 for a in m["alpha"] if 0 < a < 255)}, smooth {sum(1 for f in M["flat"] if not f)}; groups {rig["groups"]}; {"; ".join(lines)}')
        for w in warn: print('   ', w)
    return d / (f'{name}.def' if kind == 'spot' else f'{name}.loc')
