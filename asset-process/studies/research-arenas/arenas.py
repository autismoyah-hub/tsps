"""arenas.py: the real boss arenas of the study: where each is in the revision-239 map and where its boss stands.

Each entry: key, name, group, plane, seed (a walkable tile inside), boss (npc id, south-west tile x, y[, angle]) or bosses,
size (tiles a side of the boss), player (the tile a player fights from, for the played camera), and optionally
  clip (x0, y0, x1, y1)   the flood fill is kept inside it (an open doorway, an approach corridor)
  cuts [(x, y), ..]       tiles closed by hand (a doorway tile)
  locs  ["loc ID X Y SHAPE ROT 0 - sync", ..]   objects the SERVER spawns for the fight, added for the pictures only
  where: how the place was verified.  src: 'spawn' (server spawn table), 'plugin' (the owner's server plugin data),
  'object' (a named object of the fight found in the map), 'known' (my knowledge of the coordinates) - every one was then
  drawn with its boss (render/research-arenas/ARENA-top.png) and looked at.
Boss positions marked `pos='known'` are where the boss stands in the fight by my knowledge; the map does not hold them.
"""
ARENAS = [
    dict(key='nex', name="Nex's prison (Ancient Prison)", group='reference', plane=0, seed=(2925, 5203), clip=(2910, 5170, 2960, 5240),
         boss=(11278, 2924, 5202), size=3, player=(2925, 5196), src='plugin + known', where='server plugin SAFE_ROOM (2905,5203) beside it'),
    dict(key='nightmare', name="The Nightmare's arena (Sisters' sanctuary)", group='reference', plane=3, seed=(3872, 9945),
         boss=(9425, 3870, 9949), size=5, player=(3872, 9944), src='plugin', where='plugin: flower cross through (3872,9951)'),
    dict(key='maggot', name="Maggot King's lair", group='reference', plane=0, seed=(2911, 7836),
         boss=(15742, 2909, 7838), size=5, player=(2911, 7834), src='plugin', where='maggot-king.json bossSpawn (2909,7838)'),
    dict(key='doom', name='Doom of Mokhaiotl, delve floor', group='reference', plane=0, seed=(1311, 9566), clip=(1290, 9559, 1330, 9590),
         boss=(14707, 1309, 9571), size=5, player=(1311, 9565), src='plugin', where='Shared.DoomOfMokhaiotl BOSS (1309,9571)'),
    dict(key='tekton', name="Tekton's room (Chambers of Xeric)", group='cox', plane=1, seed=(3282, 5296),
         boss=(7541, 3280, 5296), size=4, player=(3282, 5290), src='object', where='Giant Anvil 29867 at (3278,5302,1)'),
    dict(key='olm', name="Great Olm's chamber (Chambers of Xeric, final)", group='cox', plane=0, seed=(3233, 5740),
         boss=None, size=5, player=(3233, 5738), src='object', where='Mystical barrier 29879 (3231,5729), Crystal structure 29888',
         locs=['loc 29881 3238 5738 10 1 0 - sync', 'loc 29884 3238 5733 10 1 0 - sync', 'loc 29887 3238 5743 10 1 0 - sync'], boss_box=(3238, 5733, 5, 15), view='e'),
    dict(key='maiden', name='Maiden of Sugadinti (Theatre of Blood)', group='tob', plane=0, seed=(3170, 4446), clip=(3150, 4430, 3185, 4462),
         boss=(8360, 3162, 4444, 1536), size=6, player=(3172, 4446), src='known', where='region 12613', view='w'),
    dict(key='bloat', name='Pestilent Bloat (Theatre of Blood)', group='tob', plane=0, seed=(3296, 4441),
         boss=(8359, 3288, 4440), size=5, player=(3296, 4441), src='known', where='region 13125', pos='walks the ring'),
    dict(key='nylocas', name='Nylocas Vasilias (Theatre of Blood)', group='tob', plane=0, seed=(3295, 4245),
         boss=(8355, 3294, 4247), size=4, player=(3295, 4244), src='known', where='region 13122'),
    dict(key='sotetseg', name='Sotetseg (Theatre of Blood)', group='tob', plane=0, seed=(3280, 4320), clip=(3265, 4308, 3295, 4340),
         boss=(8388, 3277, 4326), size=5, player=(3279, 4319), src='known', where='region 13123'),
    dict(key='xarpus', name='Xarpus (Theatre of Blood)', group='tob', plane=1, seed=(3170, 4382),
         boss=(8340, 3168, 4385), size=5, player=(3170, 4382), src='object', where='Barrier 32755 at (3169..3171,4379 / 4395,1)'),
    dict(key='verzik', name='Verzik Vitur (Theatre of Blood, final)', group='tob', plane=0, seed=(3168, 4306),
         boss=(8374, 3165, 4310), size=7, player=(3168, 4305), src='known', where='region 12611'),
    dict(key='akkha', name='Akkha (Tombs of Amascut)', group='toa', plane=1, seed=(3680, 5402),
         boss=(11789, 3679, 5407), size=3, player=(3680, 5402), src='object', where='Teleport crystal 45866 at (3694,5407,1)'),
    dict(key='kephri', name='Kephri (Tombs of Amascut)', group='toa', plane=0, seed=(3550, 5402),
         boss=(11719, 3548, 5406), size=5, player=(3550, 5402), src='object', where='Teleport crystal 45505 at (3538,5408,0)'),
    dict(key='baba', name='Ba-Ba (Tombs of Amascut)', group='toa', plane=0, seed=(3808, 5408),
         boss=(11778, 3811, 5406, 512), size=5, player=(3805, 5408), src='object', where='Teleport crystal 45754 at (3794,5408,0)', view='e'),
    dict(key='zebak', name='Zebak (Tombs of Amascut)', group='toa', plane=0, seed=(3930, 5408),
         boss=(11730, 3933, 5404, 512), size=9, player=(3929, 5408), src='object', where='Teleport crystal 45506 at (3954,5408,1)', view='e'),
    dict(key='wardens', name='The Wardens, obelisk room (Tombs of Amascut, final)', group='toa', plane=1, seed=(3808, 5146),
         boss=(11762, 3806, 5154), size=5, player=(3808, 5146), src='object', where='Teleport crystal 45579 at (3808,5172,1)'),
    dict(key='inferno', name='The Inferno floor (TzKal-Zuk)', group='reference', plane=0, seed=(2271, 5343),
         boss=(7706, 2268, 5360), size=7, player=(2271, 5352), src='known', where='region 9043', pos='Zuk stands behind the north edge'),
    dict(key='colosseum', name='Fortis Colosseum floor (Sol Heredit)', group='reference', plane=0, seed=(1824, 3104),
         boss=(12821, 1823, 3105), size=5, player=(1825, 3099), src='known', where='region 7216'),
    dict(key='duke', name='Duke Sucellus (Desert Treasure 2)', group='dt2', plane=0, seed=(3039, 6440),
         boss=(12166, 3036, 6452), size=7, player=(3039, 6444), src='spawn', where='npc-spawns.json Duke Sucellus (3036,6452)'),
    dict(key='vardorvis', name='Vardorvis (Desert Treasure 2)', group='dt2', plane=0, seed=(1128, 3421), clip=(1124, 3413, 1135, 3424),
         boss=(12223, 1128, 3417), size=2, player=(1129, 3414), src='spawn', where='npc-spawns.json Vardorvis (1128,3417)'),
    dict(key='whisperer', name='The Whisperer (Desert Treasure 2)', group='dt2', plane=0, seed=(2656, 6365), clip=(2630, 6340, 2680, 6384),
         boss=(12204, 2655, 6369), size=3, player=(2656, 6363), src='spawn', where='npc-spawns.json The Whisperer (2655,6369)'),
    dict(key='araxxor', name="Araxxor's lair", group='other', plane=0, seed=(3634, 9810),
         boss=(13668, 3631, 9814), size=7, player=(3634, 9809), src='plugin', where='Shared.Araxxor SPAWN_TILE (3631,9814)'),
    dict(key='hydra', name="Alchemical Hydra's lab", group='other', plane=0, seed=(1366, 10262),
         boss=(8615, 1364, 10265), size=6, player=(1366, 10260), src='spawn', where='npc-spawns.json Alchemical Hydra (1364,10265)'),
    dict(key='vorkath', name="Vorkath's island (Ungael)", group='other', plane=0, seed=(2272, 4058),
         boss=(8061, 2269, 4062), size=7, player=(2272, 4057), src='known', where='region 9023'),
    dict(key='muspah', name="Phantom Muspah's cavern", group='other', plane=0, seed=(2848, 4254),
         boss=(12077, 2846, 4257), size=5, player=(2848, 4252), src='known', where='region 11330'),
]
BY = {a['key']: a for a in ARENAS}
