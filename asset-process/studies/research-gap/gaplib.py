"""research-gap shared library: the two groups and a loader.

Loading goes through research-clean/cleanlib.py (the client's assembly and light, checked there), with the cache of
assembled meshes redirected into THIS folder's data/ so nothing is written into the other studies' folders.
"""
import sys, pathlib, json
HERE = pathlib.Path(__file__).resolve().parent
B = HERE.parent
sys.path.insert(0, str(B / 'research-clean'))
import cleanlib as cl
import cmesh
cmesh.DATA = HERE / 'data'          # assembled meshes and converted defs go here, not into research-clean/data

# REAL: 2019 on by model id (36,000 or more), plus Tekton (2017; named by the owner and the task). Names checked in the census.
REAL = {
    'Tekton': 7541, 'Minotaur': 12812, 'Lynx': 11068, 'Nex': 11278, 'Vardorvis': 12223, 'Branda': 12596, 'Eldric': 14147,
    'Araxxor': 13668, 'Jaguar': 12976, 'Tormented Demon': 13599, 'Basilisk Knight': 9293, 'Sol Heredit': 12821,
    'Akkha': 11789, 'Ba-Ba': 11778, 'Duke Sucellus': 12191, 'Yama': 14176, 'Bull': 15625, 'Alchemical Hydra': 8615,
    'The Nightmare': 9425, 'Araxyte': 11175, 'The Leviathan': 12214, 'The Whisperer': 12204, 'The Hueycoatl': 14009,
    'Doom of Mokhaiotl': 14707, 'Maggot King': 15742, 'Callisto': 6609, 'Zebak': 11730, 'Kephri': 11719,
    'Amoxliatl': 13685, "Elidinis' Warden": 11753,
}
BEST = {
    'Warden-General III': B / 'boss-warden-general-3' / 'boss-warden-general-3.json',
    'Stone watcher III': B / 'creature-stone-watcher-3' / 'creature-stone-watcher-3.json',
    'Star hound 3': B / 'creature-star-hound-3' / 'creature-star-hound-3.json',
    'Armillary sentinel': HERE / 'defs' / 'sentinel.json',       # points at research-skeleton/sentinel/r8/sentinel.bin (NPC 16360)
    'Zodiac ray': B / 'research-rig2' / 'ray' / 'work' / 'r8' / 'export' / 'zodiac-ray.json',
}
REJECTED = {
    'Stone watcher I': B / 'creature-stone-watcher' / 'creature-stone-watcher.json',
    'Stone watcher II': B / 'creature-stone-watcher-2' / 'creature-stone-watcher-2.json',
    'Star hound 2': B / 'creature-star-hound-2' / 'star-hound.json',
    'Warden-General I': B / 'boss-warden-general' / 'boss-warden-general.json',
    'Warden-General II': B / 'boss-warden-general-2' / 'eclipse.json',
    'Orrery spider': B / 'creature-orrery-spider' / 'orrery-spider.json',
}
# nearest real neighbour of each best original, for the boards (chosen by kind and size, see LOOK.md)
NEAREST = {'Warden-General III': 'Tekton', 'Stone watcher III': 'Minotaur', 'Star hound 3': 'Lynx',
           'Armillary sentinel': "Elidinis' Warden", 'Zodiac ray': 'Maggot King'}

ALL = {**{k: f'npc:{v}' for k, v in REAL.items()}, **{k: 'def:' + str(p) for k, p in BEST.items()}, **{k: 'def:' + str(p) for k, p in REJECTED.items()}}
GROUP = {**{k: 'REAL' for k in REAL}, **{k: 'BEST' for k in BEST}, **{k: 'REJ' for k in REJECTED}}

_cache = {}
def load(entry):
    if entry not in _cache: _cache[entry] = cl.load(entry)
    return _cache[entry]
