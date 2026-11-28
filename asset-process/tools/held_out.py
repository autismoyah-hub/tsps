"""Teacher's register of rebuild and calibration assignments. Check before issuing a brief.

An unqualified number checks all three namespaces. Prefix object:, item: or model:
to disambiguate. Object aliases are resolved from cache definitions, including objects
without an archive sheet. Item model IDs cover every variant listed on its sheet.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse
import json
import re
import sys
from pathlib import Path
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from objlight import decode, scene

# Reviewed assignment evidence. A style neighbour, original brief or ceiling probe
# by itself is not an assignment. Add the build name here when issuing a new brief.
ASSIGNMENTS = {
    'object': {
        55076: 'bench-55076', 15704: 'crate-15704', 22472: 'lectern-22472',
        25702: 'cmp-drawers-astra cmp-drawers-claude cmp-drawers-opus',
        42083: 'trial-astra-table', 2677: 'trial-opus-chest-2677',
        7195: 'obs2-cal-drawers-7195', 13659: 'obs1-cal-crystal-13659',
    },
    'item': {
        33174: 'weapon-fang-33174 qual-fang-astra', 33460: 'weapon-crossbow-33460',
        1101: 'armour-chainbody-1101 qual-chainbody-astra', 10828: 'armour-helm-10828',
        1079: 'qual-legs-astra', 1127: 'qual-platebody-astra',
    },
}


def assignments():
    """Include new single-target briefs even before a build folder exists."""
    entries = {kind: {key: builds.split() for key, builds in rows.items()}
               for kind, rows in ASSIGNMENTS.items()}
    root = Path(_AP.fix('<ROOT>'))
    files = set((root/'compare-authors').glob('prompt*.txt'))
    for group in ('trial','night','builds'):
        files.update((root/group).glob('*/prompts/*.txt'))
        files.update((root/group).glob('prompt*.txt'))
        files.update((root/group).glob('*/prompt*.txt'))
    for path in sorted(files):
        prompt = path.read_text(encoding='utf-8-sig', errors='replace')
        task = re.search(r'^Task line[^:\n]*:(.+)',prompt,re.I|re.M)
        if not task or not re.search(r'BRIEF-(?:furniture|armour|weapon)-rebuild\.md',prompt,re.I):
            continue
        line = task[1]
        target = re.search(r'/(objects|items)/[^/\s]+/(\d+)\.png',line)
        build = re.search(r'\bNAME\s*=\s*([\w-]+)',line) or re.search(r'/builds/([\w-]+)/',line)
        if target and build:
            kind = {'objects':'object','items':'item'}[target[1]]
            names = entries[kind].setdefault(int(target[2]),[])
            if build[1] not in names: names.append(build[1])
    return entries


def definition(kind, ident):
    if kind == 'object':
        raw = scene.config_files(6, 'objects').get(ident)
        if raw is not None:
            obj = decode(raw)
            return obj['name'], set(obj['models'])
    if kind == 'item':
        for path in (scene.ARCH / 'items').glob(f'*/{ident}.json'):
            item = json.loads(path.read_bytes())
            return item['name'], set(item['model_ids'])
    return None, set()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', metavar='ID')
    value = parser.parse_args().check
    namespaces = set(ASSIGNMENTS) | {'model'}
    ident = None
    related = set()
    if value is not None:
        kind, sep, number = value.rpartition(':')
        if (sep and kind not in namespaces) or not number.isdecimal():
            parser.error('expected [object:|item:|model:]nonnegative-id')
        namespaces = {kind} if sep else namespaces
        ident = int(number)
        for kind in namespaces - {'model'}:
            related.update(definition(kind, ident)[1])
        if 'model' in namespaces:
            related.add(ident)
    rows = []
    for kind, entries in assignments().items():
        for key, builds in sorted(entries.items()):
            name, models = definition(kind, key)
            if not models:
                raise SystemExit(f'Missing reference models for {kind}:{key}')
            if ident is None or (key == ident and kind in namespaces) or related & models:
                rows.append(f"{kind}:{key} | {name} | {', '.join(map(str, sorted(models)))} | {' '.join(builds)}")
    if value is not None:
        print(f"{value}: {'used' if rows else 'free'}")
    if rows:
        print('definition | name | model IDs | builds')
        print(*rows, sep='\n')


if __name__ == '__main__':
    main()
