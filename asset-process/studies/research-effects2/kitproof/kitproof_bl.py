"""kitproof_bl.py (runs inside Blender): one piece per kit function, each in the size and palette of a REAL effect of the same
kind (not a copy of its mesh: the kit's own geometry), for the side-by-side proof in kitproof.py.
  cloud  lump_cloud    against object 11700 snakeboss_poisoncloud (Zulrah's venom cloud)
  orb    glow_shell    against spot 1340 olm_generic_range_proj
  tail   streaks       against spot 2007 nex_finale_attack_proj
  dust   dust_burst    against spot 3370 vfx_doom_area_slam
  pool   pool          against object 51046 pmoon_boss_blood_pool
"""
import sys, pathlib
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import softkit as K

out = HERE / 'mesh'

k = K.Kit(seed=11)
k.anchor((0, -60, 0), 'hub')
k.lump_cloud((0, -78, 0), 15, 112, 50, 'c', 'c', (30, 7, 20), alpha=208, sets=3, squash_y=0.42, r_jitter=0.3, rough=0.3)
k.export(out / 'cloud.json')

k = K.Kit(seed=5)
k.glow_shell((0, 0, 0), 15, 'core', 'g', (14, 7, 66), [(14, 7, 60), (13, 7, 51), (13, 7, 46)], layers=3, grow=1.36, alpha_core=30, alphas=[60, 110, 200], level=1, rough=0.1)
k.export(out / 'orb.json')

k = K.Kit(seed=3)
k.glow_shell((0, 0, -4), 15, 'core', 'g', (49, 4, 96), [(49, 5, 78), (60, 7, 70)], layers=2, grow=1.34, alpha_core=16, alphas=[64, 144], stretch=(1, 1, 1.35), rough=0.12)
k.streaks(7, 6, 92, 15, 6.5, 't', 't', [(49, 5, 78), (60, 7, 70), (60, 7, 62)], alpha0=48, alpha1=208, segments=3, sets=3)
k.export(out / 'tail.json')

k = K.Kit(seed=8)
k.anchor((0, -2, 0), 'hub')
k.dust_burst((0, -2, 0), 28, 44, 13, 'd', 'd', (2, 2, 24), alpha=0, sets=4, tones=[(2, 2, 30), (2, 2, 24), (2, 3, 19)])
k.dust_burst((0, -2, 0), 12, 20, 11, 'e', 'e', (2, 2, 26), alpha=0, sets=2, tones=[(2, 2, 31), (2, 2, 25), (2, 3, 20)])
k.export(out / 'dust.json')

k = K.Kit(seed=21)
k.anchor((0, -2, 0), 'hub')
k.pool((0, -1, 0), 64, 'pool', 'pool', [(63, 6, 30), (63, 6, 41), (62, 6, 51)], (59, 7, 66), skirt_alpha=(80, 160), dome=9, ripples=2, ripple_paint=(59, 7, 72), bubbles=6, bubble_paint=(60, 7, 60), drips=5)
k.export(out / 'pool.json')
