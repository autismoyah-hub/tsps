"""ifpalette.py: the NAMED PALETTE of INTERFACES.md (v2) section 12.

A rules sheet may name a colour only by one of these names. Built from recipes/census_colours.py: every colour used
by visible text, <col> spans, rectangles or lines in at least 2 of the drawn 239 groups (held-out panels excluded),
plus the three colours the 242 HPBAR_HUD scripts set at run time. The commonest have descriptive names; the rest
are P-numbers. A colour not in the palette is named 'unlisted' (an author cannot recover it; INTERFACES.md says so).
"""
NAMED = [
    ('orange', 0xff981f), ('black', 0x000000), ('white', 0xffffff), ('bevel dark', 0x0e0e0c), ('bevel light', 0x474745),
    ('red', 0xff0000), ('grey', 0x9f9f9f), ('yellow', 0xffff00), ('brown bevel dark', 0x383023), ('mid grey', 0x808080),
    ('brown bevel light', 0x5a5245), ('deep orange', 0xff7d1f), ('light orange', 0xffb83f), ('light grey', 0xbfbfbf),
    ('green', 0x0dc10d), ('olive grey', 0x5d5848), ('silver', 0xaaaaaa), ('parchment', 0xebd5b2), ('dark umber', 0x30201c),
    ('dim grey', 0x7f7f7f), ('gold', 0xffcc33), ('pale silver', 0xcccccc), ('near black brown', 0x170801),
    ('dark brown', 0x403020), ('maroon', 0x800000), ('khaki brown', 0x726451), ('bright green', 0x00ff00),
    ('table rule brown', 0x2e2b23), ('tangerine', 0xff9933), ('brown', 0x46320a), ('amber', 0xffa82f),
    ('dark rule', 0x28251e), ('dark red', 0xbb0000), ('lavender', 0xccccff), ('off-white', 0xeaeaea),
    ('health red', 0xcc0000), ('health green', 0x00cc00), ('frame black', 0x191511), ('blue', 0x0000ff),
    ('pale grey', 0xcfcfcf), ('cream', 0xf5e9d5),
    # set at run time by the 242 HPBAR_HUD scripts (TSPS)
    ('hud red', 0xc80000), ('hud dark green', 0x009000), ('hud green', 0x00c800),
]
OTHER = [
    0x202020, 0x303030, 0x008000, 0x999999, 0x4c3d26, 0x888800, 0x2a1e06, 0xff891f, 0xffaa44, 0x404040, 0x505050,
    0x333333, 0x282315, 0x37242b, 0x333300, 0xfcfce7, 0x00c000, 0xc00000, 0x2f2a25, 0x494034, 0xff991f, 0xff6600,
    0x990000, 0x585040, 0x666666, 0xfaa500, 0xdfdfdf, 0x29ffbd, 0xffc920, 0x9fd6ff, 0xff9720, 0xff9800, 0x140b02,
    0xf7f0df, 0xeb9a2f, 0x321e25, 0x2d0000, 0xb70000, 0x03171b, 0x1a88a2, 0x140d20, 0x563a8c, 0x5a7b86, 0xff9820,
    0x7f7fff, 0x339900, 0xcc6600, 0x005f00, 0x11d311, 0xac1414, 0xbf751d, 0x525252, 0x676725, 0xee1111, 0xff9900,
    0xffa53d, 0xff3333, 0x302c27, 0x0000cc, 0x1d7878, 0xc8c8c8, 0x191919, 0x0a2133, 0x0e2537, 0x12293b, 0x162d3f,
    0x1a3143, 0x1e3547, 0x22394b, 0x263d4f, 0x2a4153, 0x2e4557, 0x32495b, 0x364d5f, 0xf5bf18, 0x8080ff, 0x888888,
    0x9e9e9e, 0x6b6b69, 0xd299ff, 0x322b19, 0x281515, 0x99e7ff,
]
PALETTE = dict(NAMED)
for n, c in enumerate(OTHER, 1):
    PALETTE[f'P{n}'] = c
BY_COLOUR = {}
for k, v in PALETTE.items():
    BY_COLOUR.setdefault(v, k)


def name(c):
    return BY_COLOUR.get(c & 0xffffff, 'unlisted')


def colour(n):
    return PALETTE[n]


def table_md():
    rows = ['| name | rgb | name | rgb | name | rgb |', '|---|---|---|---|---|---|']
    items = list(PALETTE.items())
    for i in range(0, len(items), 3):
        cells = []
        for k, v in items[i:i + 3]:
            cells += [k, f'{v:06x}']
        cells += [''] * (6 - len(cells))
        rows.append('| ' + ' | '.join(cells) + ' |')
    return '\n'.join(rows)


if __name__ == '__main__':
    print(table_md())
