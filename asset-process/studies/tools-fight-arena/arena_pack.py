"""Pack a composed court into a fresh Warden fight cache. Use --films for playback and parity."""
import argparse
import csv
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
D = Path(__file__).resolve().parent
ROOT = D.parents[1]
FX = D.parent / 'research-effects2'
WARDEN = D.parent / 'fx-warden-general'
sys.path.insert(0, str(FX))
import fxpack as pack
import objlib


def pose(board, view):
    scope = (board / ('scene-' + view) / view / 'scope.txt').read_text()
    line = next(s for s in scope.splitlines() if s.startswith('pose='))
    return list(map(float, re.findall(r'-?\d+(?:\.\d+)?', line)))


def build(arena, out, revision):
    board = arena / 'boards' / revision
    scene = board / 'scene-played'
    floor = json.loads((board / 'floor.json').read_text())
    source = WARDEN / 'cache-own'
    cache = pack.A.Cache(source)
    fxcache = pack.A.Cache(arena / 'effect-cache')
    if out.exists():
        raise ValueError(f'Choose a fresh output cache: {out}')
    work = out.with_name(out.name + '-inputs')
    work.mkdir(parents=True)
    package = scene / 'package'
    files = {}
    for line in (package / 'registry.txt').read_text().splitlines()[1:]:
        kind, ident, path, crc = line.split()
        payload = package / path
        assert pack.zlib.crc32(payload.read_bytes()) == int(crc,16), payload
        files[kind, int(ident)] = payload
    content = dict(s.split('=', 1) for s in (scene/'content.txt').read_text().splitlines()[1:])
    bwr = (scene / content['floor']).read_bytes()
    assert hashlib.sha256(bwr).hexdigest() == content['floor_sha256']
    tiles, placements = {}, []
    for line in bwr.decode().splitlines()[1:]:
        kind, *v = line.split()
        if kind == 'V':
            x, y, h, underlay, walk = map(int, v)
            if x < 64 and y < 64:
                assert h % 8 == 0 and h // 8 != 1
                tiles[0,x,y] = dict(height=h//8, underlay=underlay, flags=0 if walk else 1)
            else:
                assert not (h or underlay or walk), 'court spans multiple regions'
        elif kind == 'O':
            placements.append(tuple(map(int, v)))
    groups = cache.reference(5)['groups']
    rx, ry = next((x,y) for x in range(16,120) for y in range(16,120)
                  if all(((x+i)*256 + y+j) not in groups for i in range(-1,2) for j in range(-1,2)))
    rid = rx*256 + ry
    rows = [r for r in csv.DictReader((board/'placements.csv').open()) if not r['source'].startswith('terrain:')]
    manifest = {k:{} for k in ('models','objects','bases','sequences','maps')}
    # authored_floor.rs COLORS are mounted by the room renderer at config IDs 5801..5807.
    colors = ['68685a','55564e','3e4444','655b48','495c4b','2c505b','282d2d']
    manifest['underlays'] = {u-1:{'rgb':colors[u-5802]} for u in {t['underlay'] for t in tiles.values()} if u}
    effects = set()
    locations = []
    blocked = set()
    for r, (oid,x,y,shape,rot) in zip(rows,placements,strict=True):
        assert (x,y,shape,rot) == tuple(int(r[k]) for k in ('tile_x','tile_z','shape','rotation'))
        assert 0 <= x < 64 and 0 <= y < 64
        data = files['object',oid].read_bytes()
        obj = objlib.decode(data)
        manifest['objects'][oid] = {'hex':data.hex()}
        for mid in obj['models']:
            manifest['models'][mid] = str(files['model',mid])
        if r['func'] == 'boss':
            continue
        if obj['anim'] is not None:
            effects.add(obj['anim'])
        if shape != 22:
            w,h = obj['size']
            if rot%2: w,h = h,w
            blocked.update((x+i,y+j) for i in range(w) for j in range(h))
        locations.append((oid,0,x,y,shape,rot))
    for sid in sorted(effects):
        folder = work / f'seq-{sid}'
        folder.mkdir(exist_ok=True)
        seq = fxcache.file(2,12,sid)
        (folder/'seq.dat').write_bytes(seq)
        manifest['sequences'][sid] = str(folder)
        for fid in set(pack.A.decode_sequence(seq)['frames']):
            g,f = divmod(fid,65536)
            data = fxcache.file(0,g,f)
            (folder/f'frame_{g}_{f}.dat').write_bytes(data)
            base = int.from_bytes(data[:2],'big')
            path = work / f'base_{base}.dat'
            path.write_bytes(fxcache.file(1,base,0))
            manifest['bases'][base] = str(path)
    terrain, loc = work/'terrain.bin', work/'locations.bin'
    terrain.write_bytes(pack.encode_terrain(tiles,height=0))
    loc.write_bytes(pack.encode_locations(locations))
    manifest['maps'][rid] = dict(terrain=str(terrain),locations=str(loc))
    path = work/'manifest.json'
    path.write_text(json.dumps(manifest,indent=1))
    subprocess.run([sys.executable,'-I',str(FX/'fxpack.py'),source,out,path],check=True)
    fight = json.loads((WARDEN/'fight.json').read_text())
    for asset in fight['assets'].values():
        if 'file' in asset:
            asset['file'] = (WARDEN/asset['file']).as_posix()
    bx,by,w,h = floor['boss']
    px = bx + (w-1)//2
    assert px == floor['entries'][0][0] and w == h == fight['stage']['npc_size']
    offsets = {(0,0)} | {tuple(b['tile']) for a in fight['attacks'].values() for b in a['beats'] if isinstance(b.get('tile'),list)}
    # Keep a full tile inside the reachable entry. At the entry the piers mask the warning row.
    max_distance = min(fight['stage']['distance'], by - floor['entries'][0][1] - 1)
    distance = next(d for d in range(max_distance,0,-1)
                    if all(tiles[0,px+dx,by-d+dy]['underlay'] and (px+dx,by-d+dy) not in blocked for dx,dy in offsets))
    camera = pose(board,'played')
    camera[0] += rx*64
    camera[1] += ry*64
    camera[2:] = [700,1800,1900,340]
    fight['stage'].update(distance=distance,place=[rx*64+px+1,ry*64+by-distance+distance//2],
                          camera=camera, size=[608,432], focal=670, region=[rx,ry])
    fight.update(cache=out.as_posix(), board=board.as_posix(), output=(ROOT/'render'/('fight-arena-'+arena.name)).as_posix())
    (D/'fight-arena.json').write_text(json.dumps(fight,indent=2))
    return fight


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('arena',type=Path)
    p.add_argument('out',type=Path)
    p.add_argument('--board',choices=['final','final2'])
    p.add_argument('--films',action='store_true')
    a = p.parse_args()
    rev = a.board or ('final2' if (a.arena/'boards/final2').is_dir() else 'final')
    build(a.arena.resolve(),a.out.resolve(),rev)
    if a.films:
        subprocess.run([sys.executable,'-I',D/'proof.py'],check=True)
        subprocess.run([sys.executable,'-I',D/'play_arena.py'],check=True)

