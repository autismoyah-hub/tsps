"""kitproof.py: the soft-mass kit proven piece by piece against a real effect of the same kind.
usage: kitproof.py build | draw | measure | all
  build    Blender (kitproof_bl.py) -> mesh/*.json; skeletons and clips below -> cloud/ orb/ tail/ dust/ pool/ (loose files)
  draw     render/research-effects2-kit/NAME-pair.gif (real | kit, game timing, close and game size), NAME-pair.png, and
           NAME-blind.gif + NAME-blind.key.txt (unlabelled, order shuffled)
  measure  soft.py numbers for both of each pair -> data/kitproof.txt
"""
import sys, pathlib, json, random
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent; sys.path.insert(0, str(HERE.parent))
import fxkit, fx, soft
from fxlib import RENDER
from PIL import Image

OUT = RENDER / 'research-effects2-kit'
W = lambda ch, amp, period, phase=0.0, axes=(0, 1, 2): [dict(channel=ch, axis=a, amp=amp, period=period, phase=phase) for a in axes]


def build():
    for l in fxkit.blender(HERE / 'kitproof_bl.py'): print(l)
    # cloud: three sets breathe about the hub, out of step; an OBJECT 3 x 3 that loops
    sk = dict(order='child-first', joints=[dict(name=f's{i}', parent='root', pivot=['hub'], own=[f'c{i}'], channels=['scale']) for i in range(3)] + [dict(name='root', parent=None, pivot=['hub'], own=['hub'], channels=['scale'])])
    clip = dict(name='cloud_idle', frames=17, hold=3, loop=True, sequence=dict(loop_frames=17), keys=[dict(t=0, pose={f's{i}.scale': [128, 128, 128] for i in range(3)})],
                waves=sum((W(f's{i}.scale', 13, 51, i / 3) for i in range(3)), []))
    fxkit.finish(HERE / 'cloud', 'cloud', HERE / 'mesh' / 'cloud.json', sk, clip, kind='loc', ids=dict(base=2720, seq=20200), what='kit proof: lump_cloud', ambient=0, contrast=0, size='3,3', randomstart=0)
    # orb: pulse of the shells, flicker of the outer shell's alpha; a projectile that loops
    sk = dict(order='child-first', joints=[dict(name='orb', parent=None, pivot=['core'], own=['core'], channels=['scale', 'turn', dict(kind='alpha', name='outer', faces=['g3']), dict(kind='alpha', name='mid', faces=['g2'])])])
    clip = dict(name='orb_travel', frames=30, hold=2, loop=True, sequence=dict(loop_frames=30), keys=[dict(t=0, pose={'orb.scale': [128, 128, 128], 'orb.turn': [0, 0, 0]}), dict(t=60, pose={'orb.turn': [0, 0, 256]})],
                waves=W('orb.scale', 14, 20) + [dict(channel='orb.outer', axis=0, amp=3, period=20, phase=0.5), dict(channel='orb.mid', axis=0, amp=2, period=30)])
    fxkit.finish(HERE / 'orb', 'orb', HERE / 'mesh' / 'orb.json', sk, clip, ids=dict(base=2721, seq=20201), what='kit proof: glow_shell', scale='128,128', ambient=0, contrast=0)
    # tail: the head pulses; three sets of tongues stretch along z out of step and flicker
    sk = dict(order='child-first', joints=[dict(name=f't{i}', parent='root', pivot=['core'], own=[f't{i}'], channels=['scale', dict(kind='alpha', faces=[f't{i}'])]) for i in range(3)] +
              [dict(name='root', parent=None, pivot=['core'], own=['core'], channels=['scale', 'turn'])])
    clip = dict(name='tail_travel', frames=16, hold=2, loop=True, sequence=dict(loop_frames=16), keys=[dict(t=0, pose={'root.turn': [0, 0, 0], **{f't{i}.scale': [128, 128, 128] for i in range(3)}}), dict(t=32, pose={'root.turn': [0, 0, 256]})],
                waves=sum(([dict(channel=f't{i}.scale', axis=2, amp=34, period=16, phase=i / 3), dict(channel=f't{i}.alpha', axis=0, amp=3, period=16, phase=i / 3 + 0.5)] for i in range(3)), []) + W('root.scale', 8, 16))
    fxkit.finish(HERE / 'tail', 'tail', HERE / 'mesh' / 'tail.json', sk, clip, ids=dict(base=2722, seq=20202), what='kit proof: streaks + glow_shell', scale='128,128', ambient=10, contrast=40)
    # dust: sectors thrown outward by a scale about the hub, each on its own beat; opaque at first, fading from 45% of the life
    joints = []; pose0 = {}; keys = []
    for name, sets, far in (('d', 4, 400), ('e', 2, 330)):
        for i in range(sets):
            j = f'{name}{i}'
            joints.append(dict(name=j, parent='root', pivot=['hub'], own=[j], channels=['scale', 'move', dict(kind='alpha', faces=[j])]))
            t0 = 3 * i if name == 'd' else 6 + 6 * i
            pose0.update({f'{j}.scale': [36, 60, 36], f'{j}.alpha': [32, 0, 0], f'{j}.move': [0, 0, 0]})
            keys += [dict(t=t0, ease='hold', pose={f'{j}.alpha': [0, 0, 0]}), dict(t=t0, pose={f'{j}.scale': [36, 60, 36]}),
                     dict(t=t0 + 24, ease='out', pose={f'{j}.scale': [int(far * 0.62), 230, int(far * 0.62)]}),
                     dict(t=38 + 2 * i, pose={f'{j}.alpha': [0, 0, 0]}),
                     dict(t=84, ease='out', pose={f'{j}.scale': [far, 330, far], f'{j}.move': [0, -26, 0]}),
                     dict(t=80, pose={f'{j}.alpha': [31, 0, 0]}), dict(t=84, pose={f'{j}.alpha': [32, 0, 0]})]
    joints.append(dict(name='root', parent=None, pivot=['hub'], own=['hub'], channels=['scale']))
    clip = dict(name='dust_slam', frames=28, hold=3, keys=[dict(t=0, pose=pose0)] + keys)
    fxkit.finish(HERE / 'dust', 'dust', HERE / 'mesh' / 'dust.json', dict(order='child-first', joints=joints), clip, ids=dict(base=2723, seq=20203), what='kit proof: dust_burst', scale='128,128', ambient=10, contrast=100)
    # pool: an OBJECT: spawn (20 frames) then an idle of 8 frames that loops (loop_frames 8), as the real blood pool's sequence
    sk = dict(order='child-first', joints=[
        dict(name='rip0', parent='root', pivot=['hub'], own=['pool_rip0'], channels=['scale', dict(kind='alpha', faces=['pool_rip0'])]),
        dict(name='rip1', parent='root', pivot=['hub'], own=['pool_rip1'], channels=['scale', dict(kind='alpha', faces=['pool_rip1'])]),
        *[dict(name=f'bub{i}', parent='root', pivot=[f'pool_bub{i}'], own=[f'pool_bub{i}'], channels=['scale', dict(kind='alpha', faces=[f'pool_bub{i}'])]) for i in range(3)],
        dict(name='root', parent=None, pivot=['hub'], own=['pool', 'hub'], channels=['scale', dict(kind='alpha', name='skirt', faces=['pool_skirt']), dict(kind='alpha', name='fade', faces=['pool', 'pool_skirt'])])])
    lengths = [3] * 20 + [4] * 8      # 60 + 32 cycles
    keys = [dict(t=0, pose={'root.scale': [14, 128, 14], 'root.skirt': [20, 0, 0], 'rip0.scale': [40, 128, 40], 'rip0.alpha': [32, 0, 0], 'rip1.scale': [40, 128, 40], 'rip1.alpha': [32, 0, 0],
                            **{f'bub{i}.scale': [0, 0, 0] for i in range(3)}, **{f'bub{i}.alpha': [0, 0, 0] for i in range(3)}}),
            dict(t=30, ease='out', pose={'root.scale': [138, 128, 138]}), dict(t=45, ease='inout', pose={'root.scale': [128, 128, 128], 'root.skirt': [0, 0, 0]}),
            dict(t=60, pose={'root.scale': [128, 128, 128], 'root.skirt': [0, 0, 0]}),
            # idle: 60..92; values at 92 equal the values at 60
            dict(t=60, ease='hold', pose={'rip0.scale': [40, 128, 40], 'rip0.alpha': [18, 0, 0], 'rip1.scale': [118, 128, 118], 'rip1.alpha': [10, 0, 0]}),
            dict(t=76, pose={'rip0.scale': [118, 128, 118], 'rip0.alpha': [10, 0, 0], 'rip1.scale': [196, 128, 196], 'rip1.alpha': [32, 0, 0]}),
            dict(t=76, ease='hold', pose={'rip1.scale': [40, 128, 40]}),
            dict(t=92, pose={'rip0.scale': [196, 128, 196], 'rip0.alpha': [32, 0, 0], 'rip1.scale': [118, 128, 118], 'rip1.alpha': [10, 0, 0]}),
            dict(t=76, pose={'root.skirt': [5, 0, 0], 'root.scale': [131, 128, 131]}), dict(t=92, pose={'root.skirt': [0, 0, 0], 'root.scale': [128, 128, 128]})]
    for i in range(3):
        a = 60 + 10 * i
        keys += [dict(t=60, ease='hold', pose={f'bub{i}.scale': [0, 0, 0], f'bub{i}.alpha': [0, 0, 0]}), dict(t=a, ease='hold', pose={f'bub{i}.scale': [20, 20, 20]}),
                 dict(t=a + 8, ease='out', pose={f'bub{i}.scale': [128, 128, 128], f'bub{i}.alpha': [0, 0, 0]}), dict(t=a + 12, pose={f'bub{i}.scale': [150, 90, 150], f'bub{i}.alpha': [32, 0, 0]}),
                 dict(t=92, ease='hold', pose={f'bub{i}.scale': [0, 0, 0], f'bub{i}.alpha': [0, 0, 0]})]
    clip = dict(name='pool_spawn_idle', lengths=lengths, sequence=dict(loop_frames=8), keys=keys)
    fxkit.finish(HERE / 'pool', 'pool', HERE / 'mesh' / 'pool.json', sk, clip, kind='loc', ids=dict(base=2724, seq=20204), what='kit proof: pool', ambient=30, contrast=0, size='1,1', randomstart=0)


