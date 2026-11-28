"""Acceptance checks for rebuilt scenery, original scenery and worn equipment.

Placement shape (rebuild-object, original-object): 10 = ordinary object standing on the floor (the default, and
unchanged), 4 = wall decoration, 22 = floor decoration.
  rebuild-object REAL_ID MODEL --ambient A --contrast C [--shape N]
      Without --shape the shape is read from the real object: 4 or 22 when the archive holds it as a wall or
      floor decoration, otherwise 10. With shape 4 / 22 both pieces are mounted with that shape in the line-up,
      the camera is aimed at the middle of the real piece as mounted, and two measures are added: where the piece
      sits (each end of each axis against the real one) and the drawn outline as mounted.
  original-object MODEL --faces LO,HI --size X0,X1,Y0,Y1,Z0,Z1 [--shape N] [--bottom LO,HI] [--wall-plane X]
      There is no real object to read, so the task line states the shape. For 4 / 22 the floor-standing check
      (base height 0) is replaced by bands measured on real pieces of that shape: tools/decor_bands.json, made by
      tools/decor_bands.py. --size gives extents only, so --bottom LO,HI states where the lowest point must be
      (how high a wall piece hangs). --wall-plane is the x of the wall surface the piece hangs on, in the model's own frame
      (default -64, the tile's west edge; a room whose masonry stands proud of the edge states its own).
"""
import argparse
import collections
import contextlib
import io
import json
from pathlib import Path
import re
import sys
import tempfile

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from handoff import ROOT, TOOLS, decode, entry, osrscache, run
import compare
import numpy as np
from PIL import Image

LEVELS = ['PASS', 'BORDERLINE', 'FAIL']
ALLOWED = dict(head=[5, 6, 7, 8], body=[2, 3, 4], legs=[1, 2], hands=[10], feet=[0], weapon=[10])


class Checks:
    def __init__(self):
        self.rows = []

    def record(self, name, value, threshold, verdict):
        self.rows.append(dict(name=name, value=value, threshold=threshold, verdict=verdict))
        print(f'{name} | {json.dumps(value)} | {threshold} | {verdict}', flush=True)

    def info(self, name, value):
        self.record(name, value, 'informational', 'INFO')

    def add(self, name, value, threshold, passed=True, borderline=False):
        self.record(name, value, threshold, LEVELS[0 if passed else 1 if borderline else 2])

    def limit(self, name, value, good, edge, provisional=False):
        prefix = 'provisional: ' if provisional else ''
        self.add(name, value, f'{prefix}PASS <= {good}, BORDERLINE <= {edge}', value <= good, value <= edge)

    def band(self, name, value, entry, population):
        """A limit measured on real pieces (tools/decor_bands.json): limits = [edge low, pass low, pass high,
        edge high]; the two low limits are null for a measure that is only limited from above."""
        e_lo, p_lo, p_hi, e_hi = entry['limits']
        source = f'real {entry["pct"]} of {population}'
        if value is None:
            self.add(name, value, source, False)
        elif p_lo is None:
            self.add(name, value, f'PASS <= {p_hi:g}, BORDERLINE <= {e_hi:g} ({source})', value <= p_hi, value <= e_hi)
        else:
            self.add(name, value, f'PASS {p_lo:g}..{p_hi:g}, BORDERLINE {e_lo:g}..{e_hi:g} ({source})',
                     p_lo <= value <= p_hi, e_lo <= value <= e_hi)

    def interval(self, name, value, bounds):
        lo, hi = bounds
        if hi < lo:
            raise ValueError(f'{name}: reversed interval')
        self.add(name, value, f'{lo}..{hi} inclusive', lo <= value <= hi)


def integers(value, length):
    values = list(map(int, value.split(',')))
    if len(values) != length:
        raise ValueError(f'expected {length} comma-separated integers, got {value}')
    return values


