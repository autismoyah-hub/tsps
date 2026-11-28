"""Find additional compatible sequences. Frame-base matches do not assign action names."""
import sys,pathlib,json,collections
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from effects import c,frames
from definitions import ROOT,records
bybase=collections.defaultdict(list)
for i,data in records(c,'sequence'):
 n,g,b=frames(i)
 for base in b:bybase[base].append([i,n])
rows=json.loads((ROOT/'study.json').read_text())
osrs=collections.defaultdict(list);osrsseq={}
for line in (ROOT.parent/'research-npc/census/sequences.jsonl').open():
 d=json.loads(line);osrsseq[d['id']]=d
 if d['kind']=='classic':osrs[d['skeleton']].append([d['id'],d['frames']])
md=['# Boss animation families','','Base animation set links identify the stand, walk and weighted idle roles. Other sequences below use a frame base found in those animations. Multiple NPCs can share a base. Attack and death roles require additional encounter evidence. Counts are stored frames.','']
for r in rows:
 if r['kind']!='npc':continue
 hd=r['animations']
 bases=sorted(set().union(*(frames(a)[2] for a,n in hd.values())))
 ref=r['ref'];an=ref['anim'];rb=sorted({osrsseq[a]['skeleton'] for a,n in an.values() if a in osrsseq and osrsseq[a]['kind']=='classic'})
 md.extend([f"## {r['name'].strip()} · 614 NPC {r['id']}",'',f"239 control: {ref['name']} (NPC {ref['id']}).",'',
            '| Revision | Frame base | Distinct sequences | Sequence ID / frame count |','|---|---:|---:|---|'])
 for rev,bank,keys in [(614,bybase,bases),(239,osrs,rb)]:
    for base in keys:
        seq=bank[base]
        md.append(f"| {rev} | {base} | {len(seq)} | "+', '.join(f'{a}/{n}' for a,n in seq)+' |')
 md.append('')
(ROOT/'ANIMATIONS.md').write_text('\n'.join(md),encoding='utf8')

