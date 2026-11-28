"""itemcodec.py: write a complete item definition, as the cache stores it and as the owner's server takes it.

usage: itemcodec.py roundtrip            every real record: client-decoded fields -> encode() -> the client's decoder
                                         again (research_items.exe decode) -> compare field by field
       itemcodec.py encode SPEC.json OUT.def [--server OUT.json]     one of ours
       itemcodec.py show FILE.def        decode a .def with the client and print its fields

The field names are the client's (rust-client/src/item.rs `Definition`), as research_items.exe prints them:
  model, name, examine, zoom, angles [xan, yan, zan], offset [x, y], stackable, price, wear_positions [3],
  members, recolor [[from, to]..], retexture, wear_models [[male x3], [female x3]], wear_offset [male, female],
  head_models [[male x2], [female x2]], category, weight, note, bought, placeholder, counts, scale, ambient,
  contrast (the decoded value, 5 x the stored byte), team, tradeable, stockmarket, shift_click,
  inventory_operations [5], ground_operations, sub_operations, params [[key, kind, value]..].
A SPEC.json for one of ours may leave out anything it does not need; defaults are the decoder's.

Opcodes written (item.rs decode_mode; the same numbers the cache's own records use):
  2 name, 3 examine, 4 zoom, 5 / 6 / 95 xan yan zan, 7 / 8 offsets, 11 stackable (160: always stackable), 12 price,
  13 / 14 / 27 wear positions, 15 not tradeable, 16 members, 30-34 ground options, 35-39 inventory options,
  40 recolours, 41 retextures, 42 shift-click, 43 sub-options, 65 stock market, 75 weight, 94 category,
  97 / 98 note link and template, 100-109 stack variants, 110-112 scale, 113 ambient, 114 contrast, 115 team,
  139 / 140 bought, 148 / 149 placeholder, 249 params, 0 end;
  model ids: 1 inventory, 23 / 24 / 78 male worn (23 carries the male offset), 25 / 26 / 79 female, 90 / 92 male head,
  91 / 93 female head, all 16-bit; when ANY model id of the record is above 65535 the 32-bit forms are written
  instead: 44 inventory, 45 / 46 / 47 male, 48 / 49 / 50 female, 51 / 52 male head, 53 / 54 female head.
Not written: opcode 9 (the client reads and drops it) and the local-only opcodes 200-212, which the client's
decoder skips without storing, so nothing decoded depends on them (`roundtrip` counts the records that hold one).
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, subprocess, struct
import itemlib as L

DEFAULT = dict(model=0, name='null', examine='null', zoom=2000, angles=[0, 0, 0], offset=[0, 0], stackable=0, price=1,
               wear_positions=[-1, -1, -1], members=False, recolor=[], retexture=[], shift_click=-2,
               wear_models=[[-1, -1, -1], [-1, -1, -1]], wear_offset=[0, 0], head_models=[[-1, -1], [-1, -1]], category=-1,
               weight=0, note=[-1, -1], bought=[-1, -1], placeholder=[-1, -1], counts=[[0, 0]] * 10, scale=[128, 128, 128],
               ambient=0, contrast=0, team=0, tradeable=True, stockmarket=False, searchable=False,
               inventory_operations=[None, None, None, None, 'Drop'],
               ground_operations=[dict(index=2, sub=None, condition=None, text='Take')], sub_operations=[None] * 5, params=[], trailing='')


def _s(text):
    return text.encode('cp1252', errors='replace') + b'\0'


def encode(spec):
    d = dict(DEFAULT); d.update(spec)
    o = bytearray()
    u8 = lambda v: o.append(v & 255)
    u16 = lambda v: o.extend(struct.pack('>H', v & 0xffff))
    u32 = lambda v: o.extend(struct.pack('>I', v & 0xffffffff))
    for first, key in ((97, 'note'), (139, 'bought'), (148, 'placeholder')):     # the linked forms start with their link
        if d[key][0] != -1 or d[key][1] != -1:
            if d[key][0] != -1: u8(first); u16(d[key][0])
            if d[key][1] != -1: u8(first + 1); u16(d[key][1])
    ids = [d['model']] + sum(d['wear_models'], []) + sum(d['head_models'], [])
    wide = any(i > 65535 for i in ids)
    if d['model'] != 0:
        if wide: u8(44); u32(d['model'])
        else: u8(1); u16(d['model'])
    if d['name'] != 'null': u8(2); o.extend(_s(d['name']))
    if d['examine'] != 'null': u8(3); o.extend(_s(d['examine']))
    if d['zoom'] != 2000: u8(4); u16(d['zoom'])
    for op, v in ((5, d['angles'][0]), (6, d['angles'][1]), (95, d['angles'][2]), (7, d['offset'][0]), (8, d['offset'][1])):
        if v != 0: u8(op); u16(v)
    if d['stackable'] == 1: u8(11)
    elif d['stackable'] == 2: u8(160)
    if d['price'] != 1: u8(12); u32(d['price'])
    for op, v in ((13, d['wear_positions'][0]), (14, d['wear_positions'][1]), (27, d['wear_positions'][2])):
        if v != -1: u8(op); u8(v)
    if not d['tradeable']: u8(15)
    if d['members']: u8(16)
    ground = list(d['ground_operations'])
    if ground and ground[0] == dict(index=2, sub=None, condition=None, text='Take'):
        ground = ground[1:]                                    # the decoder's own default, still in first place
    for g in ground:
        if g['sub'] is not None or g['condition'] is not None:
            raise ValueError('ground sub-options and conditions (opcodes 200-202) are not read by the client in cache mode')
        u8(30 + g['index']); o.extend(_s(g['text']))
    for i, t in enumerate(d['inventory_operations']):
        if t is not None and not (i == 4 and t == 'Drop'): u8(35 + i); o.extend(_s(t))
    for op, key in ((40, 'recolor'), (41, 'retexture')):
        if d[key]:
            u8(op); u8(len(d[key]))
            for a, b in d[key]: u16(a); u16(b)
    if d['shift_click'] != -2: u8(42); u8(d['shift_click'])
    for parent, subs in enumerate(d['sub_operations']):
        if subs is not None:
            u8(43); u8(parent)
            for n, t in enumerate(subs):
                if t is not None: u8(n + 1); o.extend(_s(t))
            u8(0)
    for g, (first, second, third) in enumerate((((23, 45), (24, 46), (78, 47)), ((25, 48), (26, 49), (79, 50)))):
        m = d['wear_models'][g]
        if m[0] != -1:
            u8(first[wide]); (u32 if wide else u16)(m[0]); u8(d['wear_offset'][g])
        if m[1] != -1: u8(second[wide]); (u32 if wide else u16)(m[1])
        if m[2] != -1: u8(third[wide]); (u32 if wide else u16)(m[2])
    for (g, n), (narrow, big) in {(0, 0): (90, 51), (0, 1): (92, 52), (1, 0): (91, 53), (1, 1): (93, 54)}.items():
        v = d['head_models'][g][n]
        if v != -1: u8(big if wide else narrow); (u32 if wide else u16)(v)
    if d['stockmarket']: u8(65)
    if d['weight'] != 0: u8(75); u16(d['weight'])
    if d['category'] != -1: u8(94); u16(d['category'])
    for i, (a, b) in enumerate(d['counts']):
        if (a, b) != (0, 0): u8(100 + i); u16(a); u16(b)
    for i, v in enumerate(d['scale']):
        if v != 128: u8(110 + i); u16(v)
    if d['ambient'] != 0: u8(113); u8(d['ambient'])
    if d['contrast'] != 0:
        if d['contrast'] % 5: raise ValueError('contrast is stored as value / 5')
        u8(114); u8(d['contrast'] // 5)
    if d['team'] != 0: u8(115); u8(d['team'])
    if d['params']:
        u8(249); u8(len(d['params']))
        for key, kind, value in d['params']:
            u8({'int': 0, 'string': 1, 'long': 2}[kind]); o.extend(struct.pack('>I', key)[1:])
            if kind == 'string': o.extend(_s(value))
            elif kind == 'long': o.extend(struct.pack('>q', int(value)))
            else: o.extend(struct.pack('>i', value))
    u8(0)
    o.extend(bytes.fromhex(d.get('trailing', '')))
    return bytes(o)


def client_decode(records, plain=False):
    """records: [(id, bytes)]. Returns the client's decoded fields for each (research_items.exe decode)."""
    tmp = L.HERE / 'data' / '_codec.bin'; out = L.HERE / 'data' / '_codec.jsonl'
    with tmp.open('wb') as f:
        f.write(struct.pack('>I', len(records)))
        for i, b in records: f.write(struct.pack('>II', i, len(b))); f.write(b)
    r = subprocess.run([L.RESEARCH_ITEMS, L.CACHE, 'decode', str(tmp), str(out)] + (['plain'] if plain else []), capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    rows = [json.loads(x) for x in out.read_text(encoding='utf-8').splitlines()]
    tmp.unlink(); out.unlink()
    return rows


def server_row(item_id, base_item, d, model_files):
    """The owner's server format (server/data/definitions/custom-items.json; CacheDefinitions.registerCustomItems,
    ObjType field names). model_files: {model id: path relative to data/models}. NOT run against the server."""
    d0 = dict(DEFAULT); d0.update(d); d = d0
    t = dict(name=d['name'], examine=d['examine'], model=d['model'], zoom2d=d['zoom'], xan2d=d['angles'][0], yan2d=d['angles'][1], zan2d=d['angles'][2],
             offsetX2d=d['offset'][0], offsetY2d=d['offset'][1],
             maleModel=d['wear_models'][0][0], maleModel1=d['wear_models'][0][1], maleModel2=d['wear_models'][0][2], maleOffset=d['wear_offset'][0],
             femaleModel=d['wear_models'][1][0], femaleModel1=d['wear_models'][1][1], femaleModel2=d['wear_models'][1][2], femaleOffset=d['wear_offset'][1],
             maleHeadModel=d['head_models'][0][0], maleHeadModel2=d['head_models'][0][1], femaleHeadModel=d['head_models'][1][0], femaleHeadModel2=d['head_models'][1][1],
             wearPos=d['wear_positions'][0], wearPos2=d['wear_positions'][1], wearPos3=d['wear_positions'][2],
             recolorFrom=[a for a, _ in d['recolor']], recolorTo=[b for _, b in d['recolor']], retextureFrom=[a for a, _ in d['retexture']], retextureTo=[b for _, b in d['retexture']],
             resizeX=d['scale'][0], resizeY=d['scale'][1], resizeZ=d['scale'][2], ambient=d['ambient'], contrast=d['contrast'],
             inventoryActions=d['inventory_operations'], price=d['price'], isMembers=d['members'], isTradable=d['tradeable'],
             note=-1, noteTemplate=-1, unnotedId=-1, notedId=-1, placeholder=-1, placeholderTemplate=-1)
    return dict(id=item_id, baseItemId=base_item, objType=t, itemDef=dict(tradeable=d['tradeable']), models={str(k): v for k, v in model_files.items()})


def roundtrip():
    IT = L.items()
    recs = []; skipped_local = 0; unencodable = []
    keys = [k for k in DEFAULT if k != 'searchable'] + ['searchable']
    for i, d in sorted(IT.items()):
        try:
            recs.append((i, encode({k: d[k] for k in DEFAULT})))
        except ValueError as e:
            unencodable.append((i, str(e)))
    back = client_decode(recs)
    bad = 0; shown = 0; same_bytes = 0; fields = 0
    for (i, b), r in zip(recs, back):
        d = IT[i]
        same_bytes += b.hex() == d['raw']
        diff = [k for k in DEFAULT if r[k] != d[k]]
        fields += len(DEFAULT)
        if diff:
            bad += 1
            if shown < 8: shown += 1; print(f'  item {i} {d["name"]!r}: differs in {diff}: {[(d[k], r[k]) for k in diff][:3]}')
    print(f'{len(IT)} real item records; {len(recs)} re-encoded from the client-decoded fields and decoded again by the client '
          f'(Definition::decode_local): {bad} records differ in any of {len(DEFAULT)} fields ({fields} field comparisons); '
          f'{len(unencodable)} could not be encoded {unencodable[:3]}')
    print(f're-encoded bytes identical to the cache record: {same_bytes} of {len(recs)} (opcode order and the opcodes the client drops differ in the rest)')
    return bad


def main():
    a = sys.argv[1:]
    if not a: raise SystemExit(__doc__)
    if a[0] == 'roundtrip':
        sys.exit(1 if roundtrip() else 0)
    if a[0] == 'encode':
        spec = json.loads(pathlib.Path(a[1]).read_text(encoding='utf-8'))
        fields = {k: v for k, v in spec.items() if k in DEFAULT}
        b = encode(fields)
        pathlib.Path(a[2]).write_bytes(b)
        want = dict(DEFAULT); want.update(fields)
        for mode in (False, True):
            got = client_decode([(spec.get('id', 60000), b)], plain=mode)[0]
            diff = [k for k in DEFAULT if got[k] != want[k] and k != 'searchable']
            wide = any(i > 65535 for i in [want['model']] + sum(want['wear_models'], []) + sum(want['head_models'], []))
            if mode and not wide: continue        # Definition::decode (no cache opcodes) reads only the 32-bit model forms
            print(f'{a[2]}: {len(b)} bytes; decoded by the client ({"Definition::decode" if mode else "Definition::decode_local"}): '
                  + ('every field as specified' if not diff else f'DIFFERS in {diff}'))
            if diff: sys.exit(1)
        if '--server' in a:
            row = server_row(spec.get('id', 60000), spec.get('base_item'), fields, spec.get('model_files', {}))
            pathlib.Path(a[a.index('--server') + 1]).write_text(json.dumps(row, indent=2), encoding='utf-8')
    elif a[0] == 'show':
        r = client_decode([(60000, pathlib.Path(a[1]).read_bytes())])[0]
        r.pop('raw'); print(json.dumps(r, indent=1))


if __name__ == '__main__':
    main()