def mounted(a, m, checks):
    """A wall decoration (shape 4) or floor decoration (shape 22) against real pieces of its shape, in the frame
    the game mounts it in at rotation 0. Replaces 'base height 0', which is only right for a floor-standing piece.
    Returns (measures, population) for the checks original() makes further down."""
    bands = json.loads((TOOLS/'decor_bands.json').read_text())[str(a.shape)]
    real, population = bands['measures'], f'{bands["n"]} {bands["population"]}'
    xs, ys, zs = zip(*m['vertices'])
    low, high = -max(ys), -min(ys)
    checks.info('placement shape', a.shape)
    if a.bottom:
        checks.interval('lowest point, task line', low, integers(a.bottom, 2))
    if a.shape == 4:
        wall = a.wall_plane
        checks.band('lowest point above the floor', low, real['low'], population)
        checks.band('highest point above the floor', high, real['high'], population)
        checks.band(f'back of the piece from the wall plane x = {wall} (negative = sunk into the wall)', min(xs) - wall, real['gap'], population)
        checks.band(f'front of the piece from the wall plane x = {wall} (how far proud)', max(xs) - wall, real['proud'], population)
        checks.band('thickness (x)', max(xs) - min(xs), real['thick'], population)
        checks.band('width along the wall (z)', max(zs) - min(zs), real['width'], population)
        checks.band('middle of the piece along the wall (z)', (min(zs) + max(zs)) / 2, real['zc'], population)
    else:
        checks.band('lowest point above the ground', low, real['low'], population)
        checks.band('highest point above the ground', high, real['high'],
                    population + '; lower limit 1: a surface at height 0 is not drawn on our terrain')
        checks.band('footprint x', max(xs) - min(xs), real['ext'], population)
        checks.band('footprint z', max(zs) - min(zs), real['ext'], population)
        checks.band('overhang past the tile edge', max(0, -64 - min(xs), max(xs) - 64, -64 - min(zs), max(zs) - 64), real['over'], population)
    checks.band('face count against real pieces of this shape', len(m['faces']), real['faces'], population)
    checks.band('distinct colours', len(set(m['colors'])), real['colours'], population)
    return real, population


def original(a, m, checks):
    vertices, faces = m['vertices'], m['faces']
    duplicates = len(vertices) - len(set(vertices))
    checks.add('duplicate vertices', duplicates, '0', duplicates == 0)
    if a.shape == 10:
        base = -max(v[1] for v in vertices)
        checks.add('base height', base, '0', base == 0)
        real = None
    else:
        real, population = mounted(a, m, checks)
    checks.interval('faces', len(faces), integers(a.faces, 2))
    limits = integers(a.size, 6)
    for i, coords in enumerate(zip(*vertices)):
        checks.interval('size ' + 'xyz'[i], max(coords) - min(coords), limits[2*i:2*i+2])
    areas = [compare.area(vertices, face) for face in faces]
    material_areas = collections.Counter()
    for colour, area in zip(m['colors'], areas):
        material_areas[colour >> 7] += area
    total_area = sum(areas)
    checks.info('all hue/saturation pairs', len(material_areas))
    main = sum(area > 0 and area >= .03 * total_area for area in material_areas.values())
    if real:
        checks.band('main materials (at least 3% of surface each)', main, real['main'], population)
    else:
        checks.limit('main materials (at least 3% of surface each)', main, 6, 8)
    degenerate = areas.count(0)
    checks.add('zero-area triangles', degenerate, '0', degenerate == 0)
    flat = m['render_types'].count(1) / len(faces)
    if a.shape == 22:
        checks.info('flat face share (real floor decorations are often all flat-shaded)', flat)
    else:
        checks.add('flat face share', flat, '< 1', flat < 1)
    share = max(areas) / sum(areas) if sum(areas) else None
    if real:
        checks.band('largest triangle surface share', share, real['largest'], population)
    else:
        checks.add('largest triangle surface share', share, '<= 0.25', share is not None and share <= .25)


