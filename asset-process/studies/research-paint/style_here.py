"""Run research-creatures/style_npc.py unchanged on entries, with its cache kept in THIS folder (data/).
usage: style_here.py ENTRY [ENTRY...]     ENTRY = def:PATH.json | npc:ID"""
import sys, pathlib
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import paintlib                      # sets cmesh.DATA to this folder
import cmesh, style_npc
for e in sys.argv[1:]:
    r = cmesh.measure(cmesh.load(e)); print('==', e); style_npc.check(r)