PAIRS = [('cloud', 'locf', 'loc:11700', 'loc', 0, 102), ('orb', 'def', 'id:1340', 'proj', 0, None), ('tail', 'def', 'id:2007', 'proj', 0, None), ('dust', 'def', 'id:3370', 'spot', 0, None), ('pool', 'locf', 'loc:51046', 'loc', 0, 184)]


def eff(name, kind): return f'{kind}:{(HERE / name / (name + (".loc" if kind == "locf" else ".def"))).as_posix()}'


def draw():
    OUT.mkdir(parents=True, exist_ok=True)
    rng = random.Random(4)
    for name, kind, real, job, h, life in PAIRS:
        fx.LOC_LIFE = life
        e = [real, eff(name, kind)]
        fx.gif(OUT / f'{name}-pair.gif', e + e, job, close=False, size=(300, 240), labels=['real', f'kit: {name}', '', ''], cols=2) if False else None
        # close row and game-size row in one moving picture
        fx.gif(OUT / f'{name}-pair-close.gif', e, job, close=True, size=(360, 290), labels=[f'REAL {fx.label_of(real)}', f'KIT {name}'])
        fx.gif(OUT / f'{name}-pair-game.gif', e, job, close=False, size=(360, 290), labels=[f'REAL {fx.label_of(real)}', f'KIT {name}'])
        fx.sheet(OUT / f'{name}-close.png', e, job, n=6, close=True, size=(300, 240), labels=[f'REAL {fx.label_of(real)}', f'KIT {name}'])
        fx.sheet(OUT / f'{name}-game.png', e, job, n=6, close=False, size=(300, 240), labels=[f'REAL {fx.label_of(real)}', f'KIT {name}'])
        a = Image.open(OUT / f'{name}-close.png'); b = Image.open(OUT / f'{name}-game.png')
        im = Image.new('RGB', (a.width, a.height + b.height)); im.paste(a, (0, 0)); im.paste(b, (0, a.height)); im.save(OUT / f'{name}-pair.png')
        (OUT / f'{name}-close.png').unlink(); (OUT / f'{name}-game.png').unlink()
        order = [0, 1]; rng.shuffle(order)
        fx.gif(OUT / f'{name}-blind.gif', [e[i] for i in order], job, close=False, size=(360, 290), labels=['1', '2'])
        (OUT / f'{name}-blind.key.txt').write_text('\n'.join(f'{k + 1}: {e[i]}' for k, i in enumerate(order)) + '\n')
        print('drawn', name)
    fx.LOC_LIFE = None


def measure():
    L = []
    for name, kind, real, job, h, life in PAIRS:
        fx.LOC_LIFE = life
        for e in (real, eff(name, kind)):
            l = soft.line(soft.one(e, job, h)); l = l.replace(HERE.as_posix() + '/', 'kit ')
            print(l[:400]); L.append(l)
    fx.LOC_LIFE = None
    (HERE.parent / 'data' / 'kitproof.txt').write_text('\n'.join(L) + '\n')


if __name__ == '__main__':
    what = sys.argv[1] if len(sys.argv) > 1 else 'all'
    if what in ('build', 'all'): build()
    if what in ('draw', 'all'): draw()
    if what in ('measure', 'all'): measure()
