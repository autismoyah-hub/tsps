"""Score a Titan submission in an isolated results folder under teacher-only."""
import sys
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent))
from titan import ATTACKS, DEATH, ROOT, tl, L, cmesh, dlib, detail, detailgate, decomp, gate_npc, setup, write
import argparse, tempfile, contextlib
def run(n,path):
    parent=ROOT/'teacher-only'/str(n)/'submissions'
    parent.mkdir(parents=True,exist_ok=True)
    w=Path(tempfile.mkdtemp(prefix=path.stem+'-',dir=parent))
    setup(n,w)
    worst=0
    print(w,flush=True)
    for attack in ATTACKS[n]:
        L.KNOWN_SEQS[n]['attack']=attack
        with (w/f'gate-{attack}.txt').open('w') as fp, contextlib.redirect_stdout(fp):
            code,rows,_=gate_npc.gate(n,str(path),overrides=dict(attack=attack,death=DEATH[n]))
        for row in rows:print(row['name'],row['value'],row['verdict'],sep=' | ')
        print('Attack',attack,gate_npc.LEVELS[code],flush=True)
        worst=max(worst,code)
        with (w/f'tone-{attack}.txt').open('w') as fp, contextlib.redirect_stdout(fp):
            out=decomp.run(f'tone-{attack}',n,path,pictures=False)
        weighted=decomp.agg(out['cells'],lambda c:c['all'])
        print('Solid body · paint and light',weighted['paint_alone'],weighted['light_alone'],flush=True)
    L.KNOWN_SEQS[n]['attack']=ATTACKS[n][0]
    result=detailgate.line(detail.run(f'submission={n}:{path}'))
    detailgate.show(result);write(w/'detail.json',result)
    return max(worst,gate_npc.LEVELS.index(result['verdict']))
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['rebuild-npc'])
    parser.add_argument('npc',type=int,choices=list(ATTACKS))
    parser.add_argument('definition',type=Path)
    args=parser.parse_args()
    sys.exit(run(args.npc,args.definition.resolve()))

