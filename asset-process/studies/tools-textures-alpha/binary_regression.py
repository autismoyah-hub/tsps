"""For each distinct opaque model, reconstruct a Blender mesh, re-encode its sections and compare every stored file."""
import contextlib,hashlib,json,os,sys,runpy
from pathlib import Path
HERE=Path(__file__).resolve().parent
TOOLS=HERE.parents[1]/'tools'
sys.path.insert(0,str(TOOLS))
import bpy
def invoke(name,args):
    sys.argv=['blender','--',*map(str,args)]
    runpy.run_path(str(TOOLS/name),run_name='__main__')
def main():
    groups=json.loads((HERE/'inventory.json').read_bytes())
    result=HERE/'binary-regression.jsonl'
    passed=set()
    if result.exists() and '--fresh' not in sys.argv:
        for line in result.read_text().splitlines():
            r=json.loads(line)
            if r.get('exact'):passed.add(r['hash'])
    output=HERE/'packed-output.tmp'
    with result.open('a') as log,open(os.devnull,'w') as quiet:
        for h,paths in groups.items():
            if h in passed:continue
            try:
                with contextlib.redirect_stdout(quiet):
                    invoke('real_to_blender.py',[paths[0],'unused','--in-memory'])
                    invoke('blender_export.py',[output,'--preserve-packing'])
                b=output.read_bytes()
                exact=all(b==Path(p).read_bytes() for p in paths)
                row=dict(hash=h,count=len(paths),exact=exact)
                if not exact:
                    src=Path(paths[0]).read_bytes()
                    row.update(size=[len(src),len(b)],first=next((i for i,(a,c) in enumerate(zip(src,b)) if a!=c),-1))
            except (Exception,SystemExit) as e:row=dict(hash=h,count=len(paths),error=repr(e))
            log.write(json.dumps(row)+'\n');log.flush()
if __name__=='__main__':main()