def compare_mesh(model, m, checks, real_id, kind, shape=None):
    with contextlib.redirect_stdout(io.StringIO()):
        rows, outlines = compare.report(real_id, model, kind, shape)
        real = compare.real_item(real_id) if kind == 'item' else compare.real_model(real_id, shape)
    metrics = {name: (r, u) for name, r, u in rows}
    for axis, coords, ref in zip('xyz', zip(*m['vertices']), zip(*real['vertices'])):
        checks.limit('size ' + axis + ' absolute error', abs((max(coords)-min(coords)) - (max(ref)-min(ref))), 1, 2)
    r, u = metrics['faces']
    checks.limit(f'face count error % (ours {u}, real {r})', 100 * abs(u-r) / r, 10, 15)
    r, u = metrics['colours']
    cap = max(r + 1, -(-r * 3 // 2))    # Teacher 2026-11-26: real + 1 failed a sword rebuild not told from the real one (11 against 8)
    checks.add('distinct colours', u, f'PASS <= {cap} (real + 1 or half as many again), BORDERLINE <= {2*r}', u <= cap, u <= 2*r)
    checks.limit('colour distance', compare.colour_distance(compare.colour_shares(m), compare.colour_shares(real)), 2.5, 3.5)
    good, edge = (.93, .90) if kind == 'item' else (.97, .95)
    for view, overlap in outlines.items():
        checks.add(view + ' outline', float(overlap), f'PASS >= {good}, BORDERLINE >= {edge}', overlap >= good, overlap >= edge)
    return real


def placement_shape(a):
    """10 (ordinary object) unless --shape says otherwise or the archive holds the real object as a wall
    decoration (folder shape-4) or a floor decoration (shape-22)."""
    if a.shape is not None:
        return a.shape, '--shape'
    with contextlib.redirect_stdout(io.StringIO()):
        folder = compare.scene.object_json(a.real_id, quiet=True)['shape_folder']
    return (folder if folder in (4, 22) else 10), f'read from the real object: archive folder shape-{folder}'


def framing(real, shape):
    """Line-up options that put the middle of the real piece, as mounted, in the middle of every view."""
    lo, hi = ([f(c) for c in zip(*real['vertices'])] for f in (min, max))
    cx, cy, cz = ((l + h) / 2 for l, h in zip(lo, hi))
    # lineup.py mounts shape 4 at rotation 1 (the tile's north edge, facing the front view): model (x, z) -> (z, -x)
    east, north = (cz, -cx) if shape == 4 else (cx, cz)
    radius = sum((h - l) ** 2 for l, h in zip(lo, hi)) ** .5 / 2
    back = max(1, radius / 105)             # the default camera holds a piece of up to about this radius in its window
    # A wall decoration is only ever seen from the room: the usual turned views (135, 225, 180 degrees) would show
    # its back, which is inside the wall, and three views of nothing would dilute the front in the pairs mean.
    views = ['--yaws', '0,45,315,60'] if shape == 4 else []
    return [f'--aim={east:.1f},{north:.1f}', '--up', round(min(650, max(0, -cy))),
            '--dist', round(330 * back), '--elev', round(250 * back), *views]


def rebuild(a, m, checks):
    shape, why = placement_shape(a)
    if shape == 10:
        compare_mesh(a.model, m, checks, a.real_id, 'object')
        entries, options = f'{a.real_id},{a.model.resolve()}', []
    else:
        checks.info('placement shape', f'{shape} ({why})')
        real = compare_mesh(a.model, m, checks, a.real_id, 'object', shape)
        # the outlines above are centred on each piece's own bounds; a mounted piece must also SIT where the real one does
        for axis, coords, ref in zip('xyz', zip(*m['vertices']), zip(*real['vertices'])):
            checks.limit(f'position {axis}: larger error of the two ends, as mounted',
                         max(abs(min(coords) - min(ref)), abs(max(coords) - max(ref))), 1, 2)
        entries, options = f'{a.real_id}@{shape},{a.model.resolve()}@{shape}', framing(real, shape)
    with tempfile.TemporaryDirectory(dir=ROOT/'render', prefix='gate-') as work:
        folder = Path(work)/'lineup'
        run(TOOLS/'lineup.py', folder, entries, '--views', 4,
            '--ambient', a.ambient, '--contrast', a.contrast, *options)
        output = run(ROOT/'author-pack/tools/pairs.py', folder)
        if shape != 10:
            counts = re.findall(r'outline: (\d+) px only real, (\d+) px only ours \(of (\d+) shared\)', output)
            checks.info('line-up camera', ' '.join(map(str, options)))
            checks.info('pixels drawn by both, by view', [int(s) for _, _, s in counts])
            drawn = [int(s) / max(int(s) + int(r) + int(u), 1) for r, u, s in counts]
            worst = min(drawn) if len(drawn) == 4 else 0.0
            checks.add('drawn outline as mounted, worst of four views', worst, 'PASS >= 0.97, BORDERLINE >= 0.95', worst >= .97, worst >= .95)
        if len(re.findall(r'^view \d+:.*\(of [1-9]\d* shared\)', output, re.M)) != 4:
            raise ValueError('pairs needs four views with shared object pixels')
        score = float(re.search(r'^mean score ([\d.]+)', output, re.M)[1])
        # pairs prints only two decimals. Recover the unrounded value if that
        # rounding could change the verdict at either inclusive boundary.
        if score in (3.0, 4.0):
            sys.path.insert(0, str(ROOT/'author-pack/tools'))
            import pairs
            with Image.open(folder/'lineup.png') as board:
                crop = board.width // 2
            scores = []
            for view in range(4):
                windows = []
                for index in range(2):
                    image = pairs.load(folder/f'e{index}v{view}.png')
                    h, w = image.shape[:2]
                    x, y = (w-crop)//2, (h-crop)//2
                    windows.append(image[y:y+crop, x:x+crop])
                scores.append(pairs.compare(*windows)[1]['mean_abs'])
            score = sum(scores)/4
        if shape == 4:
            # The limits 3 / 4 were set on furniture seen from all sides. Here all four views are of the front,
            # so nothing unseen dilutes the mean: provisional until more wall rebuilds have been judged by eye.
            checks.limit('pairs mean (four views from the room side: 0, 45, 315, 60 degrees)', score, 3, 4)
        else:
            checks.limit('pairs mean (four views)', score, 3, 4)


def item_spec(value):
    if not re.fullmatch(r'item:\d+', value):
        raise argparse.ArgumentTypeError('expected item:ID')
    return value


def weapon(a, m, checks):
    compare_mesh(a.model, m, checks, int(a.real[5:]), 'item')
    for name, stream, required, count in [
        ('priority', m['priorities'], 10, len(m['faces'])),
        ('vertex label', m['vertex_skins'], 50, len(m['vertices'])),
        ('face label', m['face_skins'] if m['face_labels_present'] else None, 29, len(m['faces'])),
    ]:
        matches = (stream or []).count(required)
        checks.add(name, matches, f'all {count} entries equal {required}', matches == count)


def pose_diff(checks, key, output, mask, count):
    matches = re.findall(r'^diff col 1 vs col 3 frame (\d+): (\d+) of', output, re.M)
    if [int(f) for f, _ in matches] != list(range(count)):
        raise ValueError('incomplete pose diff')
    diffs = [int(n) for _, n in matches]
    sizes = [int(frame.sum()) for frame in np.vsplit(mask, count)]
    checks.info(key + ' diff px by frame', diffs)
    checks.info(key + ' own worn px by frame', sizes)
    checks.info(key + ' diff / own worn px by frame', [d / n if n else None for d, n in zip(diffs, sizes)])


def motion(a, checks, slot):
    entries = [entry(a.model, slot), 'none']
    target = a.reference or a.real
    if target:
        entries.append(target)
    total_changed = total_frames = 0
    with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parent, prefix='gate-') as work:
        for seq, count in [(819, 8), (824, 8), (390, 10)]:
            frames = ','.join(map(str, range(count)))
            for yaw in [192, 1216]:
                rasters = []
                key = f'seq {seq} yaw {yaw}'
                for mode in ['actor', 'depth']:
                    png = Path(work)/(mode + '.png')
                    options = ['--diff', '1:3'] if a.real else []
                    out = run(TOOLS/'pose.py', png, ','.join(entries), '--seq', seq, '--frames', frames,
                              '--yaw', yaw, '--draw', mode, '--keep-bmp', *options)
                    if target and not re.search(rf'^{target}: item \d+ .* in {slot.split("+")[0]} \(', out, re.M):
                        raise ValueError(f'{target} must occupy the primary slot of {slot}')
                    with Image.open(png.with_suffix('.bmp')) as image:
                        columns = np.hsplit(np.asarray(image.convert('RGB')), len(entries))
                    rasters.append(columns)
                    if a.real:
                        pose_diff(checks, f'{key} {mode} vs {target}', out,
                                  np.any(columns[0] != columns[1], axis=2), count)
                actor, depth = rasters

                def differing_mean(i):
                    worn = np.any(actor[i] != actor[1], axis=2) | np.any(depth[i] != depth[1], axis=2)
                    return int(np.count_nonzero(np.any(actor[i] != depth[i], axis=2) & worn)) / count

                ours = differing_mean(0)
                checks.info(key + ' mean differing worn px/frame', ours)
                total_changed += ours * count
                total_frames += count
                if a.reference:
                    ref = differing_mean(2)
                    checks.info(f'{key} {target} mean differing worn px/frame', ref)
                    name = key + ' motion ratio ours/reference'
                    # Teacher 2026-11-26: INFO only. The ratio failed legs the owner graded Strong and approved in motion
                    # (a reference near zero pixels makes any ratio huge). Not a verdict until calibrated on more pieces.
                    checks.info(name, (ours / ref) if ref else None)
    checks.info(f'all {total_frames} views of frames, mean differing worn px/frame', total_changed / total_frames)


def worn(a, m, checks):
    checks.add('decode', m['format'], 'nonempty mesh, valid triangle indices')
    names = 'head cape amulet weapon body shield arms legs hair hands feet jaw'.split()
    aliases = {str(index): name for index, name in enumerate(names)}
    slots = [aliases.get(token, token) for token in a.slot.lstrip('@').split('+')]
    if slots[0] not in ALLOWED or any(s not in names + ['nohide'] for s in slots[1:]):
        raise ValueError(f'unsupported slot specification {a.slot}')
    slot = '+'.join(slots)
    primary, *hidden = slot.split('+')
    allowed = ALLOWED[primary] + ([10] if primary == 'body' and 'arms' in hidden else [])
    used = sorted(set(m['priorities']))
    checks.add('priorities', used, f'subset of {allowed}', set(used) <= set(allowed))
    for kind, stream, total in [('vertex', m['vertex_skins'], len(m['vertices'])),
                                ('face', m['face_skins'] if m['face_labels_present'] else None, len(m['faces']))]:
        n = len(stream or [])
        checks.add(kind + ' labels', n, f'{total} explicit entries', n == total)
    if a.steel:
        steel = tuple(integers(a.steel, 2))
        exempt = {(4, 3, 70), *(tuple(integers(c, 3)) for c in a.non_steel)}
        bad = collections.Counter()
        for value in m['colors']:
            h, s, l = osrscache.hsl_split(value)
            if s <= 1 and 20 <= l <= 120 and (h,s) != steel and (h,s,l) not in exempt:
                bad[f'{h},{s},{l}'] += 1
        checks.add('offending H,S,L and face counts', dict(bad), f'0 outside steel {steel} or exact exemptions', not bad)
    if a.real:
        compare_mesh(a.model, m, checks, int(a.real[5:]), 'item')
    motion(a, checks, slot)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--json', type=Path)
    modes = parser.add_subparsers(required=True)
    for name, function in [('rebuild-object', rebuild), ('original-object', original), ('worn', worn), ('rebuild-weapon', weapon)]:
        p = modes.add_parser(name)
        p.set_defaults(function=function)
        p.add_argument('--json', type=Path, default=argparse.SUPPRESS)
        if function == rebuild:
            p.add_argument('real_id', type=int)
            for opt in ['--ambient', '--contrast']:
                p.add_argument(opt, required=True, type=int)
            p.add_argument('--shape', type=int, choices=[4, 10, 22], default=None,
                           help='placement shape; default: read from the real object (4 or 22 if the archive holds it so, else 10)')
        if function == weapon:
            p.add_argument('real', type=item_spec, metavar='item:ID')
        p.add_argument('model', type=Path)
        if function == original:
            p.add_argument('--faces', required=True)
            p.add_argument('--size', required=True)
            p.add_argument('--shape', type=int, choices=[4, 10, 22], default=10,
                           help='placement shape: 10 ordinary object (default), 4 wall decoration, 22 floor decoration')
            p.add_argument('--bottom', help='shape 4 / 22: LO,HI the task line allows for the lowest point (the hanging height of a wall piece)')
            p.add_argument('--wall-plane', type=int, default=-64,
                           help='shape 4: x of the wall surface the piece hangs on, in the model frame (default -64, the tile edge)')
        if function == worn:
            p.add_argument('slot')
            target = p.add_mutually_exclusive_group()
            target.add_argument('--real', type=item_spec)
            target.add_argument('--reference', type=item_spec)
            p.add_argument('--steel')
            p.add_argument('--non-steel', action='append', default=[], metavar='H,S,L')
    a = parser.parse_args()
    checks = Checks()
    try:
        model = decode(a.model)
        if a.function in (original, rebuild):
            pop = f'shape-{a.shape if a.shape is not None else placement_shape(a)[0]}'
        else:
            ref = getattr(a,'real',None) or getattr(a,'reference',None)
            if ref:
                pop = next((compare.scene.ARCH/'items').glob(f'*/{ref[5:]}.json')).parent.name
            else:
                slot = {'head':0,'cape':1,'amulet':2,'weapon':3,'body':4,'shield':5,
                        'legs':7,'hands':9,'feet':10}.get(a.slot.lstrip('@').split('+')[0])
                pop = f'slot-{slot}' if slot is not None else 'worn'
        checks.info('cache materials',compare.material_info(model,pop))
        a.function(a, model, checks)
    except (Exception, SystemExit) as error:
        checks.add('measurement error', f'{type(error).__name__}: {error}', 'all measurements complete', False)
    code = max(LEVELS.index(row['verdict']) for row in checks.rows if row['verdict'] != 'INFO')
    if a.json:
        a.json.write_text(json.dumps(dict(checks=checks.rows, verdict=LEVELS[code]), indent=2)+'\n')
    print('VERDICT', LEVELS[code])
    return code


if __name__ == '__main__':
    sys.exit(main())
