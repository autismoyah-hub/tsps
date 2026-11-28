"""Print a creature's materials and ramps. usage: show_mats.py ENTRY [ENTRY...]   (npc:ID | def:PATH.json)"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import paintlib as pl
for e in sys.argv[1:]:
    sk = pl.skin(e); mats, _ = pl.materials(sk)
    print(e[-60:], f"skin faces {len(sk['C'])}")
    for d in mats:
        if d['share'] >= 0.003:
            print('   %-16s %5.1f%% tones %2d lights %s step %.1f cv %.2f hues %d sats %d | %s' % (pl.mat_name(d), 100 * d['share'], d['tones'], d['lights'], d['step_med'], d['step_cv'], d['hues'], d['sats'],
                  ' '.join('%d.%d.%d' % pl.hsl(c) for c in d['colours'] if d['shares'][c] >= 0.003)))
