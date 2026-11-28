"""Assemble local review evidence. SINCE_DATE is inclusive, in machine local time.

Build and verdict selection uses modification time (including descendants).
Saved verdicts describe the submission at the time of the saved measurement.
Fresh checks use the current binary and the brief's limits where identifiable.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse
from collections import defaultdict
from datetime import date, datetime, time
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace
from urllib.parse import quote

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = Path(_AP.fix('<ROOT>'))
BUILDS = ROOT / 'builds'
VERDICTS = Path(_AP.fix('<VERDICTS_DIR>'))
ASTRA = ROOT / 'trial/astra'
OPUS = ROOT / 'trial/opus'
TRIAL = {
    'original-trial-astra-chest-a': (ASTRA, 'gate-0-*', '74f352a3'),
    'original-trial-astra-chest-b': (ASTRA, 'gate-1-*', '352a4159'),
    'trial-astra-table': (ASTRA, 'gate-2-*', 'cf0e435a, cb46a6ea, e8b5fac5'),
    'original-trial-opus-docchest-a': (OPUS, 'gate-docchest-a-*', '7753f41c, 9f7b804d'),
    'original-trial-opus-docchest-b': (OPUS, 'gate-docchest-b-*', '670a82c6, 98dfcf48'),
    'trial-opus-chest-2677': (OPUS, 'gate-chest-2677-*', '64533eac, 9a610155'),
}


def read(path):
    raw = path.read_bytes()
    return raw.decode('utf-16' if raw.startswith((b'\xff\xfe', b'\xfe\xff')) else 'utf-8-sig', errors='replace').replace('\r\n','\n')


def write(path, body):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body.rstrip() + '\n', encoding='utf-8')


def link(path, out):
    return '[' + path.name + '](' + quote(path.relative_to(out).as_posix()) + ')'


def verdict(text):
    matches = re.findall(r'^\s*VERDICT (PASS|BORDERLINE|FAIL)\b', text, re.M)
    return matches[-1] if matches else None


def gate_lines(text):
    return '\n'.join(x for x in text.splitlines() if re.search(r'\| (PASS|INFO|BORDERLINE|FAIL)\s*$|^VERDICT ', x))


def saved(build):
    paths = []
    if build.name in TRIAL:
        folder, pattern, _ = TRIAL[build.name]
        paths.extend(folder.glob(pattern))
    short = build.name.removeprefix('original-')
    for group in ('trial', 'night'):
        for path in (ROOT/group).glob('*/*'):
            if short in path.stem and 'gate' in path.stem and path.suffix in ('.log','.txt','.json'):
                if path.is_file(): paths.append(path)
    for path in build.iterdir():
        if path.is_file() and ('gate' in path.stem.lower() or path.name == 'REPORT.md') and path.suffix.lower() in ('.md','.txt','.log','.json'):
            if build.name.startswith(('tools-', 'research-')):
                continue
            paths.append(path)
    found = []
    for path in sorted(set(paths), key=lambda p:(p.stat().st_mtime, p.name)):
        content = read(path)
        if path.suffix == '.json':
            obj = json.loads(content)
            if isinstance(obj, dict) and 'checks' in obj and 'verdict' in obj:
                content = '\n'.join(' | '.join(str(row[k]) for k in ('name','value','threshold','verdict')) for row in obj['checks']) + '\nVERDICT ' + obj['verdict']
        if verdict(content):
            found.append((path.relative_to(ROOT).as_posix(), gate_lines(content)))
    # Previous handoff: machine-readable check lists associated with a named binary.
    handoff = BUILDS/'tools-handoff'
    if not found and build != handoff:
        for command in read(handoff/'REPORT.md').splitlines():
            if f'builds/{build.name}/' not in command or 'gate.py' not in command:
                continue
            match = re.search(r"'--json' '([^']+)'", command)
            if match:
                path = ROOT / match[1]
                if path.is_file():
                    obj = json.loads(read(path))
                    if 'checks' in obj:
                        found.append((path.name + ' (saved handoff measurement)', '\n'.join(' | '.join(str(row[k]) for k in ('name','value','threshold','verdict')) for row in obj['checks']) + '\nVERDICT ' + obj['verdict']))
    return found


def fresh_args(build, briefs):
    from held_out import assignments, decode, scene
    name = build.name
    short = name.removeprefix('original-')
    if name.startswith(('research-', 'room-', 'tools-')) or name == 'warden-set':
        return [], 'No applicable piece gate.'
    model = next((p for p in [build/(short+'-labelled.bin'), build/(short+'.bin'), build/(name+'.bin')] if p.is_file()), None)
    if not model:
        variants = sorted(p.name for p in build.glob('*-labelled.bin'))
        if variants:
            return [], 'Multiple candidate binaries without one recorded submission recipe: ' + ', '.join(variants)
        return [], 'No identified submission binary.'
    for kind, entries in assignments().items():
        for ident, names in entries.items():
            if name not in names: continue
            if kind == 'object':
                d = decode(scene.object_def(ident))
                return ['rebuild-object', ident, model,'--ambient',d['ambient'],'--contrast',d['contrast']], ''
            if 'fang' in name or 'crossbow' in name:
                return ['rebuild-weapon', f'item:{ident}', model], ''
            slot = {1079:'legs',1127:'body+arms',1101:'body',10828:'head+hair+jaw'}[ident]
            return ['worn', model, slot, '--real', f'item:{ident}'], ''
    brief = briefs.get(short, '')
    dims = re.search(r'(\d+,\d+,\d+,\d+,\d+,\d+)', brief)
    faces = re.search(r'(\d+) to (\d+) triangles|triangle range (\d+),(\d+)', brief)
    if name.startswith('original-') and dims and faces:
        return ['original-object',model,'--faces',','.join(x for x in faces.groups() if x),'--size',dims[1]], ''
    return [], 'No complete gate recipe identified for this binary.'


def measure(scratch, args):
    # Keep both explicit TemporaryDirectory sites of the installed gate within OUT_DIR.
    sys.path.insert(0, str(ROOT/'tools'))
    import gate
    def local_temp(**options):
        return tempfile.TemporaryDirectory(dir=scratch, prefix=options.get('prefix','gate-'))
    gate.tempfile = SimpleNamespace(TemporaryDirectory=local_temp)
    sys.argv = ['gate.py', *args]
    return gate.main()


def remaining(path):
    if not path.is_file(): return 'REPORT.md missing.'
    body = read(path)
    sections = re.split(r'(?=^#{1,6} )', body, flags=re.M)
    selected = [s for s in sections if re.search(r'differ|honest|judg|remain|fault|limitation|still|assessment|self.review|reservation', s.partition('\n')[0], re.I)]
    return '\n'.join(selected).strip() if selected else 'Full report (no separately headed assessment)\n\n' + body


TOPICS = {
    'Lighting lookup': r'objlight\.py',
    'Mixed line-up framing': r'line-up framing|per-entry footprint|footprint.*centre|framing crops|crop clipped|desk clipped|crop to fit',
    'Transparent materials': r'alpha parameter|transparent material|translucen',
    'Sheet projection and hidden geometry': r'hidden rail depths|hidden joint depths|mirroring the back|half-pixel projection',
    'Held-out model register': r'held.out (selection|list)|calibration register',
    'Visible colour and covered surfaces': r'exported surface shares|exporter.*(shares|covered surfaces|area calculation)|surface.area percentages',
    'Blender errors and stale exports': r'exit flag|export crash|stale binary',
    'Recognising the ordered object': r'visual recognition|judge the opening|read as the thing ordered',
    'Isolated sitting pages': r'shared review shortcut|repoints the owner',
}


def proposals(path, groups):
    entries = []
    for line in read(path).splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith(('#', '| Source', '| ---')):
            continue
        if line[:1].isspace() and entries:
            entries[-1] += ' ' + stripped
        else:
            entries.append(stripped)
    for entry in entries:
        cleaned = re.sub(r'^\d+\. |^- ', '', entry).strip('| ').strip()
        topic = next((k for k, pattern in TOPICS.items() if re.search(pattern,cleaned,re.I)), cleaned)
        groups[topic].append((path.parent.relative_to(ROOT).as_posix(), cleaned))


def collect(since, out):
    since_ts = datetime.combine(since,time()).timestamp()
    out.mkdir(parents=True, exist_ok=True)
    producers = sorted(p for group in ('trial','night') for p in (ROOT/group).iterdir() if p.is_dir())
    groups, briefs, sessions = defaultdict(list), {}, {}
    # Explicit assignments in the project repository/teacher/project/STATE.md (2026-11-26).
    sessions.update({'qual-legs-astra':'27af944d','qual-platebody-astra':'6ba01426',
                     'meridian-crush-a':'0e3ff851','meridian-crush-b':'2c3fd7fc',
                     'room-observatory':'c8e74098','tools-handoff2':'d9b1fc98'})
    missing, parked, records, owner = [], [], [], []
    for folder in producers:
        label = folder.relative_to(ROOT).as_posix()
        for name in ('STATE','COUNT','PROPOSALS'):
            path = folder/(name+'.md')
            if not path.is_file():
                missing.append(f'{label}: {path.name} missing')
                continue
            dest = out/'producers'/label/path.name
            write(dest,read(path))
            records.append(label + ' ' + link(dest,out))
            if name == 'PROPOSALS': proposals(path,groups)
        for path in folder.glob('prompts/*.txt'):
            briefs[path.stem] = read(path)
        path = folder/'sessions.txt'
        if path.is_file():
            for line in read(path).splitlines():
                fields = line.split()
                if len(fields) >= 2: sessions[fields[0]] = fields[1]
        for path in sorted((folder/'parked').glob('*')):
            parked.append(f'{label}/parked/{path.name}')
    verdict_files = []
    for path in sorted(VERDICTS.iterdir()):
        if path.is_file() and path.stat().st_mtime >= since_ts:
            dest = out/'owner'/path.name
            dest.parent.mkdir(exist_ok=True)
            shutil.copyfile(path,dest)
            owner.append(link(dest,out))
            verdict_files.append(dest)
    props = ['# Producer proposals']
    for topic, entries in groups.items():
        props.extend(['', '## '+ (topic if topic in TOPICS else entries[0][0]+' — '+re.sub(r'\*|\|','',topic)[:65].strip()),''])
        props.extend(f'- {source} — {entry}' for source,entry in entries)
    write(out/'PROPOSALS-ALL.md','\n'.join(props))
    builds = sorted(p for p in BUILDS.iterdir() if p.is_dir())
    names = {b.name for b in builds}
    renders = defaultdict(list)
    for folder in (ROOT/'render').iterdir():
        if not folder.is_dir(): continue
        matches = [n for n in names if folder.name == n or folder.name.startswith(n+'-')]
        if matches: renders[max(matches,key=len)].append(folder)
    rows = ['| Piece | Kind | Author session | Gate | Picture | Owner verdict |','|---|---|---|---|---|---|']
    for build in builds:
        if build == out or out in build.parents: continue
        if build.stat().st_mtime < since_ts and not any(p.stat().st_mtime >= since_ts for p in build.rglob('*') if p.is_file() and out not in p.parents):
            continue
        name = build.name
        short = name.removeprefix('original-')
        kind = ('original scenery' if name.startswith('original-') else 'tools' if name.startswith('tools-') else
                'research' if name.startswith('research-') else 'room' if name.startswith('room-') else
                'original worn gear' if name.startswith('meridian-') else 'worn rebuild' if name.startswith(('armour-','weapon-','qual-')) else
                'assembled set' if name=='warden-set' else 'scenery rebuild' if name != 'barrel' else 'original scenery')
        candidates = []
        for folder in [*renders[name],build/'render',build/'renders']:
            for pic in folder.glob('**/*.png'):
                if pic.name.lower().startswith(('lineup','figure')) or pic.with_suffix('.key.txt').is_file():
                    candidates.append(pic)
        if candidates:
            picture = max(candidates,key=lambda p:(p.stat().st_mtime,p.name))
            dest = out/'pictures'/(name+'.png')
            dest.parent.mkdir(exist_ok=True)
            shutil.copyfile(picture,dest)
            pic_link = link(dest,out)
        else:
            picture = None
            pic_link = 'Missing'
            missing.append(f'{name}: no line-up or figure picture found')
        gates = saved(build)
        reason = ''
        if not gates:
            args, reason = fresh_args(build, briefs)
            recipes = [args] if args else []
            if name == 'meridian-helm-v3':
                # design.md specifies the slot and house steel for all three candidates.
                recipes = []
                for model in sorted(build.glob('*-labelled.bin')):
                    options = ['--steel','42,0']
                    if model.name.endswith(('-b-labelled.bin','-c-labelled.bin')):
                        options += ['--non-steel','32,1,104','--non-steel','32,1,90']
                    recipes.append(['worn',model,'head+hair+jaw',*options])
            for recipe in recipes:
                cmd = [sys.executable,'-I','-B',str(Path(__file__).resolve()),'--measure',str(out),*map(str,recipe)]
                result = subprocess.run(cmd,cwd=out,capture_output=True,text=True,errors='replace')
                if verdict(result.stdout):
                    gates.append(('Fresh: '+' '.join(map(str,recipe)),result.stdout.strip()))
                else: reason = (result.stderr or result.stdout or str(result.returncode)).strip()
        status = verdict(gates[-1][1]) if gates else 'Unavailable'
        if name == 'meridian-helm-v3' and len(gates) > 1:
            status = ', '.join(verdict(lines) for _, lines in gates) + ' (see candidates)'
        if not gates: missing.append(f'{name}: {reason}')
        report = remaining(build/'REPORT.md') if build != HERE else 'Pack generator (this delivery).'
        if report == 'REPORT.md missing.': missing.append(name+': '+report)
        data = out/'pieces'/(name+'.md')
        details = [f'# {name}',kind,'## Gate']
        details.extend(f'{source}\n\n```text\n{body}\n```' for source,body in gates)
        if not gates: details.append(reason)
        details.extend(['## Picture source',str(picture or 'Missing'),'## Author assessment',report])
        source = read(build/'REPORT.md') if build != HERE and (build/'REPORT.md').is_file() else ''
        costs = [line for line in source.splitlines() if re.search(r'\b(tokens?|minutes?|elapsed time|wall.clock)\b',line,re.I)]
        details.extend(['## Reported time and usage', '\n'.join(costs) or 'No time or token count found in the build report.'])
        write(data,'\n\n'.join(details))
        sid = TRIAL.get(name, ('','',''))[2] or sessions.get(short,'Unknown')
        related = []
        for v in verdict_files:
            if v.suffix != '.json': continue
            content = read(v)
            # Sitting g keys are explicitly assigned in its spec and COUNT.
            sit = v.stem.removeprefix('sitting-')
            historic = {
                '2026-11-25-a': ('lectern-22472','crate-15704','bench-55076','barrel'),
                '2026-11-25-b': ('lectern-22472','crate-15704','original-scribe-desk','barrel'),
                '2026-11-26-a': ('original-scribe-desk','original-scribe-chair','original-scribe-bookcase','original-scribe-cabinet'),
                '2026-11-26-b': ('weapon-fang-33174','weapon-crossbow-33460','meridian-glaive','meridian-staff','meridian-sword'),
                '2026-11-26-c': ('room-scribes-hall',),
                '2026-11-26-d': ('room-scribes-hall','armour-chainbody-1101','meridian-sword','meridian-glaive','meridian-staff'),
                '2026-11-26-e': ('cmp-drawers-astra','cmp-drawers-claude','cmp-drawers-opus','original-maptable-astra','original-maptable-sol','warden-set'),
                '2026-11-26-f': ('meridian-helm-v2',),
                '2026-11-26-g': (*TRIAL,'meridian-helm-v2'),
            }
            if re.search(r'(?<![\w-])'+re.escape(name)+r'(?![\w-])',content) or name in historic.get(sit,()):
                summary = ''
                if sit == '2026-11-26-g':
                    key = ('table' if name=='trial-astra-table' else 'chest' if name=='trial-opus-chest-2677'
                           else 'helm' if name=='meridian-helm-v2' else 'docchests')
                    answer = json.loads(content).get('answers',{}).get(key,{})
                    parts = [f'{k} {val}' for k,val in answer.items() if k in ('pick','answer','sure','grade')]
                    if parts: summary = ', '.join(parts).replace('|','\\|') + ' '
                related.append(summary + link(v,out))
        rows.append(f'| [{name}](pieces/{name}.md) | {kind} | {sid} | {status} | {pic_link} | {" ".join(related) or "No match"} |')
    sections = ['# Fable review since '+since.isoformat(), '\n'.join(rows), '## Producer records', '\n'.join('- '+r for r in records),
                '[Merged proposals](PROPOSALS-ALL.md)', '## Owner files','\n'.join('- '+x for x in owner) or 'None.',
                '## Parked archives','\n'.join('- '+x for x in parked) or 'None.', '## Missing evidence','\n'.join('- '+x for x in missing) or 'None.']
    write(out/'INDEX.md','\n\n'.join(sections))
    print(f'{len(rows)-2} builds, {len(producers)} Producers, {len(verdict_files)} owner files, {len(parked)} parked archives, {sum(map(len,groups.values()))} proposals')
    print(*missing,sep='\n')


if __name__ == '__main__':
    if sys.argv[1:2] == ['--measure']:
        sys.exit(measure(Path(sys.argv[2]),sys.argv[3:]))
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('since_date',type=date.fromisoformat)
    ap.add_argument('out_dir',type=Path)
    args = ap.parse_args()
    sys.path.insert(0,str(HERE))
    collect(args.since_date,args.out_dir.resolve())
