"""Open every distinct stored Blender source. Compare normal exporter entry points and stored outputs."""
import bpy, sys, runpy, json, hashlib, contextlib, os, traceback
import importlib.util
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1];TOOLS=ROOT/'tools'
sys.path.insert(0,str(TOOLS))
import osrscache
import author
spec=importlib.util.spec_from_file_location('baseline_author',TOOLS/'author.pre-textures.py')
baseline_author=importlib.util.module_from_spec(spec)
spec.loader.exec_module(baseline_author)
def digest(b):return hashlib.sha256(b).hexdigest()
sources={}
for p in (ROOT/'builds').rglob('*.blend'):
    if HERE not in p.parents:sources.setdefault(digest(p.read_bytes()),[]).append(str(p))
stored=json.loads((HERE/'inventory.json').read_bytes());report=HERE/'blend-regression.jsonl'
def export(script,options):
    dest=HERE/'regression-output.tmp'
    sys.argv=['blender','--',str(dest),*options]
    sys.modules['author']=baseline_author if script.endswith('.pre-textures.py') else author
    try:
        runpy.run_path(str(TOOLS/script),run_name='__main__')
    finally:
        sys.modules['author']=author
    return dest.read_bytes()
with report.open('w') as log,open(os.devnull,'w') as quiet:
    for paths in sources.values():
        p=Path(paths[0]);matches=[];trials=[]
        try:
            bpy.ops.wm.open_mainfile(filepath=str(p),load_ui=False,use_scripts=False)
            options=[[]]
            for path in paths:
                near=Path(path)
                for b in near.parent.glob(near.stem+'*.bin'):
                    try: m=osrscache.decode_model(b.read_bytes())
                    except Exception:continue
                    vs=set(m['vertex_skins'] or []);fs=set(m['face_skins']);ps=set(m['priorities'])
                    args=[]
                    if len(vs)==1 and len(fs)==1:args+=['--labels',f'{min(vs)},{min(fs)}']
                    if len(ps)==1 and min(ps)!=4:args+=['--priority',str(min(ps))]
                    if args not in options:options.append(args)
            for args in options:
                with contextlib.redirect_stdout(quiet):
                    old=export('blender_export.pre-textures.py',args)
                    new=export('blender_export.py',args)
                trials.append(old==new)
                matches+=stored.get(digest(new),[])
            row=dict(paths=paths,trials=trials,matches=sorted(set(matches)))
        except (Exception,SystemExit) as e:row=dict(paths=paths,error=repr(e),trials=trials,matches=matches)
        log.write(json.dumps(row)+'\n');log.flush()
