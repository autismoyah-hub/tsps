"""decode_real.py: write the five reference interfaces to real/<id>-<name>.json, complete.

Each file holds: the verified id and name, where the id was verified, the definition (revision 239, every field of
every component, with its layout in the pane the game opens it in), the 242 definition where it differs, the
listeners and scripts, the state after the group's own onLoad scripts (from the tooling client), the sprites and
fonts used (with cache names where known), and what is static versus script-driven."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifcache as C, iface

GAMEVAL = Path(_AP.fix(r'<TSPS_CHECKOUT>/client/runelite/api/gameval'))
names_if = {int(i): n for n, i in re.findall(r'export const (\w+) = (\d+);', (GAMEVAL / 'InterfaceID.ts').read_text())}
names_sp = {int(i): n for n, i in re.findall(r'export const (\w+) = (\d+);', (GAMEVAL / 'SpriteID.ts').read_text())}
line_of = {int(m.group(2)): n + 1 for n, l in enumerate((GAMEVAL / 'InterfaceID.ts').read_text().splitlines())
           for m in [re.search(r'export const (\w+) = (\d+);', l)] if m}

PICKS = [
    (303, 'boss health overlay', (512, 334), 'opened at login into the gameframe HUD pane: TSPS server ClientProtocol.ts:2803-2804 '
     '"[2, 303]" (161:2 resizable; fixed 548:28); driven by server/plugins/interface/BossHud.plugin.js:31 GROUP = 303'),
    (210, 'simple framed panel with title and close button', (512, 334), 'main modal 161:16 / fixed 548:41 (::interface 210)'),
    (155, 'item container panel (reward chest)', (512, 334), 'main modal; item slots filled by script 1065 from inventory transmit'),
    (231, 'dialogue box (NPC chat, head on the left)', (479, 96), 'chatbox modal 162:567 (239) / 162:568 (242, TSPS '
     'PacketSender.ts:116 CHATBOX_MODAL_TARGET_UID); seen live in TSPS talking to the Lumbridge Guide'),
    (160, 'minimap orbs (hitpoints, prayer, run, special) overlay', (236, 163), 'TSPS server ClientProtocol.ts:2771 ORBS_GROUP_ID = 160; '
     'fixed 548:25, resizable 161:33'),
]


def sprite_info(cache, sid):
    if sid is None or sid < 0:
        return None
    try:
        sh = C.sprite_sheet(cache, sid)
    except KeyError:
        return dict(id=sid, missing=True)
    nm = names_sp.get(sid)
    if nm is None:
        h = cache.reference(8)[sid]['name']
        nm = f'(name hash {h})' if h is not None else None
    return dict(id=sid, name=nm, canvas=[sh.width, sh.height], frames=len(sh.frames),
                frame0=dict(x=sh.frames[0]['x'], y=sh.frames[0]['y'], w=sh.frames[0]['w'], h=sh.frames[0]['h']),
                palette_colours=len(sh.palette) - 1)


def main():
    a, b = C.Cache(), C.Cache(C.CACHE_242)
    fonts = C.font_names(a)
    out_dir = HERE / 'real'
    out_dir.mkdir(exist_ok=True)
    for gid, role, pane, where in PICKS:
        ws = C.load_interface(a, gid)
        lay, order = iface.layout(ws, pane)
        comps = []
        for w in ws:
            p = lay[w['id']]
            d = {k: v for k, v in w.items() if k not in ('trailing',)}
            d['kind_name'] = C.KIND_NAMES.get(w['kind'])
            d['parent_file'] = None if w['parent'] is None else w['parent'] & 0xffff
            d['layout_in_pane'] = dict(pane=list(pane), pos=p['pos'], size=p['size'], abs=p['abs'], hidden=p['hidden'])
            if w['kind'] == 5:
                d['sprite'] = sprite_info(a, w['sprites'][0] if w['if3'] else w['sprites'][1])
            if w['kind'] == 4:
                d['font_name'] = fonts.get(w['font'])
                d['colour_hex'] = '%06x' % (w['colors'][0] & 0xffffff)
            comps.append(d)
        w242 = C.load_interface(b, gid)
        same = a.group(3, gid) == b.group(3, gid)
        st = HERE / 'state' / f'{gid}.jsonl'
        state = [json.loads(l) for l in st.read_text().splitlines()] if st.exists() else []
        drawn = []
        for r in state:
            if r['hidden'] or r['kind'] in (0, 11):
                continue
            e = dict(id=r['id'], file=r['id'] & 0xffff, dynamic=r['child_index'] >= 0, kind=C.KIND_NAMES.get(r['kind']),
                     abs=r['abs'], size=r['size'])
            if r['kind'] == 5:
                e['sprite'] = sprite_info(a, r['sprites'][0]); e['tiled'] = r['sprite_tiling']
                e['transparency'] = r['transparency']
            if r['kind'] == 4:
                e.update(text=r['text'], font=r['font'], font_name=fonts.get(r['font']), colour='%06x' % r['colors'][0],
                         shadow=r['text_shadow'], align=r['text_alignment'], line_height=r['line_height'])
            if r['kind'] == 3:
                e.update(colour='%06x' % r['colors'][0], filled=r['filled'], transparency=r['transparency'])
            if r['kind'] == 6:
                e.update(model=r['models'][0])
            drawn.append(e)
        scripts = [dict(file=w['file'], listeners=w['listeners'], triggers=w['triggers']) for w in ws
                   if w['listeners'] or w['triggers']]
        doc = dict(
            id=gid, name=names_if.get(gid), role=role,
            id_verified_by=[f'TSPS client/runelite/api/gameval/InterfaceID.ts:{line_of.get(gid)} '
                            f'"export const {names_if.get(gid)} = {gid};" (RuneLite gameval names, revision 242)', where],
            cache=dict(revision_239=dict(components=len(ws)), revision_242=dict(components=len(w242), identical_bytes=same)),
            pane=dict(size=list(pane), note='parent size the group is laid out in when the game opens it (fixed mode)'),
            static_vs_scripted=dict(
                components_with_listeners=len(scripts),
                onload_scripts=sorted({w['listeners']['onLoad'][0] for w in ws if 'onLoad' in w['listeners']}),
                hidden_in_definition=[w['file'] for w in ws if w['hidden']],
                note='state_after_onload lists what the client draws once the group\'s own onLoad scripts have run '
                     '(dynamic=True rows were created by those scripts); values the server sends (text, items, varps) '
                     'are not in it'),
            components=comps,
            state_after_onload=dict(source=f'state/{gid}.jsonl (tooling client iface_dump, revision 239)',
                                    drawn=drawn),
            scripts=scripts,
        )
        if not same:
            doc['components_242'] = [{k: v for k, v in w.items() if k != 'trailing'} for w in w242]
        path = out_dir / f'{gid}-{(names_if.get(gid) or "unnamed").lower()}.json'
        path.write_text(json.dumps(doc, indent=1))
        print(path.name, len(comps), 'components;', len(drawn), 'drawn after onLoad;', 'identical in 242' if same else 'differs in 242')


if __name__ == '__main__':
    main()
