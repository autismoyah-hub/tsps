"""Build the owner's review page for one sitting from a small JSON spec.

usage: python tools/make_sitting.py SPEC.json [--root <REVIEW_DIR>]

The page and its pictures are private and stay on the asset machine; this script and the spec format are not.
Spec:
{
  "id": "2026-11-25-a",                       lowercase, used in the saved file's name
  "title": "...", "intro": "one or two sentences",
  "items": [
    {"id": "bench", "kind": "lineup", "title": "...", "image": "drive:/.../lineup.png", "count": 5,
     "ours": [5], "note": "optional line under the title"},
    {"id": "room", "kind": "pair", "title": "...", "image": "drive:/.../board.png",
     "question": "Which side reads closer to the game?", "options": ["Left", "Right", "No real difference"],
     "after": "bench"}                      optional: hidden until that line-up's pick is locked
  ],
  "ask": ["things the Teacher would find useful to hear, one per line"]
}
Any item that shows one of ours openly MUST carry "after" naming the line-up that tests it blind: the first
page that did not was spoiled (the owner: "[quoted words withheld]").
A line-up's answer key ("ours") is not shown until the owner has locked a pick; the page records the pick
as made before the reveal. The owner clicks "[quoted words withheld]"; tools/collect_sitting.py files the download.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse, base64, html, json, shutil
from pathlib import Path

import re
from datetime import date, datetime
from urllib.parse import quote


def update_index(root):
    rows = []
    for page in (root / 'sittings').glob('*/index.html'):
        sid = page.parent.name
        match = re.search(r'<title>(.*?)</title>', page.read_text(encoding='utf-8'), re.I | re.S)
        title = html.unescape(match[1]) if match else sid
        prefix = re.match(r'\d{4}-\d{2}-\d{2}', sid)
        day = prefix[0] if prefix else datetime.fromtimestamp(page.stat().st_mtime).date().isoformat()
        rows.append((day, sid, title))
    if not rows:
        raise SystemExit('No sitting pages under ' + str(root))
    links = [f'<li><time>{html.escape(day)}</time> <a href="sittings/{quote(sid, safe="")}/index.html">{html.escape(title)}</a></li>'
             for day, sid, title in sorted(rows, reverse=True)]
    index = root / 'index.html'
    backup = root / 'index.before-sittings.html'
    if index.exists() and not backup.exists() and 'sittings/' not in index.read_text(encoding='utf-8', errors='replace'):
        shutil.copyfile(index, backup)
    index.write_text('<!doctype html><html lang="en"><meta charset="utf-8"><title>Asset review sittings</title>'
                     '<h1>Asset review sittings</h1><p>Newest first</p><ol>' + '\n'.join(links) + '</ol></html>', encoding='utf-8')
    print(index)


ap = argparse.ArgumentParser()
ap.add_argument('spec', nargs='?'); ap.add_argument('--index-only', action='store_true'); ap.add_argument('--root', default=_AP.fix('<REVIEW_DIR>'))
a = ap.parse_args()
if a.index_only:
    if a.spec: ap.error('--index-only takes no spec')
    update_index(Path(a.root))
    raise SystemExit
if not a.spec: ap.error('provide SPEC.json or --index-only')
spec = json.loads(Path(a.spec).read_text(encoding='utf-8'))
_lineups = {i['id'] for i in spec['items'] if i.get('kind') == 'lineup'}
for _i in spec['items']:      # only a line-up has a lock button; an item waiting on anything else could never open
    if _i.get('after') and _i['after'] not in _lineups:
        raise SystemExit(f'item {_i["id"]!r}: "after" names {_i["after"]!r}, which is not a line-up item and cannot be locked')
sid = spec['id']
for component in [sid, *(item['id'] for item in spec['items'])]:
    if not re.fullmatch(r'[A-Za-z0-9_-]+', component):
        raise SystemExit(f'Invalid path component {component!r}')
root = Path(a.root); out = root / 'sittings' / sid
out.mkdir(parents=True, exist_ok=True)
items = []
for it in spec['items']:
    src = Path(it['image']); name = f"{it['id']}{src.suffix}"
    shutil.copyfile(src, out / name)
    d = {k: v for k, v in it.items() if k not in ('image', 'ours')}
    d['image'] = name
    if it['kind'] == 'lineup':
        d['key'] = base64.b64encode(json.dumps(it['ours']).encode()).decode()
    items.append(d)
data = json.dumps({'id': sid, 'items': items}, ensure_ascii=False)
E = html.escape
ask = ''.join(f'<li>{E(x)}</li>' for x in spec.get('ask', []))
page = f"""<!doctype html><html lang="en"><meta charset="utf-8"><title>{E(spec['title'])}</title>
<style>
body{{background:#14181d;color:#e6e6e6;font:16px/1.5 Segoe UI,Arial,sans-serif;margin:0 auto;max-width:1500px;padding:24px}}
h1{{font-size:26px;margin:0 0 6px}} h2{{font-size:20px;margin:0 0 4px}} .muted{{color:#9aa4ae}}
section{{background:#1c2229;border:1px solid #2c3540;border-radius:8px;margin:22px 0;padding:18px}}
img{{width:100%;border-radius:4px;display:block;margin:12px 0;cursor:zoom-in}} img.zoom{{width:auto;max-width:none;cursor:zoom-out}}
.scroll{{overflow:auto}}
button{{background:#2a323c;color:#e6e6e6;border:1px solid #3d4855;border-radius:6px;font:inherit;margin:4px 6px 4px 0;padding:8px 16px;cursor:pointer}}
button.on{{background:#2f6fb0;border-color:#5aa0e6}} button:disabled{{opacity:.55;cursor:default}}
button.save{{background:#2e7d4f;border-color:#58b886;font-size:18px;padding:12px 26px}}
textarea{{background:#11151a;color:#e6e6e6;border:1px solid #3d4855;border-radius:6px;font:inherit;width:100%;min-height:70px;padding:8px;box-sizing:border-box}}
.q{{margin:14px 0 4px;font-weight:600}} .reveal{{background:#23303d;border-radius:6px;margin-top:10px;padding:10px 14px}}
#bar{{position:sticky;bottom:0;background:#14181d;border-top:1px solid #2c3540;padding:12px 0}}
</style>
<h1>{E(spec['title'])}</h1><p class="muted">{E(spec.get('intro', ''))} Click a picture to see it at full size. Everything is optional; answer what you have a view on. Your answers are kept in this browser as you type and leave the machine only as the file you save.</p>
<div id="items"></div>
<section><h2>Anything else</h2><p class="muted">Useful to the Teacher, if you have it:</p><ul class="muted">{ask}</ul><textarea id="general" placeholder="Whatever you want the Teacher to know"></textarea></section>
<div id="bar"><button class="save" onclick="save()">Save answers</button> <span id="saved" class="muted"></span></div>
<script>
const S={data};const K='sitting-'+S.id;let A=JSON.parse(localStorage.getItem(K)||'{{}}');
const GRADES=['Rework','Usable','Strong'];
function keep(){{localStorage.setItem(K,JSON.stringify(A));document.getElementById('saved').textContent='';}}
function el(t,p,c,x){{const e=document.createElement(t);if(c)e.className=c;if(x!==undefined)e.textContent=x;p.appendChild(e);return e;}}
function choice(p,it,field,opts,multi){{const w=el('div',p);const a=A[it.id];
 opts.forEach(o=>{{const b=el('button',w,'',String(o));const cur=a[field];
  if(multi?(cur||[]).includes(o):cur===o)b.className='on';
  if(a.locked&&field==='pick')b.disabled=true;
  b.onclick=()=>{{if(multi){{let v=new Set(a[field]||[]);if(typeof o==='string'){{v=new Set([o]);}}else{{[...v].forEach(x=>{{if(typeof x==='string')v.delete(x)}});v.has(o)?v.delete(o):v.add(o);}}a[field]=[...v];}}else a[field]=cur===o?null:o;keep();draw();}};}});}}
function text(p,it,field,ph){{const t=el('textarea',p);t.placeholder=ph;t.value=A[it.id][field]||'';t.oninput=()=>{{A[it.id][field]=t.value;keep();}};}}
function draw(){{const root=document.getElementById('items');root.innerHTML='';
 S.items.forEach(it=>{{A[it.id]=A[it.id]||{{}};const a=A[it.id];if(it.after&&!(A[it.after]&&A[it.after].locked)){{const h=el('section',root);el('div',h,'muted','One more item appears here after you lock your pick on an item above.');return;}}const s=el('section',root);el('h2',s,'',it.title);if(it.note)el('div',s,'muted',it.note);
  const sc=el('div',s,'scroll');const im=el('img',sc);im.src=it.image;im.onclick=()=>im.classList.toggle('zoom');
  if(it.kind==='lineup'){{
   el('div',s,'q','Which, if any, do you think is ours? Pick one or more numbers, or one of the other answers.');
   choice(s,it,'pick',[...Array(it.count).keys()].map(i=>i+1).concat(['None stands out',"Can't tell"]),true);
   el('div',s,'q','How sure are you?');choice(s,it,'sure',['Guessing','Fairly sure','Certain'],false);
   text(s,it,'tell','What gives it away, or what made you unsure');
   if(!a.locked){{const b=el('button',s,'','Lock my pick and show which is ours');b.disabled=!(a.pick&&a.pick.length);b.onclick=()=>{{a.locked=true;a.locked_at=new Date().toISOString();keep();draw();}};}}
   else{{const ours=JSON.parse(atob(it.key));const r=el('div',s,'reveal');r.textContent='Ours: number '+ours.join(', ')+'. Your pick was locked before this was shown.';
    el('div',s,'q','Now that you know: how good is ours?');choice(s,it,'grade',GRADES,false);
    text(s,it,'after','Anything you see now that you know which it is; what would make it better');}}
  }}else{{el('div',s,'q',it.question);choice(s,it,'answer',it.options,false);text(s,it,'why','Why; what you notice');}}
 }});
 const g=document.getElementById('general');g.value=A._general||'';g.oninput=()=>{{A._general=g.value;keep();}};}}
function save(){{const o={{sitting:S.id,saved_at:new Date().toISOString(),answers:A}};const b=new Blob([JSON.stringify(o,null,2)],{{type:'application/json'}});
 const l=document.createElement('a');l.href=URL.createObjectURL(b);l.download='sitting-'+S.id+'-answers.json';l.click();
 document.getElementById('saved').textContent='Saved to your Downloads folder as '+l.download+'. Tell the Teacher "saved", or just "continue".';}}
draw();
</script></html>
"""
(out / 'index.html').write_text(page, encoding='utf-8')
update_index(root)
print(out / 'index.html')
