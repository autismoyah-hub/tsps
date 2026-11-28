"""Name each found room: its kind (by what stands in it), its place (by world position) and era; write rooms.json.

usage: classify.py            reads rooms-all.json, writes rooms.json and kinds.txt
Kinds are decided by weighted keyword votes over the NAMES of the objects in the room (the names a player reads
on examine), not by its place. A room with no vote of 2 or more is 'house' (an ordinary dwelling room) if it has a
bed, table or seat, otherwise 'other'. Tower: a room with at least two diagonal wall pieces (shape 9).
"""
import sys, json, re, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R

VOTES = {
    'armoury': [(r'weapon rack|armour rack|rack of|spear rack|bow rack|weapons? stand|armour stand|suit of armour|armour display|weapon display|sword cabinet|bow cabinet|axe cabinet|training dummy|dummy|archery target|\btarget\b|\barms\b|armour|weapons?', 2),
                (r'^shields?$|^standard$|^banner$|crossed|swords?$', 1)],
    'guard': [(r'guard|bunk|\bcell\b|jail|prison', 1)],
    'library': [(r'bookcase|book ?shel|bookshel|lectern|study desk|reading', 2), (r'^desk$|\bbooks\b|scroll|map table|census|records', 1)],
    'smithy': [(r'anvil|furnace|\bforge|bellows|smelt|hearth', 2)],
    'workshop': [(r'workbench|work bench|lathe|\bloom\b|spinning wheel|potter|pottery|sawmill|\bdrill\b|\btools?\b|\bvice\b|carpent|sewing|tanning|crafting', 2)],
    'kitchen': [(r'\brange\b|\bstove\b|\boven\b|\bsink\b|larder|cooking|kitchen|churn|dairy|hanging meat|\bflour\b', 2), (r'cupboard', 1)],
    'bedroom': [(r'\bbed\b|bunk bed|four.poster|wardrobe|dresser|bedside|mirror|\bbath', 2)],
    'throne': [(r'throne', 6), (r'council|banquet|long table|royal', 2)],
    'shop': [(r'counter|stall|shop|display case|market|merchant|wares', 2)],
    'bank': [(r'bank booth|bank table|bank chest|deposit box', 3)],
    'temple': [(r'altar|pew|shrine|prayer|font\b|holy|church|temple', 2)],
    'laboratory': [(r'laborator|alchem|apparatus|experiment|vials?|beakers?|potion|test tube|still\b|telescope|orrery|crystal ball|astronom|research|specimen|chemistry|distillation|mixing tube|bunsen|machine|globe|chart', 2), (r'cauldron|crystal', 1)],
    'tavern': [(r'\bbar\b|barstool|\bbeer|tavern|\bpub\b|bar pump|\bale\b', 2)],
}
VOTES = {k: [(re.compile(p, re.I), w) for p, w in v] for k, v in VOTES.items()}

PLACES = [  # name, x0, x1, y0, y1, era  (world tiles; first match wins)
    ('Lumbridge', 3190, 3260, 3190, 3260, 'classic'), ('Varrock', 3170, 3290, 3370, 3520, 'classic'),
    ('Falador', 2930, 3070, 3300, 3400, 'classic'), ('Draynor', 3070, 3135, 3220, 3380, 'classic'),
    ('Edgeville', 3070, 3130, 3470, 3520, 'classic'), ('Barbarian Village', 3070, 3110, 3400, 3440, 'classic'),
    ('Camelot/Seers', 2680, 2790, 3440, 3530, 'classic'), ('East Ardougne', 2560, 2700, 3250, 3345, 'classic'),
    ('West Ardougne', 2430, 2560, 3270, 3345, 'classic'), ('Yanille', 2530, 2625, 3070, 3115, 'classic'),
    ('Catherby', 2790, 2840, 3410, 3470, 'classic'), ('Taverley', 2870, 2945, 3410, 3470, 'classic'),
    ('Burthorpe/Warriors', 2830, 2945, 3530, 3590, 'classic'), ('Al Kharid', 3265, 3335, 3135, 3220, 'classic'),
    ('Port Sarim', 3005, 3065, 3170, 3270, 'classic'), ('Rimmington', 2925, 2985, 3190, 3235, 'classic'),
    ("Wizards' Tower", 3095, 3125, 3145, 3175, 'classic'), ('Tutorial Island', 3050, 3160, 3050, 3130, 'classic'),
    ('Canifis', 3465, 3525, 3455, 3515, 'classic'), ('Port Phasmatys', 3640, 3715, 3455, 3515, 'classic'),
    ('Gnome Stronghold', 2380, 2500, 3400, 3520, 'classic'), ('Rellekka', 2600, 2700, 3640, 3720, 'classic'),
    ('Fremennik Isles', 2300, 2440, 3770, 3870, 'classic'), ('Lletya', 2305, 2365, 3145, 3200, 'classic'),
    ('Shilo/Karamja', 2750, 2960, 2940, 3200, 'classic'), ('Witchaven/Khazard', 2600, 2740, 3140, 3290, 'classic'),
    ('Keldagrim', 2810, 2960, 10140, 10230, 'classic'), ('Sophanem/desert', 3150, 3450, 2750, 3130, 'classic'),
    ('Mos Le\'Harmless', 3640, 3860, 2920, 3080, 'classic'), ('Wilderness', 2940, 3400, 3520, 3970, 'classic'),
    ('Prifddinas', 3190, 3350, 6000, 6160, 'recent'), ('Darkmeyer', 3585, 3665, 3325, 3405, 'recent'),
    ('Slepe', 3685, 3735, 3285, 3355, 'recent'), ('Ferox Enclave', 3115, 3160, 3605, 3655, 'recent'),
    ('Fossil Island', 3650, 3880, 3700, 3900, 'recent'),
    ('Hosidius', 1690, 1830, 3530, 3640, 'recent'), ('Piscarilius', 1770, 1870, 3660, 3810, 'recent'),
    ('Arceuus', 1570, 1730, 3700, 3880, 'recent'), ('Lovakengj', 1410, 1580, 3700, 3880, 'recent'),
    ('Shayzien', 1440, 1580, 3530, 3660, 'recent'), ('Kourend Castle', 1580, 1690, 3640, 3700, 'recent'),
    ('Kebos/Karuulm', 1190, 1410, 3600, 3880, 'recent'), ('Kourend (other)', 1150, 1900, 3400, 3950, 'recent'),
    ('Civitas illa Fortis', 1600, 1830, 3030, 3200, 'recent'), ('Aldarin', 1330, 1460, 2890, 2990, 'recent'),
    ('Varlamore (other)', 1150, 1900, 2800, 3400, 'recent'),
    ('Kourend/Varlamore (underground)', 1150, 1950, 9000, 10400, 'recent'),
    ('Prifddinas (underground)', 3140, 3400, 12300, 12500, 'recent'),
]


