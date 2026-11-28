"""Names for the groups of frame bases, so tables and keyframe files can say `r_shoulder` instead of 30.

Base 0 is the player's. The names were read off the base itself (which labels a group lists, see
`asset_pose.exe CACHE info skeleton:0` and author-pack/labels/LABELS.md) and checked by posing: each named turn group
was given a single rotation and the picture looked at (render/research-animation-rig/). A joint is a (pivot group,
turn group) pair: the pivot is the centroid of the pivot group's labels PLUS the offset real frames always give it.

PLAYER_JOINTS: name -> (turn or move group, kind, pivot group or None)
PLAYER_PIVOT_OFFSETS: pivot group -> the [x, y, z] offset real frames carry on it (measured over every base-0
sequence by measure.py: the most common value and how common it is).
"""
PLAYER = {
    'root_move': (1, 'translate', 0), 'root_turn': (2, 'rotate', 0),
    'head': (4, 'rotate', 3),
    'torso_move': (6, 'translate', 5), 'torso': (7, 'rotate', 5),
    'l_hip': (9, 'rotate', 8), 'l_knee': (11, 'rotate', 10), 'l_foot': (13, 'rotate', 12),
    'r_hip': (15, 'rotate', 14), 'r_knee': (17, 'rotate', 16), 'r_foot': (19, 'rotate', 18),
    'l_arm_move': (21, 'translate', 20), 'l_shoulder': (22, 'rotate', 20), 'l_elbow': (24, 'rotate', 23),
    'l_hand_move': (26, 'translate', 25), 'l_hand': (27, 'rotate', 25),
    'r_arm_move': (29, 'translate', 28), 'r_shoulder': (30, 'rotate', 28), 'r_elbow': (32, 'rotate', 31),
    'r_hand_move': (34, 'translate', 33), 'r_hand': (35, 'rotate', 33),
    'cape_1': (37, 'rotate', 36), 'cape_2': (39, 'rotate', 38), 'cape_3': (41, 'rotate', 40), 'cape_4': (43, 'rotate', 42),
    'flap': (45, 'rotate', 44),
    'weapon_move': (47, 'translate', 46), 'weapon': (48, 'rotate', 46),
    'skirt_l_upper': (50, 'rotate', 49), 'skirt_l_lower': (52, 'rotate', 51),
    'skirt_r_upper': (54, 'rotate', 53), 'skirt_r_lower': (56, 'rotate', 55),
    'all_scale': (65, 'scale', 64),
    'all_alpha': (68, 'alpha', None),
    'body_turn2': (113, 'rotate', 112), 'body_move2': (114, 'translate', 112),
}
BY_GROUP = {0: {v[0]: k for k, v in PLAYER.items()}}
for name, (g, kind, pivot) in PLAYER.items():
    if pivot is not None:
        BY_GROUP[0].setdefault(pivot, 'pivot:' + name.replace('_move', '').replace('_turn', ''))


def name(base_id, group):
    return BY_GROUP.get(base_id, {}).get(group, '')
