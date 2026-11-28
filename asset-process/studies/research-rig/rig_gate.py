"""The skeletal rebuild gate (research-bosses/gate_bone.py, unmodified) on a model exported by this route.

usage: rig_gate.py WORKDIR OURS_MODEL.bin [more parts..] [--out DIR] [--txt FILE] [--walk SEQ] [--attack SEQ] [--death SEQ]
WORKDIR is rig_fetch.py's (the real creature). Writes OURS.json beside the first model (the real definition's
recolours, scale, ambient, contrast; the creature's sequences) and runs gate_bone.gate(real NPC, OURS.json). The gate's
working files go under research-rig/data (riglib redirects them); nothing is written into research-bosses.
--out DIR keeps the gate's grids (real | ours) and difference strips. Prints the gate's lines; --txt also saves them.
"""
import sys, json, pathlib, io, contextlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import riglib as R
import gate_bone
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text())
models = [pathlib.Path(x).resolve() for x in a[2:] if x.endswith('.bin')]
ov = {k: int(opt('--' + k)) for k in ('attack', 'death', 'stand', 'walk') if opt('--' + k)}
seqs = dict(man['named']); seqs.update(ov)
j = dict(name=man['name'] + ' (weights by rule)', models=[m.name for m in models], recolor=[[int(x) for x in pair.split(':')] for pair in man.get('recolor', '').split(',') if pair],
         scale=man['scale'], ambient=man['ambient'], contrast=man['contrast_stored'], size=man['size'], zbuffer=bool(man['zbuffer']), **seqs)
oj = models[0].with_suffix('.json'); oj.write_text(json.dumps(j, indent=1))
buf = io.StringIO()
class Tee:
    def write(self, s): sys.__stdout__.write(s); buf.write(s)
    def flush(self): sys.__stdout__.flush()
with contextlib.redirect_stdout(Tee()):
    code, rows, Rr = gate_bone.gate(man['npc'], oj, 6, ov, opt('--out'))
if opt('--txt'): pathlib.Path(opt('--txt')).write_text(buf.getvalue())
sys.exit(code)
