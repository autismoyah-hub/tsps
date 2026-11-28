"""Rerun both inherited measurement programs in a temporary output tree.
Re-export all raw definitions and fetch and decode model bytes from the supplied cache."""
import pathlib,sys,tempfile,runpy,hashlib,contextlib,subprocess
P=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(P))
import itemlib as L
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
with tempfile.TemporaryDirectory(dir=P/'completed') as temp:
    L.HERE=pathlib.Path(temp)
    data=L.HERE/'data';data.mkdir()
    for script in ('measure_items','measure_items2'):
        with (P/'completed'/(script+'.txt')).open('w',encoding='utf8') as out,contextlib.redirect_stdout(out):
            print(subprocess.list2cmdline([sys.executable,'-I',__file__]),flush=True)
            if script=='measure_items':
                args=[L.RESEARCH_ITEMS,L.CACHE,'defs',str(data/'items.jsonl')]
                p=subprocess.run(args,capture_output=True,text=True,check=True)
                print(subprocess.list2cmdline(args),p.stdout)
            runpy.run_path(str(P/(script+'.py')),run_name='__main__')
            for name in ('items.jsonl','measure-items.json','kinds.json'):
                if (data/name).exists():print(name,digest(data/name),'matches inherited:',digest(data/name)==digest(P/'data'/name))

