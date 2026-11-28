"""Render a set. Paths are relative to SETFILE. Default is pose.py; --figure requests three angles."""
import argparse
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from handoff import TOOLS, entry, run


def main():
    p = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    p.add_argument('out', type=Path)
    p.add_argument('setfile', type=Path)
    p.add_argument('--figure', action='store_true')
    a, options = p.parse_known_args()
    spec = json.loads(a.setfile.read_text())
    ours = [entry(a.setfile.parent / piece['path'], piece['slot']) for piece in spec['pieces']]
    if not ours:
        p.error('empty set')
    columns = ['+'.join(ours)]
    for outfit in spec.get('real_outfits', []):
        if not re.fullmatch(r'(?:item:\d+)(?:\+item:\d+)*|none', outfit):
            p.error('real_outfits entries must be joined item:ID strings or none')
        columns.append(outfit)
    a.out.parent.mkdir(parents=True, exist_ok=True)
    script = 'figure.py' if a.figure else 'pose.py'
    print(run(TOOLS/script, a.out, ','.join(columns), *options), end='')


if __name__ == '__main__':
    main()