def place(x, y):
    for n, x0, x1, y0, y1, era in PLACES:
        if x0 <= x <= x1 and y0 <= y <= y1: return n, era
    if y >= 6400: return 'underground/instance', 'unknown'
    return 'elsewhere', 'unknown'


def classify(r):
    names = [R.name(it[0]) for it in r['items'] if it[5] not in ('wall', 'roof')]
    if any(n.lower().endswith(' space') for n in names):
        return 'poh-template', {}, 0       # player-owned-house build hotspots: templates, not rooms
    score = collections.Counter()
    for n in names:
        if n == 'null': continue
        for k, rules in VOTES.items():
            best = max((w for p, w in rules if p.search(n)), default=0)
            score[k] += best
    diag = sum(1 for it in r['items'] if it[3] == 9)
    kinds = [k for k, s in score.most_common() if s >= 2]
    if kinds:
        kind = kinds[0]
        if kind == 'smithy' and score['workshop'] >= score['smithy']: kind = 'workshop'
    else:
        f = [R.func_class(n) for n in names]
        kind = 'house' if any(c in ('bed', 'table', 'seat') for c in f) else 'other'
    if kind == 'guard': kind = 'armoury'  # armouries and guard rooms are one family here
    return kind, dict(score), diag


def main():
    rooms = json.loads((R.HERE / 'rooms-all.json').read_text())
    out = []
    sigs = {}
    for i, r in enumerate(rooms):
        mx = min(t[0] for t in r['tiles']); my = min(t[1] for t in r['tiles'])
        sig = (r['area'], tuple(sorted((it[0], it[1] - mx, it[2] - my, it[3], it[4]) for it in r['items'] if it[5] not in ('wall', 'roof'))))
        # copies of a room (instances, quest copies) count once: the first, preferring a named place
        r['dup_of'] = sigs.get(sig)
        if r['dup_of'] is None: sigs[sig] = i
        bx, by = r['base']
        cx = bx + sum(t[0] for t in r['tiles']) / r['area']; cy = by + sum(t[1] for t in r['tiles']) / r['area']
        kind, score, diag = classify(r)
        pl, era = place(cx, cy)
        # a tower room: four or more diagonal wall pieces, a near-square plan, splayed corners (fills under 92% of its box)
        tower = diag >= 4 and abs(r['w'] - r['h']) <= 1 and r['area'] < 0.92 * r['w'] * r['h']
        r.update(id=i, kind=kind, votes=score, tower=tower, centre=[round(cx), round(cy)], place=pl, era=era)
        out.append(r)
    (R.HERE / 'rooms.json').write_text(json.dumps(out))
    c = collections.Counter(r['kind'] for r in out)
    lines = ['kind counts over all %d candidates' % len(out)] + [f'{k:12s} {v:5d}' for k, v in c.most_common()]
    ce = collections.Counter((r['kind'], r['era']) for r in out)
    lines += ['', 'by era (recent / classic / unknown)'] + [f"{k:12s} {ce[(k,'recent')]:4d} {ce[(k,'classic')]:4d} {ce[(k,'unknown')]:4d}" for k in c]
    lines += ['', 'towers (>=4 diagonal wall pieces): %d' % sum(r['tower'] for r in out)]
    (R.HERE / 'kinds.txt').write_text('\n'.join(lines) + '\n')
    print('\n'.join(lines))


if __name__ == '__main__':
    main()
